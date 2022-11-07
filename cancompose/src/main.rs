use std::{collections::HashMap};

use indoc::indoc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum JEnumeratedValue {
    Auto(String),
    Exact(u32),
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
    #[serde(flatten)]
    messages: Vec<(String, JMessage)>
}

fn main() {
    let input = indoc! {r#"
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

    println!("{de:#?}");
}
