use std::collections::HashMap;

use can::{CANMessageDesc, CANNetwork, CANSignal, CANValueTypeInteger};
use indoc::indoc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum JEnumeratedValue {
    Auto(String),
    Exact(u32),
}

impl std::fmt::Debug for JEnumeratedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto(arg0) => write!(f, "{arg0}"),
            Self::Exact(arg0) => write!(f, "{arg0}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct JSignal {
    scale: f32,

    #[serde(default)]
    unit: Option<String>,

    #[serde(default)]
    enumerated_values: Vec<HashMap<String, JEnumeratedValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct JMessage {
    id: u32,

    #[serde(default)]
    cycletime_ms: Option<f32>,

    #[serde(with = "tuple_vec_map")]
    signals: Vec<(String, JSignal)>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JDesc {
    #[serde(with = "tuple_vec_map")]
    messages: Vec<(String, JMessage)>,
}

fn main() {
    let input = indoc! {r#"
    messages:
      BRAKE_BrakeData:
        id: 0x100
        cycletime_ms: 1

        signals:
          brakePressure:
            scale: 0.5
          brakePercent:
            scale: 0.01
            unit: "%"
            enumerated_values:
              - SNA: auto
              - SATURATED: 1
    "#};
    let de: JDesc = serde_yaml::from_str(&input).unwrap();

    for msg in &de.messages {
        println!("{}: {:#?}\n", msg.0, msg.1);
        println!("{}", serde_yaml::to_string(&de).unwrap());
    }

    let mut net = CANNetwork::new();

    for msg in de.messages {
        let sigs: Vec<CANSignal> = msg
            .1
            .signals
            .into_iter()
            .map(|(name, j)| CANSignal {
                offset: 0,
                name: name,
                value_type: can::CANValueType::Integer(CANValueTypeInteger {
                    length: 0,
                    signed: false,
                }),
            })
            .collect();

        let desc = CANMessageDesc {
            name: msg.0.clone(),
            id: msg.1.id,
            signals: sigs,
        };

        net.new_msg(desc).unwrap();
    }
}
