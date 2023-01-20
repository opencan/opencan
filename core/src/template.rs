use std::collections::HashMap;

use serde::Serialize;

use crate::*;

/// A template of a CAN message.
// newtyped to prevent misuse (e.g. adding more signals)
#[derive(Serialize, Debug)]
pub struct CANMessageTemplate(CANMessageTemplateBuilder);

impl CANMessageTemplate {
    pub fn instance(
        &self,
        name: &str,
        id: u32,
        cycletime: Option<u32>,
        signal_prefix: &str,
        tx_node: Option<&str>,
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

        if let Some(n) = tx_node {
            msg = msg.tx_node(n);
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

    pub fn name(&self) -> &str {
        &self.0.name
    }
}

#[derive(Serialize, Debug)]
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
