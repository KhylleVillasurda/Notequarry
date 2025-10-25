// src/crypto/mod.rs

pub mod encryption;
pub mod key_derivation;
pub mod secure_memory;

// Re-export commonly used items
pub use encryption::{EncryptionError, decrypt, encrypt};
pub use key_derivation::{MasterKey, derive_key, generate_salt};
pub use secure_memory::SecureString;
