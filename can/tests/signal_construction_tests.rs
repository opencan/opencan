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
    let base_sig = || CANSignal::builder().name("testSignal".into());

    // nothing given except name
    assert!(matches!(
        base_sig().infer_width(),
        Err(CANConstructionError::SignalWidthInferenceFailed(..))
    ));

    // width already specified
    let ws = base_sig().width(1).infer_width();
    assert!(matches!(ws, Ok(..)));

    // width already specified, strict
    assert!(matches!(
        base_sig().width(1).infer_width_strict(),
        Err(CANConstructionError::SignalWidthAlreadySpecified(..))
    ));
}
