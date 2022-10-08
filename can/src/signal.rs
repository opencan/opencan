use crate::value::*;

#[derive(Clone)]
pub struct CANSignal {
    pub offset: i32,
    pub name: String,

    pub value_type: CANValueType,
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

impl std::fmt::Display for CANSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        indoc::writedoc!(
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
