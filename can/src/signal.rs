use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::*;

#[derive(Serialize, Deserialize, Clone, Builder)]
#[builder(build_fn(name = "__build", error = "CANConstructionError", private))]
#[builder(pattern = "owned")]
pub struct CANSignal {
    #[builder(setter(into))]
    pub name: String,

    pub width: u32,

    #[builder(default)]
    pub description: Option<String>,

    #[builder(default)]
    pub offset: Option<f32>,

    #[builder(default)]
    pub scale: Option<f32>,

    #[builder(setter(custom), field(type = "bimap::BiMap<String, u64>"))]
    pub enumerated_values: bimap::BiMap<String, u64>,

    #[serde(skip)]
    #[builder(setter(custom), field(type = "u64"))]
    _highest_enumerated_value: u64,
}

impl CANSignalBuilder {
    pub fn build(self) -> Result<CANSignal, CANConstructionError> {
        let s = self.__build()?;
        if s.width == 0 {
            return Err(CANConstructionError::SignalHasZeroWidth(s.name));
        }

        // check that the highest enumerated value can fit within the width of the signal

        Ok(s)
    }

    /// Infer the width of this signal based on given information.
    // TODO: Write and describe inference semantics.
    // TODO: Maybe this should just be implictly called when no width is specified?
    pub fn infer_width(mut self) -> Result<Self, CANConstructionError> {
        if self.width.is_some() {
            return Ok(self);
        }

        // choose the biggest minimum width we have
        let min_width = [self.min_width_for_enumerated_values()]
            .into_iter()
            .max()
            .unwrap();

        if min_width == 0 {
            Err(CANConstructionError::SignalWidthInferenceFailed(self.name))
        } else {
            self.width = Some(min_width);
            Ok(self)
        }
    }

    /// Infer the width of the signal, but return a SignalWidthAlreadySpecified
    /// error if the signal width was already specified.
    pub fn infer_width_strict(self) -> Result<Self, CANConstructionError> {
        if self.width.is_some() {
            return Err(CANConstructionError::SignalWidthAlreadySpecified(self.name));
        }

        self.infer_width()
    }

    pub fn add_enumerated_value_inferred(self, name: String) -> Result<Self, CANConstructionError> {
        let val = self._highest_enumerated_value + 1;
        self.add_enumerated_value(name, val)
    }

    pub fn add_enumerated_value(
        mut self,
        name: String,
        val: u64,
    ) -> Result<Self, CANConstructionError> {
        if let Some(&v) = self.enumerated_values.get_by_left(&name) {
            return Err(CANConstructionError::EnumeratedValueNameAlreadyExists(
                name, v,
            ));
        }

        if let Some(n) = self.enumerated_values.get_by_right(&val) {
            return Err(CANConstructionError::EnumeratedValueValueAlreadyNamed(
                n.clone(),
                val,
            ));
        }

        if val > self._highest_enumerated_value {
            self._highest_enumerated_value = val;
        }

        assert!(!self.enumerated_values.insert(name, val).did_overwrite());

        Ok(self)
    }

    fn min_width_for_enumerated_values(&self) -> u32 {
        if !self.enumerated_values.is_empty() {
            // this is (self._highest_enumerated_value.next_power_of_two).ilog2()
            // ilog2() should be stabilized in 1.66
            u64::BITS - 1 - self._highest_enumerated_value.next_power_of_two().leading_zeros()
        } else {
            0
        }
    }
}

impl CANSignal {
    pub fn builder() -> CANSignalBuilder {
        CANSignalBuilder::default()
    }
}
