pub mod defi_types;
pub mod evm_types;
pub mod utils;
pub mod chain_id;
pub mod rpc;


// * Re-exports

pub use alloy;
pub use revm;
pub use serde_json;

pub use chain_id::ChainId;
pub use rpc::Rpc;
pub use utils::{get_client, parse_wei, format_wei, oracles::{OracleAction, block::{BlockInfo, BlockOracle, BLOCK_ORACLE, start_block_oracle}}};
pub use defi_types::{currency::{Currency, NativeCurrency, erc20::ERC20Token}, pool::*};



use alloy::{
    providers::RootProvider,
    pubsub::PubSubFrontend
};

pub type WsClient = RootProvider<PubSubFrontend>; 