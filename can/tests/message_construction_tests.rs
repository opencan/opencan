use can::*;

#[test]
fn test_message_name_chars() {
    let try_msg = |name: &str| -> Result<_, CANConstructionError> {
        let desc = CANMessageDesc {
            name: name.into(),
            id: 0x10,
            cycletime_ms: None,
            signals: vec![],
        };

        CANMessage::new(desc)
    };

    // Invalid characters
    let invalid_names = ["test!", "!!!", "test.", ".test", "."];
    for name in invalid_names {
        assert!(matches!(
            try_msg(name),
            Err(CANConstructionError::MessageNameInvalidChar(..))
        ));
    }

    // Valid names
    let valid_names = ["test", "0", "_test_", "_", "___", "THING1_THING2"];
    for name in valid_names {
        assert!(matches!(try_msg(name), Ok(_)));
    }

    // Empty name
    assert!(matches!(
        try_msg(""),
        Err(CANConstructionError::MessageNameEmpty)
    ));
}
