use can::*;

#[test]
fn test_signal_width() {
    let try_sig = |width| -> Result<_, CANConstructionError> {
        CANSignal::builder()
            .name("testSignal".into())
            .start_bit(0)
            .width(width)
            .build()
    };

    assert!(matches!(
        try_sig(0),
        Err(CANConstructionError::SignalHasZeroWidth(..))
    ));

    assert!(matches!(try_sig(1), Ok(..)));
}
