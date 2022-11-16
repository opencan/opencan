use can::*;

#[test]
fn test_signal_lookup() {
    let make_msg = |sigs: Vec<CANSignal>| -> CANMessage {
        CANMessage::builder()
            .name("TestMessage")
            .id(0x10)
            .cycletime_ms(None)
            .add_signals(sigs)
            .unwrap()
            .build()
            .unwrap()
    };

    let mut bit = 0;
    let mut test_signal = |name: &str| -> CANSignal {
        let s = CANSignal::builder()
            .name(name)
            .width(1)
            .build()
            .unwrap();

        bit += 1;
        s
    };

    let msg = make_msg(vec![]);
    assert!(matches!(msg.get_sig("testSigA"), None));

    let msg = make_msg(vec![test_signal("testSigA")]);
    assert!(matches!(msg["testSigA"].name.as_str(), "testSigA"));

    let msg = make_msg(vec![test_signal("testSigA"), test_signal("testSigB")]);
    assert!(matches!(msg["testSigA"].name.as_str(), "testSigA"));
    assert!(matches!(msg["testSigB"].name.as_str(), "testSigB"));

    let msg = make_msg(vec![
        test_signal("testSigA"),
        test_signal("testSigB"),
        test_signal("testSigC"),
    ]);
    assert!(matches!(msg["testSigA"].name.as_str(), "testSigA"));
    assert!(matches!(msg["testSigB"].name.as_str(), "testSigB"));
    assert!(matches!(msg["testSigC"].name.as_str(), "testSigC"));
    assert!(matches!(msg.get_sig("testSigD"), None));
}
