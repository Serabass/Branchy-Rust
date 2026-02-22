use crate::ast::Program;
use std::io::Read;

const MAGIC: &[u8] = b"BRCH";
/// Bumped to 2 when Node enum got span fields (incompatible with v1).
const VERSION: u16 = 2;

pub fn serialize_program(program: &Program) -> Result<Vec<u8>, String> {
  let payload = bincode::serialize(program).map_err(|e| e.to_string())?;
  let mut out = Vec::with_capacity(MAGIC.len() + 2 + payload.len());
  out.extend_from_slice(MAGIC);
  out.extend_from_slice(&VERSION.to_le_bytes());
  out.extend_from_slice(&payload);
  Ok(out)
}

pub fn deserialize_program(mut bytes: &[u8]) -> Result<Program, String> {
  let mut magic = [0u8; 4];
  bytes.read_exact(&mut magic).map_err(|e| e.to_string())?;
  if &magic != MAGIC {
    return Err("invalid magic (not a Branchy binary)".into());
  }
  let mut ver = [0u8; 2];
  bytes.read_exact(&mut ver).map_err(|e| e.to_string())?;
  let version = u16::from_le_bytes(ver);
  if version != VERSION {
    return Err(format!("unsupported format version: {}", version));
  }
  let program = bincode::deserialize(bytes).map_err(|e| e.to_string())?;
  Ok(program)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parse_program;

  #[test]
  fn serialize_deserialize_roundtrip() {
    let p = parse_program("[ a; b; ]").unwrap();
    let bytes = serialize_program(&p).unwrap();
    assert!(bytes.starts_with(b"BRCH"));
    assert_eq!(bytes[4..6], VERSION.to_le_bytes());
    let p2 = deserialize_program(&bytes).unwrap();
    assert_eq!(p.includes, p2.includes);
    assert_eq!(p.functions.len(), p2.functions.len());
    assert_eq!(p.events.len(), p2.events.len());
  }

  #[test]
  fn deserialize_invalid_magic() {
    let bytes = b"XXXX\x02\x00";
    let err = deserialize_program(bytes).unwrap_err();
    assert!(err.contains("invalid magic"), "got: {}", err);
  }

  #[test]
  fn deserialize_unsupported_version() {
    let p = parse_program("[ x; ]").unwrap();
    let payload = bincode::serialize(&p).unwrap();
    let mut bytes = Vec::from(MAGIC);
    bytes.extend_from_slice(&99u16.to_le_bytes());
    bytes.extend_from_slice(&payload);
    let err = deserialize_program(&bytes).unwrap_err();
    assert!(err.contains("unsupported format version"), "got: {}", err);
  }
}
