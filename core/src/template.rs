use std::collections::HashMap;

use crate::*;

/// A template of a CAN message.
// newtyped to prevent misuse (e.g. adding more signals)
pub struct CANMessageTemplate(CANMessageTemplateBuilder);

impl CANMessageTemplate {
    pub(crate) fn instance(
        &self,
        name: String,
        id: u32,
        cycletime: Option<u32>,
        signal_prefix: String,
    ) -> Result<CANMessage, CANConstructionError> {
        let mut msg = self
            .0
            .msg_builder
            .clone()
            .origin_template(&self.0.name)
            .name(name)
            .id(id);

        if let Some(c) = cycletime {
            // todo check if we're overriding cycletime?
            msg = msg.cycletime(c);
        }

        let mut msg = msg.build()?;

        // go through the signals and append the signal prefix everywhere, rebuild the sig_map
        // yes, this is kind of ugly.
        let mut sig_map = HashMap::new();
        for (i, sigbit) in msg.signals.iter_mut().enumerate() {
            sigbit.sig.name = format!("{signal_prefix}{}", sigbit.sig.name);
            sig_map.insert(sigbit.sig.name.clone(), i);
        }

        msg.sig_map = sig_map;

        Ok(msg)
    }
}

pub struct CANMessageTemplateBuilder {
    name: String,
    msg_builder: CANMessageBuilder,
}

impl CANMessageTemplateBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            msg_builder: CANMessage::builder()
                .name("__template") // placeholder name
                .id(0), // placeholder id
        }
    }

    pub fn build(self) -> CANMessageTemplate {
        CANMessageTemplate(self)
    }

    pub fn cycletime(mut self, cycletime: u32) -> Self {
        self.msg_builder = self.msg_builder.cycletime(cycletime);

        self
    }

    pub fn add_signal(mut self, sig: CANSignal) -> Result<Self, CANConstructionError> {
        self.msg_builder = self.msg_builder.add_signal(sig)?;

        Ok(self)
    }

    pub fn add_signal_fixed(
        mut self,
        bit: u32,
        sig: CANSignal,
    ) -> Result<Self, CANConstructionError> {
        self.msg_builder = self.msg_builder.add_signal_fixed(bit, sig)?;

        Ok(self)
    }

    pub fn add_signals(
        mut self,
        sigs: impl IntoIterator<Item = CANSignal>,
    ) -> Result<Self, CANConstructionError> {
        self.msg_builder = self.msg_builder.add_signals(sigs)?;

        Ok(self)
    }

    pub fn add_signals_fixed(
        mut self,
        sigs: impl IntoIterator<Item = (u32, CANSignal)>,
    ) -> Result<Self, CANConstructionError> {
        self.msg_builder = self.msg_builder.add_signals_fixed(sigs)?;

        Ok(self)
    }
}
