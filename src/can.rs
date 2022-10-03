use std::{collections::HashMap, ops::Index};

mod can_print;

#[derive(Clone)]
pub struct CANValueTypeInteger {
    pub length: i32,
    pub signed: bool,
}

#[derive(Clone)]
pub enum CANValueType {
    Integer(CANValueTypeInteger),
}

#[derive(Clone)]
pub struct CANSignal {
    pub offset: i32,
    pub name: String,

    pub value_type: CANValueType,
}

#[derive(Clone)]
pub struct CANMessageDesc {
    pub name: String,
    pub id: u32,

    pub signals: Vec<CANSignal>,
}

pub struct CANMessage {
    name: String,
    id: u32,

    signals: Vec<CANSignal>,
    sig_map: HashMap<String, usize>,
}

pub struct CANNetwork {
    messages: Vec<CANMessage>,

    messages_by_name: HashMap<String, usize>,
    messages_by_id: HashMap<u32, usize>,
}

impl CANSignal {
    /*
    pub fn cantools_description(&self) -> String {
        formatdoc!(
            "
            cantools.database.can.Signal(name = '{name}',
                start = {offset},
                length = {length},
                is_signed = {signed})",
            name = self.name,
            offset = self.offset,
            length = self.value_type.length,
            signed = python_capital_bool(self.value_type.signed),
        )
    }
    */
}

impl CANMessage {
    pub fn get_sig(&self, name: &str) -> Option<&CANSignal> {
        let idx = self.sig_map.get(name)?;

        // unwrap here, as signals really should have the signal if sig_map does
        Some(self.signals.get(*idx).unwrap())
    }
}

// Easy indexing of msg["signal"]. Panics if signal absent.
// (no, this can't be an Option, the Index trait doesn't allow it)
impl Index<&str> for CANMessage {
    type Output = CANSignal;

    fn index(&self, index: &str) -> &Self::Output {
        return self.get_sig(index).unwrap();
    }
}

impl Default for CANNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum CANConstructionError {
    SignalSpecifiedMultipleTimes(String),
    MessageNameAlreadyExists(String),
    MessageIdAlreadyExists(u32),
}

impl CANNetwork {
    pub fn new() -> CANNetwork {
        CANNetwork {
            messages: Vec::new(),

            messages_by_name: HashMap::new(),
            messages_by_id: HashMap::new(),
        }
    }

    pub fn message_by_name(&self, name: &str) -> Option<&CANMessage> {
        let idx = self.messages_by_name.get(name)?;
        Some(self.messages.get(*idx).unwrap())
    }

    pub fn message_by_id(&self, id: &u32) -> Option<&CANMessage> {
        let idx = self.messages_by_id.get(id)?;
        Some(self.messages.get(*idx).unwrap())
    }

    pub fn add_msg(&mut self, msg: CANMessageDesc) -> Result<(), CANConstructionError> {
        // do signals
        let mut sigs = Vec::new();
        let mut sig_map = HashMap::new();

        for sig in msg.signals {
            if sig_map.get(&sig.name).is_some() {
                return Err(CANConstructionError::SignalSpecifiedMultipleTimes(sig.name));
            }

            sig_map.insert(sig.name.clone(), sigs.len());
            sigs.push(sig);
        }

        // now do message
        if self.messages_by_name.get(&msg.name).is_some() {
            return Err(CANConstructionError::MessageNameAlreadyExists(msg.name));
        }

        if self.messages_by_id.get(&msg.id).is_some() {
            return Err(CANConstructionError::MessageIdAlreadyExists(msg.id));
        }

        let msg_idx = self.messages.len();

        self.messages_by_name.insert(msg.name.clone(), msg_idx);
        self.messages_by_id.insert(msg.id, msg_idx); // check dup again?

        self.messages.push(CANMessage {
            name: msg.name,
            id: msg.id,
            signals: sigs,
            sig_map,
        });

        Ok(())
    }
}
