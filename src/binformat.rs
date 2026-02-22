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
