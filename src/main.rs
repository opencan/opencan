mod can;

fn main() {
    println!("Hello from protobrain.");
    println!("----------------------");

    let s = can::CANSignal {
        offset: 0,
        name: "VCFRONT_driverIsLeaving".to_string(),
        value_type: can::CANValueType {
            length: 5,
            signed: true,
        },
    };

    let mut m = can::CANMessage {
        name: "VCFRONT_Occupancy".to_string(),
        signals: Vec::new(),
    };

    m.signals.push(s.clone());
    m.signals.push(s);

    let mut net = can::CANNetwork {
        messages: Vec::new(),
    };

    net.add_msg(m.clone());
    net.add_msg(m);

    for msg in net.messages {
        for sig in msg.signals {
            println!("{}", sig._human_description());
            println!("{}", sig.cantools_description());
        }
    }
}
