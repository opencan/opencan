use std::{collections::HashMap, iter::Map};

use indoc::indoc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct JSignal {
    #[serde(default)]
    name: String,
    scale: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct JMessage {
    #[serde(default)]
    name: String,
    signals: Vec<HashMap<String, JSignal>>,
}

fn main() {
    let input = indoc! {r#"
    - BRAKE_BrakeData:
        signals:
          - brakePressure:
              scale: 0.5
          - brakePercent:
              scale: 0.01
    - BRAKE_BrakeData:
        signals:
          - brakePressure:
              scale: 0.5
          - brakePercent:
              scale: 0.01
   "#};
    let de: Vec<HashMap<String, JMessage>> = serde_yaml::from_str(&input).unwrap();
    for mut msg in de {
        let mut c: Vec<JMessage> = msg
            .into_iter()
            .map(|(k, mut v)| {
                v.name = k;
                v
            })
            .collect();

        println!("{:#?}", c);
    }
}
