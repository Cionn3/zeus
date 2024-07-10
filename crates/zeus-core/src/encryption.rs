use password_hash::{ Output, PasswordHasher, SaltString};
use argon2::{ Algorithm, Argon2, Params, Version };

use aes_gcm::{ KeyInit, aead::{ Aead, generic_array::GenericArray } };
use chacha20poly1305::{ XChaCha20Poly1305, XNonce };

use sha2::{ Sha256, digest::Digest };
use anyhow::anyhow;

/// The identifier used to find the Argon2 params that was used to encrypt the data
pub const IDENTIFIER: &[u8] = b"params";

// * Argon2 Parameters

/// Memory Cost
pub const M_COST: u32 = 500;

/// Iterations
pub const T_COST: u32 = 200;

/// Parallelism
pub const P_COST: u32 = 1;

/// Hash Length
pub const HASH_LENGTH: usize = 64;


/// The credentials needed to encrypt and decrypt an encrypted file
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub confrim_password: String,
}


impl Credentials {
    /// Salt for Argon2
    fn generate_saltstring(&self) -> Result<SaltString, anyhow::Error> {
        let salt_array = Sha256::digest(self.username.as_bytes());
        let salt = salt_array.to_vec();
        let salt = salt.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        let salt = SaltString::from_b64(&salt).map_err(|e| anyhow!("Failed to generate salt string {:?}", e))?;
        Ok(salt)
    }

    fn is_valid(&self) -> Result<(), anyhow::Error> {
        if self.username.is_empty() || self.password.is_empty() || self.confrim_password.is_empty() {
            return Err(anyhow!("Username and Password must be provided"));
        }

        if self.password != self.confrim_password {
            return Err(anyhow!("Passwords do not match"));
        }

        Ok(())
    }
}


/// Encrypts the given data using the provided credentials
/// 
/// ### Arguments
/// 
/// - `file` - The file to write the encrypted data to (eg. `profile.data`)
/// - `data` - The data to encrypt
/// - `credentials` - The credentials to use for encryption
/// 
/// The encrypted data file is written in the same directory as the executable
pub fn encrypt_data(file: &str, data: Vec<u8>, credentials: Credentials) -> Result<(), anyhow::Error> {
    let encrypted = encrypt(credentials, data)?;

    let params_with_identifier = [IDENTIFIER, encrypted.params.to_vec().as_slice()].concat();

    let encrypted_data_with_params = [
        encrypted.data.as_slice(),
        params_with_identifier.as_slice(),
    ].concat();

    std::fs::write(file, encrypted_data_with_params)?;
    Ok(())
}

/// Decrypts a `file` using the provided credentials
/// 
/// Please note that the `file` should be in the same directory as the executable
/// 
/// ### Arguments
/// 
/// - `file` - The file to read the encrypted data from (eg. `profile.data`)
/// - `credentials` - The credentials to use for decryption
/// 
/// ### Returns
/// 
/// The decrypted data as a `Vec<u8>`
/// 
/// The decrypted data stays in memory and is not written to disk
pub fn decrypt_data(file: &str, credentials: Credentials) -> Result<Vec<u8>, anyhow::Error> {
    let data = std::fs::read(file)?;
    let decrypted_data = decrypt(credentials, data)?;
    Ok(decrypted_data)
}


pub struct EncryptionResult {
    /// The encrypted data
    pub data: Vec<u8>,

    /// Argon2 Params used for the encryption
    pub params: EncryptionParams,
}

/// The parameters used to encrypt the data
#[derive(Clone, Debug)]
pub struct EncryptionParams {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
    pub hash_length: usize,
}

impl EncryptionParams {
    pub fn new(argon2: Argon2) -> Result<Self, anyhow::Error> {
        let hash_lenght = argon2.params().output_len();

        if hash_lenght.is_none() {
            return Err(anyhow!("Failed to get output length"));
        }
        
        Ok(Self {
            m_cost: argon2.params().m_cost(),
            t_cost: argon2.params().t_cost(),
            p_cost: argon2.params().p_cost(),
            hash_length: hash_lenght.expect("Failed to get output length"),
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.m_cost.to_be_bytes());
        data.extend_from_slice(&self.t_cost.to_be_bytes());
        data.extend_from_slice(&self.p_cost.to_be_bytes());

        // Ensure hash_length is always 8 bytes
        data.extend_from_slice(&(self.hash_length as u64).to_be_bytes());
        data
    }

    /// Recover the params
    pub fn from_u8(data: &[u8]) -> Result<Self, anyhow::Error> {
        if data.len() != 20 {
            return Err(anyhow!("Invalid data length for EncryptionParams"));
        }
        let m_cost = u32::from_be_bytes(data[0..4].try_into().map_err(|_| anyhow!("Failed to convert m_cost from bytes"))?);
        let t_cost = u32::from_be_bytes(data[4..8].try_into().map_err(|_| anyhow!("Failed to convert t_cost from bytes"))?);
        let p_cost = u32::from_be_bytes(data[8..12].try_into().map_err(|_| anyhow!("Failed to convert p_cost from bytes"))?);
        let hash_length = u64::from_be_bytes(data[12..20].try_into().map_err(|_| anyhow!("Failed to convert hash_length from bytes"))?) as usize;

        Ok(Self {
            m_cost,
            t_cost,
            p_cost,
            hash_length,
        })
    }
}



/// Encrypts the given data using the provided credentials
pub fn encrypt(credentials: Credentials, data: Vec<u8>) -> Result<EncryptionResult, anyhow::Error> {
    credentials.is_valid()?;

    // generate a salt needed for the password hashing
    let salt = credentials.generate_saltstring()?;

    // set the argon2 parameters
    let params = match Params::new(M_COST, T_COST, P_COST, Some(HASH_LENGTH)) {
        Ok(params) => params,
        Err(e) => {
            return Err(anyhow::Error::msg(format!("{:?}", e)));
        }
    };

    let argon2 = Argon2::new(Algorithm::default(), Version::default(), params.clone());

    // hash the password
    let password_hash = match argon2.hash_password(credentials.password.as_bytes(), &salt) {
        Ok(hash) => hash,
        Err(e) => {
            return Err(anyhow!("Failed to hash password {:?}", e));
        }
    };

    // get the hash output
    let key = password_hash.hash.ok_or(anyhow!("Failed to get the hash output"))?;

    // create the cipher using the hashed password as the key
    let cipher = xchacha20_poly_1305(key);

    // use the SHA-256 hash of the username as the nonce
    // ! usually this is a random value but since the username is not saved anywhere it should be safe
    let hash = Sha256::digest(credentials.username.as_bytes());
    let nonce = XNonce::from_slice(&hash.as_slice()[..24]);

    let encrypted_data = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| anyhow!("Failed to encrypt data {:?}", e))?;

    Ok(EncryptionResult {
        data: encrypted_data,
        params: EncryptionParams::new(argon2)?,
    })
}

/// Decrypts the given data using the provided credentials
pub fn decrypt(credentials: Credentials, data: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    credentials.is_valid()?;

    // find the argon2 params in the encrypted data
    let identifier_position = find_identifier_position(&data, IDENTIFIER).ok_or(
        anyhow!("Failed to find the identifier in the encrypted data")
    )?;

    // get the argon2 params from the encrypted data
    let (encrypted_data, identifier_data) = data.split_at(identifier_position);
    let params = &identifier_data[IDENTIFIER.len()..];


    // Parse the encryption params
    let argon2_params = EncryptionParams::from_u8(params)?;


    let params = Params::new(
        argon2_params.m_cost,
        argon2_params.t_cost,
        argon2_params.p_cost,
        Some(argon2_params.hash_length)
    ).map_err(|e| anyhow!("Failed to create Argon2 params {:?}", e))?;

    // create the argon2 instance used
    let argon2 = Argon2::new(Algorithm::default(), Version::default(), params.clone());

    // generate the salt needed for the password hashing
    let salt = credentials.generate_saltstring()?;

    // hash the password
    let password_hash = argon2
        .hash_password(credentials.password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Failed to hash password {:?}", e))?;

    // get the hash output
    let key = password_hash.hash.ok_or(anyhow!("Failed to get the hash output"))?;

    // create the cipher using the hashed password as the key
    let cipher = xchacha20_poly_1305(key);

    let hash = Sha256::digest(credentials.username.as_bytes());
    let nonce = XNonce::from_slice(&hash.as_slice()[..24]);

    let decrypted_data = cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| anyhow!("Failed to decrypt data {:?}", e))?;

    Ok(decrypted_data)
}

fn xchacha20_poly_1305(key: Output) -> XChaCha20Poly1305 {
    let key = GenericArray::from_slice(&key.as_bytes()[..32]);
    XChaCha20Poly1305::new(key)
}

/// Finds the position of the [IDENTIFIER] in the encrypted data
fn find_identifier_position(data: &[u8], identifier: &[u8]) -> Option<usize> {
    data.windows(identifier.len()).rposition(|window| window == identifier)
}
