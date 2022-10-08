use std::collections::HashMap;
use std::ops::Index;

use crate::signal::*;

#[derive(Clone)]
pub struct CANMessageDesc {
    pub name: String,
    pub id: u32,

    pub signals: Vec<CANSignal>,
}

pub struct CANMessage {
    pub name: String,
    pub id: u32,

    pub signals: Vec<CANSignal>,
    pub sig_map: HashMap<String, usize>,
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

impl std::fmt::Display for CANMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        indoc::writedoc!(
            f,
            "
            Message `{}`:
              -> id: {}",
            self.name,
            self.id
        )
    }
}
