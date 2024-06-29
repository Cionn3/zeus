use std::sync::{Arc, RwLock};

use crate::oracles::block::BlockOracle;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BLOCK_ORACLE: Arc<RwLock<BlockOracle>> = BlockOracle::default();
}

pub mod block;
pub mod fork;

pub enum OracleAction {
    STOP
}