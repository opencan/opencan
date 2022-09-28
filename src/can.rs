use std::collections::HashMap;

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

pub struct CANSignalHandle {
    idx: usize,
}

pub struct CANMessageHandle {
    idx: usize,
}

pub struct CANMessageDesc {
    pub name: String,
    pub id: u32,

    pub signals: Vec<CANSignal>,
}

struct CANMessage {
    name: String,
    id: u32,

    signals: Vec<CANSignalHandle>,
}

pub struct CANNetwork {
    messages: Vec<CANMessage>,
    signals: Vec<CANSignal>,

    messages_by_name: HashMap<String, usize>,
    messages_by_id: HashMap<u32, usize>,

    signals_by_name: HashMap<String, usize>,
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
            signals: Vec::new(),

            messages_by_name: HashMap::new(),
            messages_by_id: HashMap::new(),
            signals_by_name: HashMap::new(),
        }
    }

    fn demand_sig(&self, sig: &CANSignalHandle) -> &CANSignal {
        self.signals.get(sig.idx).expect("Invalid signal handle.")
    }

    fn demand_msg(&self, msg: &CANMessageHandle) -> &CANMessage {
        self.messages.get(msg.idx).expect("Invalid message handle.")
    }

    pub fn message_by_name(&self, name: &str) -> Option<CANMessageHandle> {
        self.messages_by_name.get(name).map(|i| CANMessageHandle { idx: *i })
    }

    pub fn add_msg(&mut self, msg: CANMessageDesc) -> Option<CANMessageHandle> {
        // do signals
        let mut sigs = Vec::new();

        for sig in msg.signals {
            if let Some(_exist) = self.messages_by_name.get(&sig.name) {
                eprintln!(
                    "Error: signal with name `{}` conflicts with message of same name.",
                    &sig.name
                );
                return None;
            }

            if let Some(_exist) = self.signals_by_name.get(&sig.name) {
                // the below error message is misleading if there are conflicting signal names
                // within THIS message description.
                // TODO: make smarter
                eprintln!(
                    "Error: signal with name `{}` already exists in network.",
                    &sig.name
                );
                return None;
            }

            let sig_idx = self.signals.len();
            sigs.push(CANSignalHandle { idx: sig_idx });

            self.signals_by_name.insert(sig.name.clone(), sig_idx);
            self.signals.push(sig);
        }

        // now do message
        if let Some(_exist) = self.messages_by_name.get(&msg.name) {
            eprintln!(
                "Error: message with name `{}` already exists in network.",
                &msg.name
            );
            return None;
        }

        if let Some(_exist) = self.signals_by_name.get(&msg.name) {
            eprintln!(
                "Error: message with name `{}` conflicts with existing signal of same name.",
                &msg.name
            );
            return None;
        }

        if let Some(_exist) = self.messages_by_id.get(&msg.id) {
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
        });

        Some(CANMessageHandle { idx: msg_idx })
    }

    pub fn print_msg_human(&self, handle: &CANMessageHandle) {
        let msg = self.demand_msg(handle);

        println!("{}\n", msg.human_description());
        println!("**** Signals: ****\n");
        self.print_signals_human(handle);
        println!("******************");
    }

    pub fn print_signals_human(&self, msg: &CANMessageHandle) {
        let msg = self.demand_msg(msg);

        for sig in &msg.signals {
            let sig = self.demand_sig(sig);

            println!("{}\n", sig.human_description());
        }
    }
}
