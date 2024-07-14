use r2d2::{Pool as connPool, PooledConnection};
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

use anyhow::anyhow;
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use tracing::{error, info, trace};
use zeus_chain::{
    alloy::primitives::{Address, U256},
    Currency, ERC20Token, Pool, PoolVariant,
};

#[derive(Clone)]
pub struct ZeusDB {
    pub erc20_tokens: connPool<SqliteConnectionManager>,
    pub pools: connPool<SqliteConnectionManager>,
    pub erc20_balance: connPool<SqliteConnectionManager>,
    pub eth_balance: connPool<SqliteConnectionManager>,
}

impl ZeusDB {
    pub fn new() -> Result<Self, anyhow::Error> {
        let db_path = PathBuf::from("db");

        std::fs::create_dir_all(&db_path)?;

        let erc20_manager = SqliteConnectionManager::file(db_path.join("erc20.db"));
        let erc20_conn = connPool::builder().build(erc20_manager)?;

        {
            let conn = erc20_conn.get()?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS ERC20Token (
                          id              INTEGER PRIMARY KEY,
                          chain_id         INTEGER NOT NULL,
                          address            TEXT NOT NULL,
                          symbol             TEXT NOT NULL,
                          name         TEXT NOT NULL,
                          decimals         INTEGER NOT NULL,
                          total_supply         TEXT NOT NULL,
                          UNIQUE(chain_id, address)
                          )",
                [],
            )?;
        }

        let pools_manager = SqliteConnectionManager::file(db_path.join("pools.db"));
        let pools_conn = connPool::builder().build(pools_manager)?;

        {
            let conn = pools_conn.get()?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS Pool (
                          id              INTEGER PRIMARY KEY,
                          chain_id         INTEGER NOT NULL,
                          address            TEXT NOT NULL,
                          token0             TEXT NOT NULL,
                          token1             TEXT NOT NULL,
                          variant            TEXT NOT NULL,
                          fee                INTEGER NOT NULL,
                          UNIQUE(chain_id, address)
                          )",
                [],
            )?;
        }

        let erc20_balance_manager = SqliteConnectionManager::file(db_path.join("erc20_balance.db"));
        let erc20_balance_conn = connPool::builder().build(erc20_balance_manager)?;

        {
            let conn = erc20_balance_conn.get()?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS ERC20Balance (
                          id              INTEGER PRIMARY KEY,
                          chain_id         INTEGER NOT NULL,
                          block_number         INTEGER NOT NULL,
                          owner            TEXT NOT NULL,
                          token            TEXT NOT NULL,
                          balance             TEXT NOT NULL,
                          UNIQUE(owner, token, block_number)
                          )",
                [],
            )?;
        }

        let eth_balance_manager = SqliteConnectionManager::file(db_path.join("eth_balance.db"));
        let eth_balance_conn = connPool::builder().build(eth_balance_manager)?;

        {
            let conn = eth_balance_conn.get()?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS ETHBalance (
                          id              INTEGER PRIMARY KEY,
                          chain_id         INTEGER NOT NULL,
                          block_number         INTEGER NOT NULL,
                          address            TEXT NOT NULL,
                          balance             TEXT NOT NULL,
                          UNIQUE(address, block_number, chain_id)
                          )",
                [],
            )?;
        }

        Ok(Self {
            erc20_tokens: erc20_conn,
            pools: pools_conn,
            erc20_balance: erc20_balance_conn,
            eth_balance: eth_balance_conn,
        })
    }

    fn get_erc20_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, anyhow::Error> {
        self.erc20_tokens
            .get()
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn get_pools_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, anyhow::Error> {
        self.pools.get().map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn get_erc20_balance_conn(
        &self,
    ) -> Result<PooledConnection<SqliteConnectionManager>, anyhow::Error> {
        self.erc20_balance
            .get()
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    /// Get the eth balance of a given address at a given block for a given chain
    pub fn get_eth_balance(
        &self,
        address: Address,
        chain_id: u64,
        block: u64,
    ) -> Result<U256, anyhow::Error> {
        let conn = self.eth_balance.get()?;
        let mut stmt = conn.prepare("SELECT * FROM ETHBalance WHERE address = ?1, ?2, ?3")?;
        let mut rows = stmt.query(params![chain_id, block, address.to_string()])?;

        let eth_balance;
        if let Some(row) = rows.next()? {
            let balance: String = row.get(4)?;
            eth_balance = U256::from_str(&balance)?;
            Ok(eth_balance)
        } else {
            Ok(U256::ZERO)
        }
    }

    /// Insert the eth balance of a given address at a given block for a given chain
    pub fn insert_eth_balance(
        &self,
        address: Address,
        balance: U256,
        chain_id: u64,
        block: u64,
    ) -> Result<(), anyhow::Error> {
        let conn = self.eth_balance.get()?;
        conn.execute(
            "INSERT INTO ETHBalance (chain_id, block_number, address, balance) VALUES (?1, ?2, ?3, ?4)",
            params![chain_id, block, address.to_string(), balance.to_string()],
        )?;

        // remove any old balances < block
        if let Err(e) = self.remove_eth_balance(block, chain_id) {
            error!("Error removing old eth balances: {:?}", e);
        }

        Ok(())
    }

    /// Remove old eth balances from a given block for a given chain
    pub fn remove_eth_balance(&self, block: u64, chain_id: u64) -> Result<(), anyhow::Error> {
        let conn = self.eth_balance.get()?;
        conn.execute(
            "DELETE FROM ETHBalance WHERE block_number < ?1 AND chain_id = ?2",
            params![block, chain_id],
        )?;
        Ok(())
    }

    /// Insert a new [ERC20Token] into the database
    pub fn insert_erc20(&self, token: ERC20Token, chain_id: u64) -> Result<(), anyhow::Error> {
        let time = std::time::Instant::now();
        let conn = self.get_erc20_conn()?;
        conn.execute(
            "INSERT INTO ERC20Token (chain_id, address, symbol, name, decimals, total_supply) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                chain_id,
                token.address.to_string(),
                token.symbol.to_string(),
                token.name.to_string(),
                token.decimals.to_string(),
                token.total_supply.to_string()
            ],
        )?;
        info!("Time to insert: {:?}ms", time.elapsed().as_millis());
        Ok(())
    }

    /// Insert a new [Pool] into the database
    pub fn insert_pool(&self, pool: Pool, chain_id: u64) -> Result<(), anyhow::Error> {
        let conn = self.get_pools_conn()?;
        conn.execute(
            "INSERT INTO Pool (chain_id, address, token0, token1, variant, fee) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                chain_id,
                pool.address.to_string(),
                pool.token0.address.to_string(),
                pool.token1.address.to_string(),
                pool.variant().to_string(),
                pool.fee
            ],
        )?;
        Ok(())
    }

    /// Get the [ERC20Token] from the given address and chain_id
    pub fn get_erc20(&self, address: Address, chain_id: u64) -> Result<ERC20Token, anyhow::Error> {
        let conn = self.get_erc20_conn()?;
        let mut stmt = conn.prepare("SELECT * FROM ERC20Token WHERE address = ?1, ?2")?;
        let mut rows = stmt.query(params![address.to_string(), chain_id])?;

        if let Some(row) = rows.next()? {
            let chain_id: i32 = row.get(1)?;
            let address: String = row.get(2)?;
            let symbol: String = row.get(3)?;
            let name: String = row.get(4)?;
            let decimals: i32 = row.get(5)?;
            let total_supply: String = row.get(6)?;

            let token = ERC20Token {
                chain_id: chain_id as u64,
                address: address.parse().unwrap(),
                symbol,
                name,
                decimals: decimals as u8,
                total_supply: total_supply.parse().unwrap(),
                icon: None,
            };

            Ok(token)
        } else {
            Err(anyhow!("Token not found"))
        }
    }

    /// Get the [Pool] from the given token0, token1, pool variant, chain_id and fee
    pub fn get_pool(
        &self,
        token0: ERC20Token,
        token1: ERC20Token,
        chain_id: u64,
        variant: PoolVariant,
        fee: u32,
    ) -> Result<Pool, anyhow::Error> {
        let token0_addr = token0.address.to_string();
        let token1_addr = token1.address.to_string();

        let pool_variant = match variant {
            PoolVariant::UniswapV2 => U256::ZERO.to_string(),
            PoolVariant::UniswapV3 => U256::from(1).to_string(),
        };

        let time = std::time::Instant::now();
        let conn = self.get_pools_conn()?;
        let mut stmt = conn.prepare("SELECT * FROM Pool WHERE chain_id = ?1 AND token0 = ?2 AND token1 = ?3 AND variant = ?4 AND fee = ?5")?;
        let mut rows = stmt.query(params![
            chain_id,
            token0_addr,
            token1_addr,
            pool_variant,
            fee
        ])?;

        if let Some(row) = rows.next()? {
            let address: String = row.get(2)?;
            let variant: String = row.get(5)?;
            let pool_fee: u32 = row.get(6)?;

            let pool = Pool {
                chain_id,
                address: address.parse().unwrap(),
                token0,
                token1,
                variant: PoolVariant::from_u256(variant.parse().unwrap()),
                fee: pool_fee,
            };
            trace!(
                "Time to get pool from db: {:?}ms",
                time.elapsed().as_millis()
            );
            Ok(pool)
        } else {
            Err(anyhow!("Pool not found"))
        }
    }

    /// Get all [ERC20Token] from the given chain_id
    pub fn get_all_erc20(&self, chain_id: u64) -> Result<Vec<ERC20Token>, anyhow::Error> {
        let conn = self.get_erc20_conn()?;
        let mut stmt = conn.prepare("SELECT * FROM ERC20Token WHERE chain_id = ?1")?;
        let mut rows = stmt.query(params![chain_id])?;
        let mut tokens = Vec::new();

        while let Some(row) = rows.next()? {
            let chain_id: i32 = row.get(1)?;
            let address: String = row.get(2)?;
            let symbol: String = row.get(3)?;
            let name: String = row.get(4)?;
            let decimals: i32 = row.get(5)?;
            let total_supply: String = row.get(6)?;

            let token = ERC20Token {
                chain_id: chain_id as u64,
                address: address.parse().unwrap(),
                symbol,
                name,
                decimals: decimals as u8,
                total_supply: total_supply.parse().unwrap(),
                icon: None,
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Insert the balance of a token at a given block for a given chain
    pub fn insert_erc20_balance(
        &self,
        owner: Address,
        token: Address,
        balance: U256,
        chain_id: u64,
        block: u64,
    ) -> Result<(), anyhow::Error> {
        let conn = self.get_erc20_balance_conn()?;
        conn.execute(
            "INSERT INTO ERC20Balance (chain_id, block_number, owner, token, balance) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![chain_id, block, owner.to_string(), token.to_string(), balance.to_string()],
        )?;

        // remove any old balances < block
        if let Err(e) = self.remove_erc20_balance(owner, token, block, chain_id) {
            error!("Error removing old erc20 balances: {:?}", e);
        }

        Ok(())
    }

    /// Get the balance of a token at a given block for a given chain
    pub fn get_erc20_balance_at_block(
        &self,
        owner: Address,
        token: Address,
        chain_id: u64,
        block: u64,
    ) -> Result<U256, anyhow::Error> {
        let conn = self.get_erc20_balance_conn()?;
        let mut stmt = conn.prepare("SELECT * FROM ERC20Balance WHERE owner = ?1 AND token = ?2 AND chain_id = ?3 AND block_number = ?4")?;
        let mut rows = stmt.query(params![
            owner.to_string(),
            token.to_string(),
            chain_id,
            block
        ])?;

        if let Some(row) = rows.next()? {
            let balance: String = row.get(4)?;
            let erc_balance = U256::from_str(&balance)?;
            Ok(erc_balance)
        } else {
            return Err(anyhow!("Balance not found"));
        }
    }

    /// Get the latest balance of token for a given chain
    pub fn get_latest_erc20_balance(
        &self,
        owner: Address,
        token: Address,
        chain_id: u64,
    ) -> Result<U256, anyhow::Error> {
        let conn = self.get_erc20_balance_conn()?;
        let mut stmt = conn.prepare("SELECT * FROM ERC20Balance WHERE owner = ?1 AND token = ?2 AND chain_id = ?3 ORDER BY block_number DESC LIMIT 1")?;
        let mut rows = stmt.query(params![owner.to_string(), token.to_string(), chain_id])?;

        if let Some(row) = rows.next()? {
            let balance: String = row.get(4)?;
            let erc_balance = U256::from_str(&balance)?;
            Ok(erc_balance)
        } else {
            return Err(anyhow!("Balance not found"));
        }
    }


    /// Remove old erc20 balances from a given block for a given chain
    pub fn remove_erc20_balance(
        &self,
        owner: Address,
        token: Address,
        block: u64,
        chain_id: u64,
    ) -> Result<(), anyhow::Error> {
        let conn = self.get_erc20_balance_conn()?;
        conn.execute("DELETE FROM ERC20Balance WHERE block_number < ?1 AND owner = ?2 AND token = ?3 AND chain_id = ?4",
         params![block, owner.to_string(), token.to_string(), chain_id])?;
        Ok(())
    }

    /// Load all erc20 balances with their owner for all chains
    pub fn load_all_erc20_balances(
        &self,
        chain_ids: Vec<u64>,
    ) -> Result<HashMap<(u64, Address, Address), U256>, anyhow::Error> {
        let mut balances_map = HashMap::new();
        for chain_id in chain_ids {
            let conn = self.get_erc20_balance_conn()?;
            let mut stmt = conn.prepare("SELECT * FROM ERC20Balance WHERE chain_id = ?1")?;
            let mut rows = stmt.query(params![chain_id])?;

            while let Some(row) = rows.next()? {
                let owner: String = row.get(3)?;
                let token: String = row.get(4)?;
                let balance: String = row.get(5)?;
                let erc_balance = U256::from_str(&balance)?;

                balances_map.insert((chain_id, owner.parse()?, token.parse()?), erc_balance);
            }
        }

        Ok(balances_map)
    }

    /// Load all eth balances with their owner for all chains
    pub fn load_all_eth_balances(
        &self,
        chain_ids: Vec<u64>,
    ) -> Result<HashMap<(u64, Address), (u64, U256)>, anyhow::Error> {
        let mut balances_map = HashMap::new();
        for chain_id in chain_ids {
            let conn = self.eth_balance.get()?;
            let mut stmt = conn.prepare("SELECT * FROM ETHBalance WHERE chain_id = ?1")?;
            let mut rows = stmt.query(params![chain_id])?;

            while let Some(row) = rows.next()? {
                let address: String = row.get(3)?;
                let block: u64 = row.get(2)?;
                let balance: String = row.get(4)?;
                let eth_balance = U256::from_str(&balance)?;

                balances_map.insert((chain_id, address.parse()?), (block, eth_balance));
            }
        }

        Ok(balances_map)
    }


    /// Load all tokens to a hashmap
    pub fn load_currencies(
        &self,
        id: Vec<u64>,
    ) -> Result<HashMap<u64, Vec<Currency>>, anyhow::Error> {
        let mut currencies_map = HashMap::new();
        for chain_id in id {
            let mut currencies = Vec::new();

            let erc20_tokens = self.get_all_erc20(chain_id)?;
            let native_currency = Currency::new_native(chain_id);
            currencies.push(native_currency);

            for token in erc20_tokens {
                let erc20_currency = Currency::new_erc20(token);
                currencies.push(erc20_currency);
            }

            currencies_map.insert(chain_id, currencies.clone());
        }

        Ok(currencies_map)
    }

    /// insert some default tokens
    pub fn insert_default(&self) -> Result<(), anyhow::Error> {
        let eth_token1 = ERC20Token::eth_default_input();
        let eth_token2 = ERC20Token::eth_default_output();
        let bsc_token1 = ERC20Token::bsc_default_input();
        let bsc_token2 = ERC20Token::bsc_default_output();
        let base_token1 = ERC20Token::base_default_input();
        let base_token2 = ERC20Token::base_default_output();
        let arbitrum_token1 = ERC20Token::arbitrum_default_input();
        let arbitrum_token2 = ERC20Token::arbitrum_default_output();

        let tokens = vec![
            eth_token1,
            eth_token2,
            bsc_token1,
            bsc_token2,
            base_token1,
            base_token2,
            arbitrum_token1,
            arbitrum_token2,
        ];

        for token in &tokens {
            self.insert_erc20(token.clone(), token.chain_id.clone())?;
        }
        Ok(())
    }
}
