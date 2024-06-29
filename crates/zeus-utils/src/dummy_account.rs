use alloy::signers::local::LocalSigner;
use alloy::primitives::{Address, U256};
use revm::primitives::{Bytecode, AccountInfo, B256};
use std::str::FromStr;

use zeus_types::{ChainId, forked_db::{fork_factory::ForkFactory, keccak256}};

#[derive(Clone)]
pub enum AccountType {
    /// Externally Owned Account
    EOA,

    /// An Ethereum Smart Contract
    Contract(Bytecode),
}

/// Represents a dummy account we want to insert into the fork enviroment
#[derive(Clone)]
pub struct DummyAccount {
    pub account_type: AccountType,

    /// ETH balance to fund with
    pub balance: U256,

    /// WETH balance to fund with
    pub weth_balance: U256,

    pub address: Address
}

impl DummyAccount {
    pub fn new(account_type: AccountType, balance: U256, weth_balance: U256) -> Self {
        Self {
            account_type,
            balance,
            weth_balance,
            address: LocalSigner::random().address()
        }
    }
}



/// Inserts a dummy account to the local fork enviroment
pub fn insert_dummy_account(
    account: &DummyAccount,
    chain_id: ChainId,
    fork_factory: &mut ForkFactory
) -> Result<(), anyhow::Error> {

    let code = match &account.account_type {
        AccountType::EOA => Bytecode::default(),
        AccountType::Contract(code) => code.clone(),
    };

    let account_info = AccountInfo {
        balance: account.balance,
        nonce: 0,
        code_hash: B256::default(),
        code: Some(code),
    };

    // insert the account info into the fork enviroment
    fork_factory.insert_account_info(account.address, account_info);


    // To fund any ERC20 token to an account we need the balance storage slot of the token
    // For WETH its 3
    // An amazing online tool to see the storage mapping of any contract https://evm.storage/
    let slot_num = U256::from(3);
    let addr_padded = pad_left(account.address.to_vec(), 32);
    let slot = slot_num.to_be_bytes_vec();
    
    let data = [&addr_padded, &slot].iter().flat_map(|x| x.iter().copied()).collect::<Vec<u8>>();
    let slot_hash = keccak256(&data);
    let slot: U256 = U256::from_be_bytes(slot_hash.try_into().expect("Hash must be 32 bytes"));
    
    let native_coin = get_native_coin(chain_id);

    // insert the erc20 token balance to the dummy account
    if let Err(e) = fork_factory.insert_account_storage(native_coin, slot, account.weth_balance) {
        return Err(anyhow::anyhow!("Failed to insert account storage: {}", e));
    }

    Ok(())
}

pub fn get_native_coin(chain_id: ChainId) -> Address {
    match chain_id {
        ChainId::Ethereum(_) => Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
        ChainId::BinanceSmartChain(_) => Address::from_str("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c").unwrap(),
        ChainId::Base(_) => Address::from_str("0x4200000000000000000000000000000000000006").unwrap(),
        ChainId::Arbitrum(_) => Address::from_str("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1").unwrap(),
    }
}

fn pad_left(vec: Vec<u8>, full_len: usize) -> Vec<u8> {
    let mut padded = vec![0u8; full_len - vec.len()];
    padded.extend(vec);
    padded
}