use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::error::*;

#[derive(Serialize, Deserialize, Clone, TypedBuilder)]
#[builder(build_method(vis="", name=__build))]
pub struct CANSignal {
    pub name: String,
    pub start_bit: u32,
    pub width: u32,

    #[builder(default)]
    pub description: Option<String>,

    #[builder(default)]
    pub offset: Option<f32>,

    #[builder(default)]
    pub scale: Option<f32>,
}

// The signature looks scary, but we need this to implement our checked build()
// on CANSignalBuilder. You can get the signature from `cargo expand`.
#[allow(non_camel_case_types)]
impl<
        __scale: CANSignalBuilder_Optional<Option<f32>>,
        __offset: CANSignalBuilder_Optional<Option<f32>>,
        __description: CANSignalBuilder_Optional<Option<String>>,
    > CANSignalBuilder<((String,), (u32,), (u32,), __description, __offset, __scale)>
{
    pub fn build(self) -> Result<CANSignal, CANConstructionError> {
        let s = self.__build();
        if s.width == 0 {
            return Err(CANConstructionError::SignalHasZeroWidth(s.name));
        }

        Ok(s)
    }
}
