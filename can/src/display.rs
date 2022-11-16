use std::fmt;
use std::fmt::{Display, Formatter};

use indoc::writedoc;

use crate::message::*;
use crate::signal::*;

impl Display for CANSignal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            Signal `{}`:
              -> offset: {}",
            self.name,
            0
        )
    }
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
