//! Program, events, function definitions.

use serde::{Deserialize, Serialize};

use super::node::Node;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
  #[serde(default)]
  pub includes: Vec<String>,
  pub functions: Vec<FunctionDef>,
  #[serde(default)]
  pub events: Vec<Event>,
  pub main: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventMatcher {
  ByName(String),
  ByStr(String),
  ByRegex(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
  pub matcher: EventMatcher,
  pub body: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
  pub name: String,
  pub params: Vec<String>,
  pub body: Node,
}
