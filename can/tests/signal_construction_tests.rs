use can::*;

#[test]
fn test_signal_width_nonzero() {
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

#[test]
fn test_signal_width_inference() {
    let base_sig = || -> CANSignalBuilder {
        CANSignal::builder()
            .name("testSignal".into())
    };

    // nothing given except name
    assert!(matches!(
        base_sig().infer_width(),
        Err(CANConstructionError::SignalWidthInferenceFailed(..))
    ));
}
