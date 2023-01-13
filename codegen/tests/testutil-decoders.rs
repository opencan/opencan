use anyhow::Result;
use indoc::formatdoc;
use testutil::decoders::*;

#[test]
fn test_decode_with_trait() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 4
    "};
    let net = opencan_compose::compose_entry_str(&desc)?;
    let decoder = CodegenDecoder::new(&net, "TEST")?;

    let v = decoder.decode_message("TEST_TestMessage", &[0xFA])?;

    assert!(v.len() == 1); // one signal

    let (sig, val) = &v[0];
    assert_eq!(sig, "TEST_testSignal");
    assert_eq!(*val, SignalValue::U8(0xA));

    Ok(())
}

#[test]
fn test_decode_with_trait_cantools() -> Result<()> {
    let desc = formatdoc! {"
        nodes:
            TEST:
                messages:
                    TestMessage:
                        id: 0x10
                        signals:
                            testSignal:
                                width: 4
    "};
    let net = opencan_compose::compose_entry_str(&desc)?;
    let decoder = CantoolsDecoder::new(&net)?;

    let v = decoder.decode_message("TEST_TestMessage", &[0xFA])?;

    assert!(v.len() == 1); // one signal

    let (sig, val) = &v[0];
    assert_eq!(sig, "TEST_testSignal");
    assert_eq!(*val, SignalValue::U8(0xA));

    Ok(())
}
