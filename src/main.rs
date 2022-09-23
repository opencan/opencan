mod can;

fn main() {
    println!("Hello from protobrain.");
    println!("----------------------");

    let s = can::CANSignal {
        offset: 0,
        name: "VCFRONT_driverIsLeaving".to_string(),
        value_type: can::CANValueType {
            length: 5,
            signed: false,
        },
    };

    let mut m = can::CANMessage {
        name: "VCFRONT_Occupancy".to_string(),
        signals: Vec::new(),
    };

    m.signals.push(s.clone());
    m.signals.push(s);

    println!("Have message with name: `{}`", m.name);

    for sig in m.signals {
        println!("Have signal.\n{}", sig.human_description());

        println!("CANTOOLS:\n{}", sig.cantools_description());
    }
}
