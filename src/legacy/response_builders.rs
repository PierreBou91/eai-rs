use dicom::{
    core::{DataElement, PrimitiveValue, VR},
    dicom_value,
    dictionary_std::tags,
    object::{InMemDicomObject, StandardDataDictionary},
};

pub fn create_cecho_response(message_id: u16) -> InMemDicomObject<StandardDataDictionary> {
    let mut obj = InMemDicomObject::new_empty();

    // group length
    obj.put(DataElement::new(
        tags::COMMAND_GROUP_LENGTH,
        VR::UL,
        PrimitiveValue::from(8 + 8 + 2 + 8 + 2 + 8 + 2),
    ));

    // command field
    obj.put(DataElement::new(
        tags::COMMAND_FIELD,
        VR::US,
        dicom_value!(U16, [0x8030]),
    ));

    // message ID being responded to
    obj.put(DataElement::new(
        tags::MESSAGE_ID_BEING_RESPONDED_TO,
        VR::US,
        dicom_value!(U16, [message_id]),
    ));

    // data set type
    obj.put(DataElement::new(
        tags::COMMAND_DATA_SET_TYPE,
        VR::US,
        dicom_value!(U16, [0x0101]),
    ));

    // status
    obj.put(DataElement::new(
        tags::STATUS,
        VR::US,
        dicom_value!(U16, [0x0000]),
    ));

    obj
}

pub fn create_cstore_response(
    message_id: u16,
    sop_class_uid: &str,
    sop_instance_uid: &str,
) -> InMemDicomObject<StandardDataDictionary> {
    let mut obj = InMemDicomObject::new_empty();

    // group length
    obj.put(DataElement::new(
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
    ));

    // service
    obj.put(DataElement::new(
        tags::AFFECTED_SOP_CLASS_UID,
        VR::UI,
        dicom_value!(Str, sop_class_uid),
    ));
    // command
    obj.put(DataElement::new(
        tags::COMMAND_FIELD,
        VR::US,
        dicom_value!(U16, [0x8001]),
    ));
    // message ID being responded to
    obj.put(DataElement::new(
        tags::MESSAGE_ID_BEING_RESPONDED_TO,
        VR::US,
        dicom_value!(U16, [message_id]),
    ));
    // data set type
    obj.put(DataElement::new(
        tags::COMMAND_DATA_SET_TYPE,
        VR::US,
        dicom_value!(U16, [0x0101]),
    ));
    // status https://dicom.nema.org/dicom/2013/output/chtml/part07/chapter_C.html
    obj.put(DataElement::new(
        tags::STATUS,
        VR::US,
        dicom_value!(U16, [0x0000]),
    ));
    // SOPInstanceUID
    obj.put(DataElement::new(
        tags::AFFECTED_SOP_INSTANCE_UID,
        VR::UI,
        dicom_value!(Str, sop_instance_uid),
    ));

    obj
}
