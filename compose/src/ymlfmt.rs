//! YAML format specification as Rust structs deserialized by serde.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Small helper for turning single-length maps into a tuple.
///
/// Serde deserializes:
/// - signalName:
///     (parameter)
///
/// As a `map<String, YSignal>` with length 1. We then typically have a vector
/// of these, because it's both a sequence element and we still want to have
/// the `':'` after it.
pub fn unmap<T>(map: &HashMap<String, T>) -> (&String, &T) {
    // len should be one because every `- VALUE: val` pair is its own dict
    assert_eq!(map.len(), 1);
    map.iter().next().unwrap()
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum YEnumeratedValue {
    Auto(String),
    Exact(HashMap<String, u64>),
}

impl std::fmt::Debug for YEnumeratedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto(_) => write!(f, "(auto)"),
            Self::Exact(map) => write!(f, "{}", unmap(map).0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YSignal {
    pub width: Option<u32>,

    pub start_bit: Option<u32>,

    pub description: Option<String>,

    #[serde(default)]
    pub twos_complement: bool,

    pub scale: Option<f64>,
    pub offset: Option<f64>,

    pub unit: Option<String>,

    #[serde(default)]
    pub enumerated_values: Vec<YEnumeratedValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YMessageTemplate {
    pub cycletime: Option<u32>,
    pub signals: Vec<HashMap<String, YSignal>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YMessage {
    pub id: u32,

    pub from_template: Option<String>,

    pub cycletime: Option<u32>,

    pub signals: Option<Vec<HashMap<String, YSignal>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RxDirective {
    /// Recieve all messages in the network
    ///
    ///   rx: "*"
    #[serde(rename = "*")]
    Everything,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RxListOrDirective {
    List(#[serde(default)] Vec<String>),
    Directive(RxDirective),
}

impl Default for RxListOrDirective {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct YNode {
    #[serde(default)]
    pub messages: Vec<HashMap<String, YMessage>>,

    #[serde(default)]
    pub rx: RxListOrDirective,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YDesc {
    pub nodes: Vec<HashMap<String, YNode>>,

    #[serde(default)]
    pub message_templates: Vec<HashMap<String, YMessageTemplate>>,

    #[serde(default)]
    pub bitrate: Option<u32>,

    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub lookup_path: String,
}
