//! Request/response types for API.

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
  pub builtins: std::sync::Arc<std::collections::HashMap<String, crate::builtins::BuiltinFn>>,
}

#[derive(Deserialize)]
pub struct RunRequest {
  pub source: String,
  #[serde(default)]
  pub input: Option<String>,
  #[serde(default, deserialize_with = "deserialize_seed")]
  pub seed: Option<u64>,
}

fn deserialize_seed<'de, D>(d: D) -> Result<Option<u64>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  use serde::de::{Error, Visitor};
  use std::fmt;
  struct SeedVisitor;
  impl<'de> Visitor<'de> for SeedVisitor {
    type Value = Option<u64>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "a number or null")
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
      Ok(None)
    }
    fn visit_some<A>(self, value: A) -> Result<Self::Value, A::Error>
    where
      A: serde::Deserializer<'de>,
    {
      let n = u64::deserialize(value).map_err(|_| A::Error::custom("seed must be a number"))?;
      Ok(Some(n))
    }
  }
  d.deserialize_option(SeedVisitor)
}

#[derive(Serialize)]
pub struct RunResponse {
  pub result: String,
  pub trace: Vec<crate::ast::Span>,
}

#[derive(Serialize)]
pub struct ExampleItem {
  pub id: String,
  pub name: String,
  pub source: String,
}

#[derive(Deserialize)]
pub struct FormatRequest {
  pub source: String,
}

#[derive(Serialize)]
pub struct FormatResponse {
  pub formatted: String,
}
