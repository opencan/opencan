use crate::CANNetwork;

pub mod cantools;
pub use cantools::*;

pub mod from_dbc;
pub use from_dbc::*;

/// Translation from `OpenCAN` to other formats (e.g. `dbc`).
pub trait TranslationFromOpencan {
    fn translate(net: &CANNetwork) -> String;
}

/// Translation from other formats (e.g. `dbc`) to OpenCAN.

pub trait TranslationToOpencan {
    fn import_network(input: String, net: &mut CANNetwork);
}
