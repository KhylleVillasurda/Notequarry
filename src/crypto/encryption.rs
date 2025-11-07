use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use log::info;
use rand::RngCore;

use super::key_derivation::MasterKey;

/// Encryption error type
#[derive(Debug)]
pub enum EncryptionError {
    EncryptFailed(String),
    DecryptFailed(String),
    InvalidFormat,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EncryptionError::EncryptFailed(msg) => write!(f, "Encryption failed: {}", msg),
            EncryptionError::DecryptFailed(msg) => write!(f, "Decryption failed: {}", msg),
            EncryptionError::InvalidFormat => write!(f, "Invalid ciphertext format"),
        }
    }
}

impl std::error::Error for EncryptionError {}

const NONCE_SIZE: usize = 12;

/// Encrypt plaintext using ChaCha20-Poly1305
/// Format: [nonce (12 bytes)] + [ciphertext + tag]
pub fn encrypt(plaintext: &str, key: &MasterKey) -> Result<Vec<u8>, EncryptionError> {
    info!("Encrypting data...");

    // Create key from slice
    let cipher_key = Key::clone_from_slice(key.as_slice());
    let cipher = ChaCha20Poly1305::new(&cipher_key);

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    // Create nonce from slice
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| EncryptionError::EncryptFailed(e.to_string()))?;

    // Prepend nonce
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    info!("Encryption successful ({} bytes)", result.len());
    Ok(result)
}

/// Decrypt ciphertext using ChaCha20-Poly1305
pub fn decrypt(ciphertext: &[u8], key: &MasterKey) -> Result<String, EncryptionError> {
    info!("Decrypting data...");

    if ciphertext.len() < NONCE_SIZE + 16 {
        return Err(EncryptionError::InvalidFormat);
    }

    // Split nonce and encrypted data
    let (nonce_bytes, encrypted_data) = ciphertext.split_at(NONCE_SIZE);
    // Create nonce from slice
    let nonce = Nonce::from_slice(nonce_bytes);

    // Create key from slice
    let cipher_key = Key::from_slice(key.as_slice());
    let cipher = ChaCha20Poly1305::new(&cipher_key);

    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(&nonce, encrypted_data)
        .map_err(|e| EncryptionError::DecryptFailed(e.to_string()))?;

    // Convert to string
    let plaintext = String::from_utf8(plaintext_bytes)
        .map_err(|e| EncryptionError::DecryptFailed(format!("Invalid UTF-8: {}", e)))?;

    info!("Decryption successful ({} bytes)", plaintext.len());
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{derive_key, generate_salt};

    #[test]
    fn test_encrypt_decrypt() {
        let salt = generate_salt();
        let key = derive_key("test_password", &salt).unwrap();

        let plaintext = "Hello, World!";
        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_ciphertexts() {
        let salt = generate_salt();
        let key = derive_key("password", &salt).unwrap();

        let plaintext = "Same message";
        let ct1 = encrypt(plaintext, &key).unwrap();
        let ct2 = encrypt(plaintext, &key).unwrap();

        assert_ne!(ct1, ct2); // Different nonces
        assert_eq!(decrypt(&ct1, &key).unwrap(), plaintext);
        assert_eq!(decrypt(&ct2, &key).unwrap(), plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let salt = generate_salt();
        let key1 = derive_key("password1", &salt).unwrap();
        let key2 = derive_key("password2", &salt).unwrap();

        let plaintext = "Secret";
        let ciphertext = encrypt(plaintext, &key1).unwrap();

        assert!(decrypt(&ciphertext, &key2).is_err());
    }

    #[test]
    fn test_tampered_data_fails() {
        let salt = generate_salt();
        let key = derive_key("password", &salt).unwrap();

        let plaintext = "Original";
        let mut ciphertext = encrypt(plaintext, &key).unwrap();

        // Tamper with data
        ciphertext[NONCE_SIZE] ^= 0xFF;

        assert!(decrypt(&ciphertext, &key).is_err());
    }

    #[test]
    fn test_unicode() {
        let salt = generate_salt();
        let key = derive_key("password", &salt).unwrap();

        let plaintext = "Hello 🌍";
        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
