use can::*;

#[test]
fn test_signal_width_nonzero() {
    let try_sig = |width| -> Result<_, CANConstructionError> {
        CANSignal::builder().name("testSignal").width(width).build()
    };

    assert!(matches!(
        try_sig(0),
        Err(CANConstructionError::SignalHasZeroWidth(..))
    ));

    assert!(matches!(try_sig(1), Ok(..)));
}

#[test]
fn test_signal_width_inference() {
    let base_sig = || CANSignal::builder().name("testSignal");

    // nothing given except name
    assert!(matches!(
        base_sig().infer_width(),
        Err(CANConstructionError::SignalWidthInferenceFailed(..))
    ));

    // width already specified
    assert!(matches!(base_sig().width(1).infer_width(), Ok(..)));

    // width already specified, strict
    assert!(matches!(
        base_sig().width(1).infer_width_strict(),
        Err(CANConstructionError::SignalWidthAlreadySpecified(..))
    ));
}

#[test]
fn test_signal_width_nonexistent() {
    assert!(matches!(
        CANSignal::builder().name("testSignal").width(1).build(),
        Ok(..)
    ));

    assert!(matches!(
        CANSignal::builder().name("testSignal").build(),
        Err(CANConstructionError::UninitializedFieldError(s)) if s == "width"
    ));
}
