use indoc::writedoc;

use crate::can::*;

// --- fmt::Display --- //
impl std::fmt::Display for CANValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CANValueType::Integer(s) => s.fmt(f),
        }
    }
}

impl std::fmt::Display for CANValueTypeInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (if self.signed { "s" } else { "u" }),
            self.length
        )
    }
}

impl std::fmt::Display for CANSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for CANMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for CANConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CANConstructionError::SignalSpecifiedMultipleTimes(s) => {
                write!(f, "Signal with name {s} specified multiple times.")
            }
            CANConstructionError::MessageNameAlreadyExists(n) => {
                write!(f, "Message with name `{n}` already exists in network.")
            }
            CANConstructionError::MessageIdAlreadyExists(i) => {
                write!(f, "Message with id 0x{:x} already exists in network.", i)
            }
        }
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
