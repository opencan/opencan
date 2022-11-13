use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum YEnumeratedValue {
    Auto(String),
    Exact(u32),
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
    pub scale: f32,

    #[serde(default)]
    pub unit: Option<String>,

    #[serde(default)]
    pub enumerated_values: Vec<HashMap<String, YEnumeratedValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YMessage {
    pub id: u32,

    #[serde(default)]
    pub cycletime_ms: Option<f32>,

    #[serde(with = "tuple_vec_map")]
    pub signals: Vec<(String, YSignal)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YDesc {
    #[serde(with = "tuple_vec_map")]
    pub messages: Vec<(String, YMessage)>,
}
