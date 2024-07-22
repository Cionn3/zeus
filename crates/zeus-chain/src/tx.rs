use crate::defi_types::currency::erc20::ERC20Token;
use std::str::FromStr;

use crate::{
    alloy::{
        network::{eip2718::Encodable2718, EthereumWallet, TransactionBuilder},
        primitives::{Address, Bytes, U256},
        providers::{Provider, ProviderBuilder},
        rpc::types::{TransactionRequest, TransactionReceipt},
        signers::{
            k256::ecdsa::SigningKey,
            local::LocalSigner,
        },
    },
    WsClient,
};

use tracing::trace;
use anyhow::Context;

#[derive(Default, Clone, Debug)]
pub enum TxVariant {
    #[default]
    EthTransfer,
    ERC20Transfer(ERC20Token),
    ERC20Approve(ERC20Token),
    Swap,
    Other,
}


/// Build and send transactions
#[derive(Clone, Debug)]
pub struct TxData {
    signer: LocalSigner<SigningKey>,
    pub client: WsClient,
    pub next_base_fee: U256,
    pub call_data: Bytes,
    pub to: Address,
    pub value: U256,
    pub nonce: u64,
    pub priority_fee: U256,
    pub gas_used: u128,
    pub chain_id: u64,
    pub mev_protect: bool,
}

impl TxData {
    pub fn new(
        signer: LocalSigner<SigningKey>,
        client: WsClient,
        next_base_fee: U256,
        call_data: Bytes,
        to: Address,
        value: U256,
        nonce: u64,
        priority_fee: U256,
        gas_used: u128,
        chain_id: u64,
        mev_protect: bool,
    ) -> Self {
        Self {
            signer,
            client,
            next_base_fee,
            call_data,
            to,
            value,
            nonce,
            priority_fee,
            gas_used,
            chain_id,
            mev_protect,
        }
    }

    pub fn priority_fee_u128(&self) -> Result<u128, anyhow::Error> {
        let p = u128::from_str(&self.priority_fee.to_string().as_str())
            .context("Failed to convert priority fee to u128");
        Ok(p?)
    }

    pub fn next_base_fee_u128(&self) -> Result<u128, anyhow::Error> {
        let p = u128::from_str(&self.next_base_fee.to_string().as_str())
            .context("Failed to convert next base fee to u128");
        Ok(p?)
    }

    pub fn max_fee_per_gas(&self) -> Result<u128, anyhow::Error> {
        let priority = self.priority_fee_u128()?;
        let base = self.next_base_fee_u128()?;
        Ok(priority + base)

    }

    pub fn calc_gas_limit(&self) -> u128 {
        (self.gas_used * 15) / 10 // +50% gas limit
    }

    pub fn build_transaction(&self) -> Result<TransactionRequest, anyhow::Error> {
        let gas_limit = self.calc_gas_limit();

        let tx = if self.chain_id == 56 {
            // build a legacy transaction
            TransactionRequest::default()
            .with_from(self.signer.address())
            .with_to(self.to)
            .with_input(self.call_data.clone())
            .with_value(self.value)
            .with_nonce(self.nonce)
            .with_chain_id(self.chain_id)
            .with_gas_price(self.next_base_fee_u128()?)
            .with_gas_limit(gas_limit)
        } else {
            // build an eip-1559
            TransactionRequest::default()
            .with_from(self.signer.address())
            .with_to(self.to)
            .with_input(self.call_data.clone())
            .with_value(self.value)
            .with_nonce(self.nonce)
            .with_chain_id(self.chain_id)
            .with_gas_limit(gas_limit)
            .with_max_priority_fee_per_gas(self.priority_fee_u128()?)
            .with_max_fee_per_gas(self.max_fee_per_gas()?)
        };

        Ok(tx)
    }

     /// Send a private transaction with flashbots
     /// This is only used for Ethereum
     pub async fn send_tx_with_flashbots(&self) -> Result<TransactionReceipt, anyhow::Error> {
        let flashbots = "https://rpc.flashbots.net/fast".parse()?;

        let provider = ProviderBuilder::new().on_http(flashbots);

        let wallet = EthereumWallet::from(self.signer.clone());

        let tx = self.build_transaction()?;
        let tx_envelope = tx.build(&wallet).await?;
        let tx_encoded = tx_envelope.encoded_2718();
        let pending = provider.send_raw_transaction(&tx_encoded).await?;
        trace!("Transaction sent! {}", pending.tx_hash());

        // wait for the receipt
        let receipt = pending.get_receipt().await?;
        Ok(receipt)
     }

     /// Send a transaction without Mev protection
     pub async fn send_tx(&self) -> Result<TransactionReceipt, anyhow::Error> {
        let wallet = EthereumWallet::from(self.signer.clone());

        let tx = self.build_transaction()?;
        let tx_envelope = tx.build(&wallet).await?;

        let receipt = self.client.send_tx_envelope(tx_envelope).await?.get_receipt().await?;
        Ok(receipt)
     }
}