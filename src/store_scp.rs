use color_eyre::eyre::{Context, ContextCompat};
use dicom::{
    core::{DataElement, PrimitiveValue, VR},
    dicom_value,
    dictionary_std::tags,
    encoding::TransferSyntaxIndex,
    object::{FileMetaTableBuilder, InMemDicomObject, StandardDataDictionary},
    transfer_syntax::TransferSyntaxRegistry,
};
use dicom_ul::{association::ServerAssociationOptions, pdu::PDataValueType, Pdu};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use tracing::{debug, info, warn};

use crate::utils::{Node, Status, ABSTRACT_SYNTAXES};

/// store_SCP is a DICOM node that stores incoming data
pub(crate) fn store_scp(node: &mut Node) -> color_eyre::Result<()> {
    let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), node.port);
    let listener = match TcpListener::bind(listen_addr) {
        Ok(l) => l,
        Err(e) => {
            warn!(
                "Error binding the TCP listener at {}:{} : {}",
                node.ip, node.port, e
            );
            return Ok(());
        }
    };

    let mut buffer: Vec<u8> = Vec::with_capacity(node.max_pdu as usize);
    let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut msgid = 1;
    let mut sop_class_uid = "".to_string();
    let mut sop_instance_uid = "".to_string();

    let mut options = ServerAssociationOptions::new()
        .accept_any() // TODO: accept only the peers in the config
        .ae_title(node.aet.clone())
        .strict(node.strict);

    if node.uncompressed_only {
        options = options
            .with_transfer_syntax("1.2.840.10008.1.2")
            .with_transfer_syntax("1.2.840.10008.1.2.1");
    } else {
        for ts in TransferSyntaxRegistry.iter() {
            if !ts.unsupported() {
                options = options.with_transfer_syntax(ts.uid());
            }
        }
    };

    for uid in ABSTRACT_SYNTAXES {
        options = options.with_abstract_syntax(*uid);
    }

    debug!("Dicom node {:?} configuration: {:?}", node.aet, options); // TODO: improve the debug output

    info!("{:?} listening on {}:{}", node.aet, node.ip, node.port);
    node.status = Status::Started;

    for tcp_stream in listener.incoming() {
        let stream = tcp_stream.wrap_err("Error getting TCP stream in storeSCP")?;
        info!(
            "New tcp connection from: {}",
            stream
                .peer_addr()
                .wrap_err("Error getting peer adress from the TCP stream")?
        );

        let mut association = options
            .establish(stream)
            .wrap_err("Error establishing the association")?;

        info!(
            "New dicom association from {}",
            association.client_ae_title()
        );

        debug!(
            "Presentation contexts: {:?}",
            association.presentation_contexts()
        );

        loop {
            match association.receive() {
                Ok(mut pdu) => {
                    debug!("scu ----> scp: {}", pdu.short_description()); // TODO: relevance ?
                    match pdu {
                        Pdu::PData { ref mut data } => {
                            if data.is_empty() {
                                debug!("Ignoring empty PData PDU");
                                continue;
                            }
                            if data[0].value_type == PDataValueType::Data && !data[0].is_last {
                                instance_buffer.append(&mut data[0].data);
                            } else if data[0].value_type == PDataValueType::Command
                                && data[0].is_last
                            {
                                // commands are always in implict VR LE
                                let ts =
                            dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                .erased();
                                let data_value = &data[0];
                                let v = &data_value.data;

                                let obj = InMemDicomObject::read_dataset_with_ts(v.as_slice(), &ts)
                                    .wrap_err("Failed to read incoming DICOM command")?;

                                let command_field = obj
                                    .element(tags::COMMAND_FIELD)
                                    .wrap_err("Missing Command Field")?
                                    .uint16()
                                    .wrap_err("Command Field is not an integer")?;

                                if command_field == 0x0030 {
                                    // Handle C-ECHO-RQ
                                    let cecho_response = create_cecho_response(msgid);
                                    let mut cecho_data = Vec::new();

                                    cecho_response
                                        .write_dataset_with_ts(&mut cecho_data, &ts)
                                        .wrap_err("Could not write C-ECHO response object")?;

                                    let pdu_response = Pdu::PData {
                                        data: vec![dicom_ul::pdu::PDataValue {
                                            presentation_context_id: data[0]
                                                .presentation_context_id,
                                            value_type: PDataValueType::Command,
                                            is_last: true,
                                            data: cecho_data,
                                        }],
                                    };
                                    association
                                        .send(&pdu_response)
                                        .wrap_err("Failed to send C-ECHO response object to SCU")?;
                                } else {
                                    msgid = obj
                                        .element(tags::MESSAGE_ID)
                                        .wrap_err("Missing Message ID")?
                                        .to_int()
                                        .wrap_err("Message ID is not an integer")?;

                                    sop_class_uid = obj
                                        .element(tags::AFFECTED_SOP_CLASS_UID)
                                        .wrap_err("Missing Affected SOP Class UID")?
                                        .to_str()
                                        .wrap_err("Could not retrieve Affected SOP Class UID")?
                                        .to_string();

                                    sop_instance_uid = obj
                                        .element(tags::AFFECTED_SOP_INSTANCE_UID)
                                        .wrap_err("Missing Affected SOP Instance UID")?
                                        .to_str()
                                        .wrap_err("Could not retrieve Affected SOP Instance UID")?
                                        .to_string();
                                }

                                instance_buffer.clear();
                            } else if data[0].value_type == PDataValueType::Data && data[0].is_last
                            {
                                instance_buffer.append(&mut data[0].data);

                                let presentation_context = association
                                    .presentation_contexts()
                                    .iter()
                                    .find(|pc| pc.id == data[0].presentation_context_id)
                                    .wrap_err("Missing presentation context")?;
                                let ts = &presentation_context.transfer_syntax;

                                let obj = InMemDicomObject::read_dataset_with_ts(
                                    instance_buffer.as_slice(),
                                    TransferSyntaxRegistry.get(ts).unwrap(),
                                )
                                .wrap_err("Failed to read DICOM data object")?;
                                let file_meta = FileMetaTableBuilder::new()
                                    .media_storage_sop_class_uid(
                                        obj.element(tags::SOP_CLASS_UID)
                                            .wrap_err("Missing SOP Class UID")?
                                            .to_str()
                                            .wrap_err("Could not retrieve SOP Class UID")?,
                                    )
                                    .media_storage_sop_instance_uid(
                                        obj.element(tags::SOP_INSTANCE_UID)
                                            .wrap_err("Missing SOP Instance UID")?
                                            .to_str()
                                            .wrap_err("Missing SOP Instance UID")?,
                                    )
                                    .transfer_syntax(ts)
                                    .build()
                                    .wrap_err("Failed to build DICOM meta file information")?;
                                let file_obj = obj.with_exact_meta(file_meta);

                                // write the files to the current directory with their SOPInstanceUID as filenames
                                let mut file_path =
                                    node.out_dir.clone().wrap_err("Could not get out dir")?;
                                file_path.push(
                                    sop_instance_uid.trim_end_matches('\0').to_string() + ".dcm",
                                );
                                file_obj
                                    .write_to_file(&file_path)
                                    .wrap_err("Could not save DICOM object to file")?;
                                info!("Stored {}", file_path.display());

                                // send C-STORE-RSP object
                                // commands are always in implict VR LE
                                let ts =
                                dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                    .erased();

                                let obj = create_cstore_response(
                                    msgid,
                                    &sop_class_uid,
                                    &sop_instance_uid,
                                );

                                let mut obj_data = Vec::new();

                                obj.write_dataset_with_ts(&mut obj_data, &ts)
                                    .wrap_err("Could not write response object")?;

                                let pdu_response = Pdu::PData {
                                    data: vec![dicom_ul::pdu::PDataValue {
                                        presentation_context_id: data[0].presentation_context_id,
                                        value_type: PDataValueType::Command,
                                        is_last: true,
                                        data: obj_data,
                                    }],
                                };
                                association
                                    .send(&pdu_response)
                                    .wrap_err("Failed to send response object to SCU")?;
                            }
                        }
                        Pdu::ReleaseRQ => {
                            buffer.clear();
                            association
                                .send(&Pdu::ReleaseRP)
                                .wrap_err("Error sending release response")?;
                            info!(
                                "Released association with {}",
                                association.client_ae_title()
                            );
                        }
                        _ => {} // TODO: handle the other PDUs
                    }
                }
                Err(err @ dicom_ul::association::server::Error::Receive { .. }) => {
                    debug!(
                        "Dicom association server error while receiving data {}",
                        err
                    );
                    break;
                }
                Err(err) => {
                    warn!("Unexpected error: {}", err);
                    break;
                }
            }
        }
        info!("Dropping connection with {}", association.client_ae_title());
    }

    Ok(())
}

fn create_cstore_response(
    message_id: u16,
    sop_class_uid: &str,
    sop_instance_uid: &str,
) -> InMemDicomObject<StandardDataDictionary> {
    let elements = [
        DataElement::new(
            tags::COMMAND_GROUP_LENGTH,
            VR::UL,
            PrimitiveValue::from(
                8 + sop_class_uid.len() as i32
                    + 8
                    + 2
                    + 8
                    + 2
                    + 8
                    + 2
                    + 8
                    + 2
                    + sop_instance_uid.len() as i32,
            ),
        ),
        DataElement::new(
            tags::AFFECTED_SOP_CLASS_UID,
            VR::UI,
            dicom_value!(Str, sop_class_uid),
        ),
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x8001])),
        DataElement::new(
            tags::MESSAGE_ID_BEING_RESPONDED_TO,
            VR::US,
            dicom_value!(U16, [message_id]),
        ),
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0101]),
        ),
        DataElement::new(tags::STATUS, VR::US, dicom_value!(U16, [0x0000])),
        DataElement::new(
            tags::AFFECTED_SOP_INSTANCE_UID,
            VR::UI,
            dicom_value!(Str, sop_instance_uid),
        ),
    ];

    InMemDicomObject::from_element_iter(elements.iter().cloned())
}

fn create_cecho_response(message_id: u16) -> InMemDicomObject<StandardDataDictionary> {
    let elements = [
        DataElement::new(
            tags::COMMAND_GROUP_LENGTH,
            VR::UL,
            PrimitiveValue::from(8 + 8 + 2 + 8 + 2 + 8 + 2),
        ),
        DataElement::new(tags::COMMAND_FIELD, VR::US, dicom_value!(U16, [0x8030])),
        DataElement::new(
            tags::MESSAGE_ID_BEING_RESPONDED_TO,
            VR::US,
            dicom_value!(U16, [message_id]),
        ),
        DataElement::new(
            tags::COMMAND_DATA_SET_TYPE,
            VR::US,
            dicom_value!(U16, [0x0101]),
        ),
        DataElement::new(tags::STATUS, VR::US, dicom_value!(U16, [0x0000])),
    ];

    InMemDicomObject::from_element_iter(elements.iter().cloned())
}
