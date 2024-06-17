use rusqlite::{Connection as DbConnection, params};
use alloy::primitives::Address;
use zeus_defi::{erc20::ERC20Token, dex::uniswap::pool::{Pool, PoolVariant}};
use std::path::PathBuf;


pub struct ZeusDB {
    pub erc20_tokens: DbConnection,
    pub pools: DbConnection,
}

impl ZeusDB {
    pub fn new() -> Result<Self, anyhow::Error> {
        let db_path = PathBuf::from("db");

        std::fs::create_dir_all(&db_path)?;
    
        let erc20 = DbConnection::open(db_path.join("erc20.db"))?;
    
        erc20.execute(
            "CREATE TABLE IF NOT EXISTS ERC20Token (
                      id              INTEGER PRIMARY KEY,
                      chain_id         INTEGER NOT NULL,
                      address            TEXT NOT NULL,
                      symbol             TEXT NOT NULL,
                      name         TEXT NOT NULL,
                      decimals         INTEGER NOT NULL,
                      total_supply         TEXT NOT NULL
                      )",
            [],
        )?;

        let pools = DbConnection::open(db_path.join("pools.db"))?;

        pools.execute(
            "CREATE TABLE IF NOT EXISTS Pool (
                      id              INTEGER PRIMARY KEY,
                      chain_id         INTEGER NOT NULL,
                      address            TEXT NOT NULL,
                      token0             TEXT NOT NULL,
                      token1             TEXT NOT NULL,
                      variant            TEXT NOT NULL,
                      fee                INTEGER NOT NULL
                      )",
            [],
        )?;

        Ok(Self {
            erc20_tokens: erc20,
            pools,
        })
    }

    /// Insert a new [ERC20Token] into the database
    pub fn insert_erc20(&self, token: ERC20Token, chain_id: u64) -> Result<(), anyhow::Error> {
        let time = std::time::Instant::now();
        self.erc20_tokens.execute(
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
        println!("Time to insert: {:?}ms", time.elapsed().as_millis());
        Ok(())
    }

    /// Insert a new [Pool] into the database
    pub fn insert_pool(&self, pool: Pool, chain_id: u64) -> Result<(), anyhow::Error> {
        let time = std::time::Instant::now();
        self.pools.execute(
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
        println!("Time to insert: {:?}ms", time.elapsed().as_millis());
        Ok(())
    }

    /// Get the [ERC20Token] from the database
    pub fn get_erc20(&self, address: Address, chain_id: u64) -> Result<Option<ERC20Token>, anyhow::Error> {
        let mut stmt = self.erc20_tokens.prepare("SELECT * FROM ERC20Token WHERE address = ?1, ?2")?;
        let mut rows = stmt.query(params![address.to_string(), chain_id])?;
    
        if let Some(row) = rows.next()? {
           // let chain_id: i32 = row.get(1)?;
            let address: String = row.get(2)?;
            let symbol: String = row.get(3)?;
            let name: String = row.get(4)?;
            let decimals: i32 = row.get(5)?;
            let total_supply: String = row.get(6)?;

            let token = ERC20Token {
                address: address.parse().unwrap(),
                symbol,
                name,
                decimals: decimals as u8,
                total_supply: total_supply.parse().unwrap(),
            };
            
            Ok(Some(token))
        } else {
            Ok(None)
        }
        
    }

    /// Get the [Pool] from the database
    pub fn get_pool(&self, token0: ERC20Token, token1: ERC20Token, chain_id: u64) -> Result<Option<Pool>, anyhow::Error> {
        let token0_addr = token0.address.to_string();
        let token1_addr = token1.address.to_string();
        let mut stmt = self.pools.prepare("SELECT * FROM Pool WHERE chain_id, token0_addr, token1_addr = ?1, ?3, ?4")?;
        let mut rows = stmt.query(params![chain_id, token0_addr, token1_addr])?;
    
        if let Some(row) = rows.next()? {
            let address: String = row.get(2)?;
            let variant: String = row.get(5)?;
            let fee: u32 = row.get(6)?;

            let pool = Pool {
                address: address.parse().unwrap(),
                token0,
                token1,
                variant: PoolVariant::from_u256(variant.parse().unwrap()),
                fee
            };
            
            Ok(Some(pool))
        } else {
            Ok(None)
        }
    }
    
    

}