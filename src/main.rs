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
        name: "BRK_Status".to_string(),
        id: 0x20,
        signals: vec![s, ss],
    };

    let mut net = can::CANNetwork::new();

    // horror: https://stackoverflow.com/questions/38023871/returning-a-reference-from-a-hashmap-or-vec-causes-a-borrow-to-last-beyond-the-s
    let m = net.add_msg(new_msg).unwrap();
    let mm = net.message_by_name("BRK_Status");

    println!("{}", m["VCFRONT_driverIsLeaving"].human_description());
}
