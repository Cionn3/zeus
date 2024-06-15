pub mod database_error;

pub mod global_backend;
pub use global_backend::*;

pub mod fork_db;
pub mod fork_factory;


use tiny_keccak::{Keccak, Hasher};
use revm::primitives::Bytes;





pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut result = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut result);
    result
}

/// Revert message from EVM
pub fn revert_msg(bytes: &Bytes) -> String {
    if bytes.len() < 4 {
        return "EVM Returned 0x (Empty Bytes)".to_string();
    }
    let error_data = &bytes[4..];

    match String::from_utf8(error_data.to_vec()) {
        Ok(s) => s.trim_matches(char::from(0)).to_string(),
        Err(_) => "EVM Returned 0x (Empty Bytes)".to_string(),
    }
}