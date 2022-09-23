#[derive(Clone)]
struct CANValueType {
    length: i32,
}

#[derive(Clone)]
struct CANSignal {
    offset: i32,
    name: String,

    value_type: CANValueType,
}

impl CANSignal {
    fn human_description(&self) -> String {
        format!("Signal `{}`:\
               \n -> offset: {},\
               \n -> length: {}",
                self.name, self.offset, self.value_type.length)
    }
}

struct CANMessage {
    name: String,

    signals: Vec<CANSignal>,
}

fn main() {
    println!("Hello from protobrain.");
    println!("----------------------");

    let s = CANSignal {
        offset: 1,
        name: "VCFRONT_driverIsLeaving".to_string(),
        value_type: CANValueType { length: 1 },
    };

    let mut m = CANMessage {
        name: "VCFRONT_Occupancy".to_string(),
        signals: Vec::new(),
    };

    m.signals.push(s.clone());
    m.signals.push(s);

    println!("Have message with name: `{}`", m.name);

    for sig in m.signals {
        println!("Have signal.\n{}", sig.human_description());
    }
}
