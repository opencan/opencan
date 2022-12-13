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
    pub offset: Option<f64>,

    #[builder(default)]
    pub scale: Option<f64>,

    #[builder(setter(custom), field(type = "bimap::BiMap<String, u64>"))]
    pub enumerated_values: bimap::BiMap<String, u64>,

    #[serde(skip)]
    #[builder(setter(custom), field(type = "Option<u64>"))]
    _highest_enumerated_value: Option<u64>,
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
        let val = self._highest_enumerated_value.map_or(0, |v| v + 1);

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

        // set highest to max of existing and new
        self._highest_enumerated_value =
            Some(self._highest_enumerated_value.map_or(val, |h| h.max(val)));

        assert!(!self.enumerated_values.insert(name, val).did_overwrite());

        Ok(self)
    }

    fn min_width_for_enumerated_values(&self) -> u32 {
        if let Some(v) = self._highest_enumerated_value {
            // this is ilog2() - .ilog2() stable in 1.67
            u64::BITS - 1 - (v + 1).next_power_of_two().leading_zeros()
        } else {
            0
        }
    }
}

impl CANSignal {
    pub fn builder() -> CANSignalBuilder {
        CANSignalBuilder::default()
    }

    pub fn decode_string(&self, raw: u64) -> String {
        let mut out = String::new();

        let enval = self.enumerated_values.iter().find(|&e| *e.1 == raw);

        if self.scale.is_some() || self.offset.is_some() {
            let expanded = (raw as f64 * self.scale.unwrap_or(1.)) + self.offset.unwrap_or(0.);
            out += &format!("{}: {}", self.name, expanded);
        } else {
            out += &format!("{}: {}", self.name, raw);
        }

        if let Some(e) = enval {
            out += &format!(" ('{}')", e.0);
        }

        out
    }
}
