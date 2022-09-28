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

    let ss = can::CANSignal {
        offset: 6,
        name: "VCFRONT_drive2rIsLeaving".to_string(),
        value_type: can::CANValueType {
            length: 5,
            signed: false,
        },
    };

    let new_msg = can::CANMessageDesc {
        name: "VCFRONT_Occupancy".to_string(),
        id: 0x20,
        signals: vec![s, ss],
    };

    let mut net = can::CANNetwork::new();

    let msg = net.add_msg(new_msg).unwrap();
    net.print_msg_human(&msg);

    net.print_msg_human(&net.message_by_name("VCFRONT_Occupancy").unwrap());
}
