//! Secure memory handling for sensitive data
//!
//! Provides types that securely zero memory on drop to prevent
//! sensitive data from lingering in memory.

use std::fmt;
use std::ops::Deref;

/// A string type that zeros its contents on drop
///
/// Use this for passphrases and other sensitive string data.
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Create a new SecureString
    pub fn new(s: impl Into<String>) -> Self {
        Self { inner: s.into() }
    }

    /// Get the string contents
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero out the string's memory
        // SAFETY: We're modifying the bytes in place before the String is dropped
        // The string might be on the heap, so we need to zero those bytes
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            for byte in bytes.iter_mut() {
                std::ptr::write_volatile(byte, 0);
            }
        }
        // Clear the string (this might reallocate, but the important thing
        // is we've zeroed the original memory)
        self.inner.clear();
    }
}

impl Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for SecureString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

// Don't print the contents in Debug output
impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureString")
            .field("len", &self.inner.len())
            .finish()
    }
}

// Don't print the contents in Display output
impl fmt::Display for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED {} bytes]", self.inner.len())
    }
}

/// A byte vector that zeros its contents on drop
///
/// Use this for encryption keys and other sensitive binary data.
pub struct SecureBytes {
    inner: Vec<u8>,
}

impl SecureBytes {
    /// Create new SecureBytes
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            inner: bytes.into(),
        }
    }

    /// Create SecureBytes with a specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Get mutable bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Drop for SecureBytes {
    fn drop(&mut self) {
        // Zero out the memory
        for byte in self.inner.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        self.inner.clear();
    }
}

impl Deref for SecureBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<[u8]> for SecureBytes {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<Vec<u8>> for SecureBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl From<&[u8]> for SecureBytes {
    fn from(bytes: &[u8]) -> Self {
        Self::new(bytes.to_vec())
    }
}

// Don't print the contents in Debug output
impl fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureBytes")
            .field("len", &self.inner.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_creation() {
        let s = SecureString::new("test");
        assert_eq!(s.as_str(), "test");
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn test_secure_string_from_string() {
        let s: SecureString = String::from("test").into();
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn test_secure_string_from_str() {
        let s: SecureString = "test".into();
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn test_secure_string_deref() {
        let s = SecureString::new("test");
        let len = s.len(); // Uses Deref to &str
        assert_eq!(len, 4);
    }

    #[test]
    fn test_secure_string_debug() {
        let s = SecureString::new("secret");
        let debug = format!("{:?}", s);
        assert!(!debug.contains("secret"));
        assert!(debug.contains("SecureString"));
    }

    #[test]
    fn test_secure_string_display() {
        let s = SecureString::new("secret");
        let display = format!("{}", s);
        assert!(!display.contains("secret"));
        assert!(display.contains("REDACTED"));
    }

    #[test]
    fn test_secure_bytes_creation() {
        let b = SecureBytes::new(vec![1, 2, 3]);
        assert_eq!(b.as_bytes(), &[1, 2, 3]);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_secure_bytes_from_vec() {
        let b: SecureBytes = vec![1, 2, 3].into();
        assert_eq!(b.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_secure_bytes_from_slice() {
        let b: SecureBytes = (&[1u8, 2, 3][..]).into();
        assert_eq!(b.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_secure_bytes_debug() {
        let b = SecureBytes::new(vec![1, 2, 3, 4, 5]);
        let debug = format!("{:?}", b);
        assert!(debug.contains("SecureBytes"));
        assert!(debug.contains("5")); // length
    }
}
