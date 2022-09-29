use std::{collections::HashMap, ops::Index};

use indoc::formatdoc;

fn python_capital_bool(b: bool) -> String {
    (if b { "True" } else { "False" }).to_string()
}

#[derive(Clone)]
pub struct CANValueType {
    pub length: i32,
    pub signed: bool,
}

pub struct CANSignal {
    pub offset: i32,
    pub name: String,

    pub value_type: CANValueType,
}

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

impl CANValueType {
    pub fn _human_description(&self) -> String {
        format!("{}{}", (if self.signed { "s" } else { "u" }), self.length)
    }
}

impl CANSignal {
    pub fn human_description(&self) -> String {
        formatdoc!(
            "
            Signal `{}`:
              -> offset: {},
              -> type: {}",
            self.name,
            self.offset,
            self.value_type._human_description()
        )
    }

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
}

impl CANMessage {
    pub fn human_description(&self) -> String {
        formatdoc!(
            "
            Message `{}`:
              -> id: {}",
            self.name,
            self.id
        )
    }

    pub fn print_human(&self) {
        println!("{}\n", self.human_description());
        println!("**** Signals: ****\n");
        self.print_signals_human();
        println!("******************");
    }

    pub fn print_signals_human(&self) {
        for sig in &self.signals {
            println!("{}\n", sig.human_description());
        }
    }

    pub fn get_sig(&self, name: &str) -> Option<&CANSignal> {
        let idx = self.sig_map.get(name)?;

        self.signals.get(*idx)
    }
}

// Easy indexing of msg["signal"]. Panics if signal absent.
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
        self.messages.get(*idx)
    }

    pub fn add_msg(&mut self, msg: CANMessageDesc) -> Option<&CANMessage> {
        // do signals
        let mut sigs = Vec::new();
        let mut sig_map = HashMap::new();

        for sig in msg.signals {
            if sig_map.get(&sig.name).is_some() {
                eprintln!(
                    "Error: signal with name `{}` specified multiple times for message `{}`.",
                    &sig.name, &msg.name
                );
                return None;
            }

            sig_map.insert(sig.name.clone(), sigs.len());
            sigs.push(sig);
        }

        // now do message
        if self.messages_by_name.get(&msg.name).is_some() {
            eprintln!(
                "Error: message with name `{}` already exists in network.",
                &msg.name
            );
            return None;
        }

        if self.messages_by_id.get(&msg.id).is_some() {
            eprintln!(
                "Error: message with id `{}` already exists in network.",
                &msg.id
            );
            return None;
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

        Some(&self.messages[msg_idx])
    }
}
