use password_hash::{ Output, PasswordHasher};
use argon2::{ Algorithm, Argon2, Params, Version };

use aes_gcm::{ KeyInit, aead::{ Aead, generic_array::GenericArray } };
use chacha20poly1305::{ XChaCha20Poly1305, XNonce };

use sha2::{ Sha256, digest::Digest };
use super::Credentials;
use anyhow::anyhow;

pub const IDENTIFIER: &[u8] = b"params";

/// Memory Cost
pub const M_COST: u32 = 500;

/// Iterations
pub const T_COST: u32 = 200;

/// Parallelism
pub const P_COST: u32 = 1;

/// Hash Length
pub const HASH_LENGTH: usize = 64;


/// Encrypts the given data using the provided credentials
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

/// Decrypts a `profile.data` file using the provided credentials
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
    pub t_cost: u32,
    pub m_cost: u32,
    pub p_cost: u32,
    pub hash_length: usize,
}

impl EncryptionParams {
    pub fn new(argon2: Argon2) -> Result<Self, anyhow::Error> {
        Ok(Self {
            t_cost: argon2.params().t_cost(),
            m_cost: argon2.params().m_cost(),
            p_cost: argon2.params().p_cost(),
            hash_length: argon2.params().output_len().expect("Failed to get output length"),
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.t_cost.to_be_bytes());
        data.extend_from_slice(&self.m_cost.to_be_bytes());
        data.extend_from_slice(&self.p_cost.to_be_bytes());

        // Ensure hash_length is always 8 bytes
        data.extend_from_slice(&(self.hash_length as u64).to_be_bytes());
        data
    }

    /// Recover the params
    pub fn from_vec(data: &[u8]) -> Result<Self, anyhow::Error> {
        if data.len() != 20 {
            return Err(anyhow!("Invalid data length for EncryptionParams"));
        }
        let t_cost = u32::from_be_bytes(data[0..4].try_into().map_err(|_| anyhow!("Failed to convert t_cost from bytes"))?);
        let m_cost = u32::from_be_bytes(data[4..8].try_into().map_err(|_| anyhow!("Failed to convert m_cost from bytes"))?);
        let p_cost = u32::from_be_bytes(data[8..12].try_into().map_err(|_| anyhow!("Failed to convert p_cost from bytes"))?);
        let hash_length = u64::from_be_bytes(data[12..20].try_into().map_err(|_| anyhow!("Failed to convert hash_length from bytes"))?) as usize;

        Ok(Self {
            t_cost,
            m_cost,
            p_cost,
            hash_length,
        })
    }
}



/// Encrypts the given data using the provided credentials
pub fn encrypt(credentials: Credentials, data: Vec<u8>) -> Result<EncryptionResult, anyhow::Error> {
    credentials.is_valid()?;

    // generate a salt needed for the password hashing
    let salt = credentials.generate_saltstring();

    // set the argon2 parameters
    let params = match Params::new(T_COST, M_COST, P_COST, Some(HASH_LENGTH)) {
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
    let argon2_params = EncryptionParams::from_vec(params)?;


    let params = Params::new(
        argon2_params.m_cost,
        argon2_params.t_cost,
        argon2_params.p_cost,
        Some(argon2_params.hash_length)
    ).map_err(|e| anyhow!("Failed to create Argon2 params {:?}", e))?;

    // create the argon2 instance used
    let argon2 = Argon2::new(Algorithm::default(), Version::default(), params.clone());

    // generate the salt needed for the password hashing
    let salt = credentials.generate_saltstring();

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
