use crate::response_builders::{create_cecho_response, create_cstore_response};
use crate::settings_handler::{load_settings, UserSettings};
use crate::utils::ABSTRACT_SYNTAXES;
use dicom::dictionary_std::tags;
use dicom::encoding::TransferSyntaxIndex;
use dicom::object::{FileMetaTableBuilder, InMemDicomObject};
use dicom::transfer_syntax::TransferSyntaxRegistry;
use dicom_ul::pdu::PDataValueType;
use dicom_ul::{self, Pdu};
use snafu::{OptionExt, ResultExt, Whatever};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber;

pub mod response_builders;
pub mod settings_handler;
pub mod utils;

fn run(scu_stream: TcpStream, args: &UserSettings) -> Result<(), Whatever> {
    let UserSettings {
        log_level,
        calling_ae_title,
        strict,
        uncompressed_only,
        max_pdu_length,
        out_dir,
        port: _,
    } = args;
    let log_level = *log_level;

    let mut buffer: Vec<u8> = Vec::with_capacity(*max_pdu_length as usize);
    let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut msgid = 1;
    let mut sop_class_uid = "".to_string();
    let mut sop_instance_uid = "".to_string();

    let mut options = dicom_ul::association::ServerAssociationOptions::new()
        .accept_any()
        .ae_title(calling_ae_title)
        .strict(*strict);

    if *uncompressed_only {
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

    let mut association = options
        .establish(scu_stream)
        .whatever_context("could not establish association")?;

    info!("New association from {}", association.client_ae_title());
    debug!(
        "> Presentation contexts: {:?}",
        association.presentation_contexts()
    );

    loop {
        match association.receive() {
            Ok(mut pdu) => {
                if log_level == Level::DEBUG {
                    debug!("scu ----> scp: {}", pdu.short_description());
                }
                match pdu {
                    Pdu::PData { ref mut data } => {
                        if data.is_empty() {
                            debug!("Ignoring empty PData PDU");
                            continue;
                        }
                        if data[0].value_type == PDataValueType::Data && !data[0].is_last {
                            instance_buffer.append(&mut data[0].data);
                        } else if data[0].value_type == PDataValueType::Command && data[0].is_last {
                            // commands are always in implict VR LE
                            let ts =
                                dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                    .erased();

                            let data_value = &data[0];

                            let v = &data_value.data;

                            let obj = InMemDicomObject::read_dataset_with_ts(v.as_slice(), &ts)
                                .whatever_context("failed to read incoming DICOM command")?;

                            let command_field = obj
                                .element(tags::COMMAND_FIELD)
                                .whatever_context("missing Command Field")?
                                .uint16()
                                .whatever_context("Command Field is not an integer")?;

                            if command_field == 0x0030 {
                                // Handle C-ECHO-RQ
                                let cecho_response = create_cecho_response(msgid);
                                let mut cecho_data = Vec::new();

                                cecho_response
                                    .write_dataset_with_ts(&mut cecho_data, &ts)
                                    .whatever_context("could not write C-ECHO response object")?;

                                let pdu_response = Pdu::PData {
                                    data: vec![dicom_ul::pdu::PDataValue {
                                        presentation_context_id: data[0].presentation_context_id,
                                        value_type: PDataValueType::Command,
                                        is_last: true,
                                        data: cecho_data,
                                    }],
                                };
                                association.send(&pdu_response).whatever_context(
                                    "failed to send C-ECHO response object to SCU",
                                )?;
                            } else {
                                msgid = obj
                                    .element(tags::MESSAGE_ID)
                                    .whatever_context("Missing Message ID")?
                                    .to_int()
                                    .whatever_context("Message ID is not an integer")?;

                                sop_class_uid = obj
                                    .element(tags::AFFECTED_SOP_CLASS_UID)
                                    .whatever_context("missing Affected SOP Class UID")?
                                    .to_str()
                                    .whatever_context("could not retrieve Affected SOP Class UID")?
                                    .to_string();

                                sop_instance_uid = obj
                                    .element(tags::AFFECTED_SOP_INSTANCE_UID)
                                    .whatever_context("missing Affected SOP Instance UID")?
                                    .to_str()
                                    .whatever_context(
                                        "could not retrieve Affected SOP Instance UID",
                                    )?
                                    .to_string();
                            }
                            instance_buffer.clear();
                        } else if data[0].value_type == PDataValueType::Data && data[0].is_last {
                            instance_buffer.append(&mut data[0].data);

                            let presentation_context = association
                                .presentation_contexts()
                                .iter()
                                .find(|pc| pc.id == data[0].presentation_context_id)
                                .whatever_context("missing presentation context")?;
                            let ts = &presentation_context.transfer_syntax;

                            let obj = InMemDicomObject::read_dataset_with_ts(
                                instance_buffer.as_slice(),
                                TransferSyntaxRegistry.get(ts).unwrap(),
                            )
                            .whatever_context("failed to read DICOM data object")?;
                            let file_meta = FileMetaTableBuilder::new()
                                .media_storage_sop_class_uid(
                                    obj.element(tags::SOP_CLASS_UID)
                                        .whatever_context("missing SOP Class UID")?
                                        .to_str()
                                        .whatever_context("could not retrieve SOP Class UID")?,
                                )
                                .media_storage_sop_instance_uid(
                                    obj.element(tags::SOP_INSTANCE_UID)
                                        .whatever_context("missing SOP Instance UID")?
                                        .to_str()
                                        .whatever_context("missing SOP Instance UID")?,
                                )
                                .transfer_syntax(ts)
                                .build()
                                .whatever_context("failed to build DICOM meta file information")?;
                            let file_obj = obj.with_exact_meta(file_meta);

                            // write the files to the current directory with their SOPInstanceUID as filenames
                            let mut file_path = out_dir.clone();
                            file_path
                                .push(sop_instance_uid.trim_end_matches('\0').to_string() + ".dcm");
                            file_obj
                                .write_to_file(&file_path)
                                .whatever_context("could not save DICOM object to file")?;
                            info!("Stored {}", file_path.display());

                            // send C-STORE-RSP object
                            // commands are always in implict VR LE
                            let ts =
                                dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                    .erased();

                            let obj =
                                create_cstore_response(msgid, &sop_class_uid, &sop_instance_uid);

                            let mut obj_data = Vec::new();

                            obj.write_dataset_with_ts(&mut obj_data, &ts)
                                .whatever_context("could not write response object")?;

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
                                .whatever_context("failed to send response object to SCU")?;
                        }
                    }
                    Pdu::ReleaseRQ => {
                        buffer.clear();
                        association.send(&Pdu::ReleaseRP).unwrap_or_else(|e| {
                            warn!(
                                "Failed to send association release message to SCU: {}",
                                snafu::Report::from_error(e)
                            );
                        });
                        info!(
                            "Released association with {}",
                            association.client_ae_title()
                        );
                    }
                    _ => {}
                }
            }
            Err(err @ dicom_ul::association::server::Error::Receive { .. }) => {
                debug!("{}", err);
                break;
            }
            Err(err) => {
                warn!("Unexpected error: {}", snafu::Report::from_error(err));
                break;
            }
        }
    }
    info!("Dropping connection with {}", association.client_ae_title());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: UserSettings;

    match load_settings() {
        Ok(user_settings) => {
            settings = user_settings;
        }
        Err(e) => {
            error!("Failed to load UserSettings: {}", e);
            std::process::exit(1);
        }
    }

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(settings.log_level)
            .finish(),
    )
    .unwrap_or_else(|e| {
        eprintln!(
            "Could not set up global logger: {}",
            snafu::Report::from_error(e)
        );
    });

    let listen_addr = SocketAddrV4::new(Ipv4Addr::from(0), settings.port);
    let listener = TcpListener::bind(listen_addr)?;
    info!(
        "{} listening on: tcp://{}",
        &settings.calling_ae_title, listen_addr
    );

    for stream in listener.incoming() {
        match stream {
            Ok(scu_stream) => {
                if let Err(e) = run(scu_stream, &settings) {
                    error!("{}", snafu::Report::from_error(e));
                }
            }
            Err(e) => {
                error!("{}", snafu::Report::from_error(e));
            }
        }
    }

    Ok(())
}
