use crate::{CANMessage, CANNetwork, CANSignalWithPosition};

pub mod cantools;
pub use cantools::*;

/// Translation between `OpenCAN` and other formats (e.g. `cantools`).
pub trait TranslationLayer {
    fn dump_network(net: &CANNetwork) -> String;
    fn dump_message(msg: &CANMessage) -> String;
    fn dump_signal(sig: &CANSignalWithPosition) -> String;
}
