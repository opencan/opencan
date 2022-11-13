use can::{CantoolsDecoder, TranslationLayer};

fn main() {
    println!("Hello from protobrain.");
    println!("----------------------");

    let s = can::CANSignal {
        position: 0,
        name: "VCFRONT_driverIsLeaving".into(),
        description: Some("Nothing important".into()),
        width: 1,
    };

    let mut new_msg = can::CANMessageDesc {
        name: "BRK_Status".into(),
        id: 0x20,
        cycletime_ms: Some(10),
        signals: vec![s],
    };

    let mut net = can::CANNetwork::new();

    net.new_msg(new_msg.clone()).unwrap();
    new_msg.name = "NAH".into();

    match net.new_msg(new_msg) {
        Ok(_) => (),
        Err(s) => println!("{s}"),
    }

    let mm = net.message_by_name("BRK_Status").unwrap();

    println!("{}", mm["VCFRONT_driverIsLeaving"]);
    println!("{}", serde_json::to_string_pretty(&net).unwrap());
    println!("{}", CantoolsDecoder::dump_network(&net));
}
