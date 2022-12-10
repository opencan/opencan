use can::*;

#[test]
fn test_message_name_chars() {
    let try_msg = |name: &str| -> Result<_, CANConstructionError> {
        CANMessage::builder()
            .name(name)
            .id(0x10)
            .cycletime_ms(None)
            .build()
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

#[test]
//signal name does not repeat ([`SignalNameAlreadyExists`][CANConstructionError::SignalNameAlreadyExists])
fn test_repeated_sig_name() {
    let sig1 = CANSignal::builder()
        .name("testsig")
        .width(1)
        .build()
        .unwrap();
    let sig2 = CANSignal::builder()
        .name("testsig")
        .width(1)
        .build()
        .unwrap();
    let sig3 = CANSignal::builder()
        .name("testsig")
        .width(1)
        .build()
        .unwrap();

    CANMessage::builder()
        .name("TestMessage")
        .id(0x10)
        .add_signal(sig1)
        .expect("Expected CANMessageBuilder");

    assert!(matches!(
        CANMessage::builder()
            .name("TestMessage")
            .id(0x10)
            .add_signal(sig2)
            .unwrap()
            .add_signal(sig3),
        Err(CANConstructionError::SignalNameAlreadyExists(..))
    ));
}

///  - signals are specified in order([`MessageSignalsOutOfOrder`][CANConstructionError::MessageSignalsOutOfOrder])
//ok but CANSignal doesn't even have an option for startbit ??? in its struct so what ??
#[test]
fn test_sig_specified_in_order() {
    //try to make this into a closure or a function to make it easier
    let sig1 = CANSignal::builder()
        .name("sig1")
        .width(1)
        .build()
        .unwrap();
    let sig2 = CANSignal::builder()
        .name("sig2")
        .width(1)
        .build()
        .unwrap();
    let sigs = vec![(5, sig1), (0, sig2)];
    assert!(matches!(
        CANMessage::builder()
            .name("TestMessage")
            .id(0x10)
            .add_signals_fixed(sigs),
        Err(CANConstructionError::MessageSignalsOutOfOrder(..))
    ));
}
