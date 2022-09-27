use indoc::formatdoc;

fn python_capital_bool(b: bool) -> String {
    (if b { "True" } else { "False" }).to_string()
}

#[derive(Clone)]
pub struct CANValueType {
    pub length: i32,
    pub signed: bool,
}

#[derive(Clone)]
pub struct CANSignal {
    pub offset: i32,
    pub name: String,

    pub value_type: CANValueType,
}

#[derive(Clone)]
pub struct CANMessage {
    pub name: String,

    pub signals: Vec<CANSignal>,
}

pub struct CANNetwork {
    pub messages: Vec<CANMessage>,
}

impl CANValueType {
    pub fn _human_description(&self) -> String {
        format!("{}{}", (if self.signed { "s" } else { "u" }), self.length)
    }
}

impl CANSignal {
    pub fn _human_description(&self) -> String {
        formatdoc!("
            Signal `{}`:
              -> offset: {},
              -> type: {}",
            self.name,
            self.offset,
            self.value_type._human_description()
        )
    }

    pub fn cantools_description(&self) -> String {
        formatdoc!("
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

impl CANNetwork {
    pub fn add_msg(&mut self, msg: CANMessage) {
        self.messages.push(msg);
    }
}