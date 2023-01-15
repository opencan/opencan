//! YAML format specification as Rust structs deserialized by serde.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum YEnumeratedValue {
    Auto(String),
    Exact(u64),
}

impl std::fmt::Debug for YEnumeratedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto(arg0) => write!(f, "{arg0}"),
            Self::Exact(arg0) => write!(f, "{arg0}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YSignal {
    pub width: Option<u32>,

    pub start_bit: Option<u32>,

    pub description: Option<String>,

    pub scale: Option<f32>,
    pub unit: Option<String>,

    #[serde(default)]
    pub enumerated_values: Vec<HashMap<String, YEnumeratedValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YMessage {
    pub id: u32,

    pub cycletime: Option<u32>,

    pub signals: Vec<HashMap<String, YSignal>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YNode {
    pub messages: Vec<HashMap<String, YMessage>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YDesc {
    pub nodes: Vec<HashMap<String, YNode>>,
}
