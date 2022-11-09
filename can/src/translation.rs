use crate::{CANMessage, CANNetwork, CANSignal};

pub trait TranslationLayer {
    fn dump_network(net: &CANNetwork) -> String;
    fn dump_message(msg: &CANMessage) -> String;
    fn dump_signal(sig: &CANSignal) -> String;
}
