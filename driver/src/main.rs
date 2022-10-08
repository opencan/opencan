use can;

fn main() {
    println!("Hello from protobrain.");
    println!("----------------------");

    let s = can::CANSignal {
        offset: 0,
        name: "VCFRONT_driverIsLeaving".to_string(),
        value_type: can::CANValueType::Integer(can::CANValueTypeInteger {
            length: 5,
            signed: false,
        }),
    };

    let mut new_msg = can::CANMessageDesc {
        name: "BRK_Status".to_string(),
        id: 0x20,
        signals: vec![s],
    };

    let mut net = can::CANNetwork::new();

    // horror: https://stackoverflow.com/questions/38023871/returning-a-reference-from-a-hashmap-or-vec-causes-a-borrow-to-last-beyond-the-s
    net.add_msg(new_msg.clone()).unwrap();
    new_msg.name = "NAH".into();

    match net.add_msg(new_msg) {
        Ok(_) => (),
        Err(s) => println!("{s}"),
    }

    let mm = net.message_by_name("BRK_Status").unwrap();

    println!("{}", mm["VCFRONT_driverIsLeaving"]);
}
