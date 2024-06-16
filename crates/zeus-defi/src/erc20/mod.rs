use alloy::{
    primitives::{ Address, Bytes, U256 },
    providers::RootProvider,
    sol,
};
use alloy::pubsub::PubSubFrontend;
use alloy::core::sol_types::SolCall;
use std::sync::Arc;
use std::str::FromStr;
use tokio::try_join;

sol! {
    #[sol(rpc)]
    contract ERC20 {
        function balanceOf(address owner) external view returns (uint256 balance);
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function transferFrom(address from, address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function deposit() external payable;
        function withdraw(uint256 amount) external;
    
}
}



/// Struct that holds ERC20 token information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ERC20Token {
    pub address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: U256,
}


impl ERC20Token {
    pub async fn new(
        address: Address,
        client: Arc<RootProvider<PubSubFrontend>>
    ) -> Result<Self, anyhow::Error> {
        
        let symbol = Self::symbol(address, client.clone());
        let name = Self::name(address, client.clone());
        let decimals = Self::decimals(address, client.clone());
        let total_supply = Self::total_supply(address, client.clone());
        let res = try_join!(symbol, name, decimals, total_supply);
        let (symbol, name, decimals, total_supply) = res?;
        Ok(Self {
            address,
            symbol,
            name,
            decimals,
            total_supply,
        })
    }

    async fn symbol(address: Address, client: Arc<RootProvider<PubSubFrontend>>) -> Result<String, anyhow::Error> {
        let contract = ERC20::new(address, client);
        let symbol = contract.symbol().call().await?._0;
        Ok(symbol)
    }

    async fn name(address: Address, client: Arc<RootProvider<PubSubFrontend>>) -> Result<String, anyhow::Error> {
        let contract = ERC20::new(address, client);
        let name = contract.name().call().await?._0;
        Ok(name)
    }

    async fn decimals(address: Address, client: Arc<RootProvider<PubSubFrontend>>) -> Result<u8, anyhow::Error> {
        let contract = ERC20::new(address, client);
        let decimals = contract.decimals().call().await?._0;
        Ok(decimals)
    }

    async fn total_supply(address: Address, client: Arc<RootProvider<PubSubFrontend>>) -> Result<U256, anyhow::Error> {
        let contract = ERC20::new(address, client);
        let total_supply = contract.totalSupply().call().await?._0;
        Ok(total_supply)
    }

    pub async fn balance_of(
        &self,
        owner: Address,
        client: Arc<RootProvider<PubSubFrontend>>
    ) -> Result<U256, anyhow::Error> {
        let contract = ERC20::new(self.address, client);
        let bal = contract.balanceOf(owner).call().await?;
        Ok(bal.balance)
    }

    pub async fn allowance(
        &self,
        owner: Address,
        spender: Address,
        client: Arc<RootProvider<PubSubFrontend>>
    ) -> Result<U256, anyhow::Error> {
        let contract = ERC20::new(self.address, client);
        let allowance = contract.allowance(owner, spender).call().await?._0;
        Ok(allowance)
    }

    pub fn encode_balance_of(&self, owner: Address) -> Vec<u8> {
        let contract = ERC20::balanceOfCall {
            owner,
        };
        contract.abi_encode()
    }

    pub fn encode_approve(&self, spender: Address, amount: U256) -> Vec<u8> {
        let contract = ERC20::approveCall {
            spender,
            amount,
        };
        contract.abi_encode()
    }

    pub fn encode_transfer(&self, recipient: Address, amount: U256) -> Vec<u8> {
        let contract = ERC20::transferCall {
            recipient,
            amount,
        };
        contract.abi_encode()
    }

    pub fn encode_deposit(&self) -> Vec<u8> {
        let contract = ERC20::depositCall {};
        contract.abi_encode()
    }

    pub fn encode_withdraw(&self, amount: U256) -> Vec<u8> {
        let contract = ERC20::withdrawCall { amount };
        contract.abi_encode()
    }

    pub fn decode_balance_of(&self, bytes: &Bytes) -> Result<U256, anyhow::Error> {
        let balance = ERC20::balanceOfCall::abi_decode_returns(&bytes, true)?;
        Ok(balance.balance)
    }

   pub fn default_input() -> Self {
        Self {
            name: "Wrapped Ether".to_string(),
            address: Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
            decimals: 18,
            symbol: "WETH".to_string(),
            total_supply: U256::ZERO,
        }
    }

    pub fn default_output() -> Self {
        Self {
            name: "USC Coin".to_string(),
            address: Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
            decimals: 18,
            symbol: "USDC".to_string(),
            total_supply: U256::ZERO,
        }
    }

}

impl Default for ERC20Token {
    fn default() -> Self {
        Self {
            name: "Wrapped Ether".to_string(),
            address: Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
            decimals: 18,
            symbol: "WETH".to_string(),
            total_supply: U256::ZERO,
        }
    }
}



pub fn default_tokens() -> Vec<ERC20Token> {
    let mut tokens = Vec::new();
    tokens.push(ERC20Token {
        name: "USD Coin".to_string(),
        address: Address::from_str("0x2791bca1f2de4661ed88a30c99a7a9449aa84174").unwrap(),
        decimals: 6,
        symbol: "USDC".to_string(),
        total_supply: U256::ZERO,
    });
    tokens.push(ERC20Token {
        name: "Wrapped Ether".to_string(),
        address: Address::from_str("0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619").unwrap(),
        decimals: 18,
        symbol: "WETH".to_string(),
        total_supply: U256::ZERO,
    });
    
    tokens

}