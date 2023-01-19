use anyhow::Result;
use testutil::decoders::*;

const TEST_DESC: &str = "
nodes:
- TESTTX:
    messages:
    - TestMessage:
        id: 0x10
        signals:
        - testSignal:
            width: 4
- TESTRX:
    messages:
    rx:
      - TESTTX_TestMessage
";

#[test]
fn test_decode_with_trait() -> Result<()> {
    let net = opencan_compose::compose_str(TEST_DESC)?;
    let decoder = CodegenDecoder::new(&net, "TESTRX")?;

    let v = decoder.decode_message("TESTTX_TestMessage", &[0xFA])?;

    assert!(v.len() == 1); // one signal

    let (sig, val) = &v[0];
    assert_eq!(sig, "TESTTX_testSignal");
    assert_eq!(*val, SignalValue::U8(0xA));

    Ok(())
}

#[test]
fn test_decode_with_trait_cantools() -> Result<()> {
    let net = opencan_compose::compose_str(TEST_DESC)?;
    let decoder = CantoolsDecoder::new(&net)?;

    let v = decoder.decode_message("TESTTX_TestMessage", &[0xFA])?;

    assert!(v.len() == 1); // one signal

    let (sig, val) = &v[0];
    assert_eq!(sig, "TESTTX_testSignal");
    assert_eq!(*val, SignalValue::U8(0xA));

    Ok(())
}
