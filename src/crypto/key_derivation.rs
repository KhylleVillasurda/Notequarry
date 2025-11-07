// src/crypto/key_derivation.rs

use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use log::info;
use rand::rngs::OsRng;
use zeroize::Zeroize;

/// Master encryption key derived from password
#[derive(Clone)]
pub struct MasterKey {
    key: [u8; 32], // 256-bit key
}

impl MasterKey {
    /// Create a new MasterKey from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { key: bytes }
    }

    /// Get key as bytes reference
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }

    /// Get key as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.key
    }
}

impl Drop for MasterKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl Zeroize for MasterKey {
    fn zeroize(&mut self) {
        self.key.zeroize();
    }
}

/// Generate a cryptographically secure random salt (16 bytes)
pub fn generate_salt() -> Vec<u8> {
    let salt = SaltString::generate(&mut OsRng);
    salt.as_str().as_bytes().to_vec()
}

/// Derive an encryption key from a password using Argon2id
pub fn derive_key(password: &str, salt: &[u8]) -> Result<MasterKey, String> {
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    if salt.len() < 16 {
        return Err("Salt must be at least 16 bytes".to_string());
    }

    info!("Deriving encryption key with Argon2id...");

    // Argon2id parameters: 64 MB memory, 3 iterations, 4 threads
    let params = Params::new(65536, 3, 4, Some(32)).map_err(|e| format!("Params error: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // Create SaltString from bytes
    let salt_str = std::str::from_utf8(salt).map_err(|e| format!("Invalid salt UTF-8: {}", e))?;
    let salt_string =
        SaltString::from_b64(salt_str).map_err(|e| format!("Invalid salt format: {}", e))?;

    // Hash password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| format!("Hash failed: {}", e))?;

    // Extract key bytes
    let hash = password_hash
        .hash
        .ok_or_else(|| "No hash produced".to_string())?;

    let hash_bytes = hash.as_bytes();
    if hash_bytes.len() != 32 {
        return Err(format!("Expected 32 bytes, got {}", hash_bytes.len()));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(hash_bytes);

    info!("Key derivation successful");
    Ok(MasterKey::from_bytes(key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert!(salt1.len() >= 16);
        assert!(salt2.len() >= 16);
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_derive_key() {
        let salt = generate_salt();
        let key = derive_key("test_password", &salt).unwrap();
        assert_eq!(key.as_slice().len(), 32);
    }

    #[test]
    fn test_same_password_same_key() {
        let salt = generate_salt();
        let key1 = derive_key("password", &salt).unwrap();
        let key2 = derive_key("password", &salt).unwrap();
        assert_eq!(key1.as_slice(), key2.as_slice());
    }

    #[test]
    fn test_different_password_different_key() {
        let salt = generate_salt();
        let key1 = derive_key("password1", &salt).unwrap();
        let key2 = derive_key("password2", &salt).unwrap();
        assert_ne!(key1.as_slice(), key2.as_slice());
    }

    #[test]
    fn test_empty_password() {
        let salt = generate_salt();
        assert!(derive_key("", &salt).is_err());
    }
}
