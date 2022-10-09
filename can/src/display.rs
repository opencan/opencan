use std::fmt;
use std::fmt::{Display, Formatter};

use indoc::writedoc;

use crate::message::*;
use crate::signal::*;
use crate::value::*;

impl Display for CANValueTypeInteger {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (if self.signed { "s" } else { "u" }),
            self.length
        )
    }
}

impl Display for CANValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CANValueType::Integer(s) => s.fmt(f),
        }
    }
}

impl Display for CANSignal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            Signal `{}`:
              -> offset: {},
              -> type: {}",
            self.name,
            self.offset,
            self.value_type,
        )
    }
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
    pub fn print_human(&self) {
        println!("{}\n", self);
        println!("**** Signals: ****\n");
        self.print_signals_human();
        println!("******************");
    }

    pub fn print_signals_human(&self) {
        for sig in &self.signals {
            println!("{}\n", sig);
        }
    }
}

impl Display for CANMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            Message `{}`:
              -> id: {}",
            self.name,
            self.id
        )
    }
}
