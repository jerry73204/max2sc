//! Pattern and event types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PatternType {
    Pbind,
    Pseq,
    Ppar,
    Routine,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub params: HashMap<String, EventValue>,
    pub duration: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventValue {
    Float(f32),
    Symbol(String),
    Array(Vec<EventValue>),
}
