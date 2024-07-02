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
    pub chain_id: u64,
    pub address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: U256,
    pub icon: Option<Vec<u8>>,
}


impl ERC20Token {
    pub async fn new(
        address: Address,
        client: Arc<RootProvider<PubSubFrontend>>,
        chain_id: u64,
        icon: Option<Vec<u8>>,
    ) -> Result<Self, anyhow::Error> {
        
        let symbol = Self::symbol(address, client.clone());
        let name = Self::name(address, client.clone());
        let decimals = Self::decimals(address, client.clone());
        let total_supply = Self::total_supply(address, client.clone());
        let res = try_join!(symbol, name, decimals, total_supply);
        let (symbol, name, decimals, total_supply) = res?;
        Ok(Self {
            chain_id,
            address,
            symbol,
            name,
            decimals,
            total_supply,
            icon,
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

   pub fn eth_default_input() -> Self {
        Self {
            chain_id: 1,
            name: "Wrapped Ether".to_string(),
            address: Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
            decimals: 18,
            symbol: "WETH".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }

    pub fn eth_default_output() -> Self {
        Self {
            chain_id: 1,
            name: "USDC Coin".to_string(),
            address: Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
            decimals: 6,
            symbol: "USDC".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }

    pub fn bsc_default_input() -> Self {
        Self {
            chain_id: 56,
            name: "Wrapped BNB".to_string(),
            address: Address::from_str("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c").unwrap(),
            decimals: 18,
            symbol: "WBNB".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }

    pub fn bsc_default_output() -> Self {
        Self {
            chain_id: 56,
            name: "USDC Coin".to_string(),
            address: Address::from_str("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d").unwrap(),
            decimals: 18,
            symbol: "USDC".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }

    pub fn base_default_input() -> Self {
        Self {
            chain_id: 8453,
            name: "Wrapped Ether".to_string(),
            address: Address::from_str("0x4200000000000000000000000000000000000006").unwrap(),
            decimals: 18,
            symbol: "WETH".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }

    pub fn base_default_output() -> Self {
        Self {
            chain_id: 8453,
            name: "USDC Coin".to_string(),
            address: Address::from_str("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913").unwrap(),
            decimals: 6,
            symbol: "USDC".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }

        pub fn arbitrum_default_input() -> Self {
            Self {
                chain_id: 42161,
                name: "Wrapped Ether".to_string(),
                address: Address::from_str("0x82af49447d8a07e3bd95bd0d56f35241523fbab1").unwrap(),
                decimals: 18,
                symbol: "WETH".to_string(),
                total_supply: U256::ZERO,
                icon: None
            }
        }

        pub fn arbitrum_default_output() -> Self {
            Self {
                chain_id: 42161,
                name: "USDC Coin".to_string(),
                address: Address::from_str("0xaf88d065e77c8cc2239327c5edb3a432268e5831").unwrap(),
                decimals: 6,
                symbol: "USDC".to_string(),
                total_supply: U256::ZERO,
                icon: None
            }
        }

}


impl Default for ERC20Token {
    fn default() -> Self {
        Self {
            chain_id: 1,
            name: "Wrapped Ether".to_string(),
            address: Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
            decimals: 18,
            symbol: "WETH".to_string(),
            total_supply: U256::ZERO,
            icon: None
        }
    }
}