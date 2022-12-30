use crate::{CANMessage, CANNetwork, CANSignalWithPosition};

pub trait TranslationLayer {
    fn dump_network(net: &CANNetwork) -> String;
    fn dump_message(msg: &CANMessage) -> String;
    fn dump_signal(sig: &CANSignalWithPosition) -> String;
}
