// src/crypto/secure_memory.rs

use std::fmt;
use zeroize::Zeroize;

/// A string that zeroizes its contents when dropped
#[derive(Clone)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Create a new SecureString
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    /// Create from string slice
    pub fn from_str(s: &str) -> Self {
        Self::new(s.to_string())
    }

    /// Get as str reference
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Convert to String (consumes self)
    pub fn into_string(self) -> String {
        self.inner.clone() //added change: K
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl Zeroize for SecureString {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

impl fmt::Display for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let s = SecureString::from_str("password");
        assert_eq!(s.as_str(), "password");
        assert_eq!(s.len(), 8);
    }

    #[test]
    fn test_debug_redacted() {
        let s = SecureString::from_str("secret");
        let debug = format!("{:?}", s);
        assert!(!debug.contains("secret"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_display_redacted() {
        let s = SecureString::from_str("password");
        let display = format!("{}", s);
        assert!(!display.contains("password"));
        assert!(display.contains("REDACTED"));
    }
}
