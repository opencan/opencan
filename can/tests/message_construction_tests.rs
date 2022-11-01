use can::*;

#[test]
fn test_message_name_chars() {
    let invalid_names = ["test!", "!!!", "test.", ".test", "."];

    for name in invalid_names {
        let desc = CANMessageDesc {
            name: name.into(),
            id: 0x10,
            signals: vec![],
        };

        assert!(matches!(
            CANMessage::new(desc),
            Err(CANConstructionError::MessageNameInvalidChar(_))
        ))
    }

    let valid_names = ["test", "0", "_test_", "_", "___", "THING1_THING2"];

    for name in valid_names {
        let desc = CANMessageDesc {
            name: name.into(),
            id: 0x10,
            signals: vec![],
        };

        assert!(matches!(CANMessage::new(desc), Ok(_)))
    }
}
