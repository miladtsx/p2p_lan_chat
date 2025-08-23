//! Cryptographic operations module for P2P Chat.
//! 
//! This module provides Ed25519 key generation, message signing, and verification
//! to ensure message authenticity and integrity in the peer-to-peer network.

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a cryptographic identity for a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoIdentity {
    /// The public key as bytes
    pub public_key: Vec<u8>,
    /// The peer ID this identity belongs to
    pub peer_id: String,
    /// The peer's display name
    pub name: String,
}

/// A signed message with cryptographic proof of authenticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    /// The original message content
    pub message: String,
    /// The signature of the message
    pub signature: Vec<u8>,
    /// The public key of the signer
    pub public_key: Vec<u8>,
    /// The peer ID of the signer
    pub signer_id: String,
    /// The peer name of the signer
    pub signer_name: String,
    /// Timestamp when the message was signed
    pub timestamp: u64,
}

/// Manages cryptographic operations for a peer
pub struct CryptoManager {
    /// The peer's signing key (private)
    signing_key: SigningKey,
    /// The peer's verifying key (public)
    verifying_key: VerifyingKey,
    /// Cache of known peer public keys
    known_keys: Arc<RwLock<HashMap<String, VerifyingKey>>>,
    /// The peer's own identity
    identity: CryptoIdentity,
}

impl CryptoManager {
    /// Create a new crypto manager with a fresh Ed25519 keypair
    pub fn new(peer_id: String, name: String) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        let identity = CryptoIdentity {
            public_key: verifying_key.to_bytes().to_vec(),
            peer_id: peer_id.clone(),
            name,
        };

        Self {
            signing_key,
            verifying_key,
            known_keys: Arc::new(RwLock::new(HashMap::new())),
            identity,
        }
    }

    /// Get the peer's public identity
    pub fn get_identity(&self) -> &CryptoIdentity {
        &self.identity
    }

    /// Get the peer's public key as bytes
    pub fn get_public_key(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    /// Sign a message with the peer's private key
    pub fn sign_message(&self, message: &str, timestamp: u64) -> Result<SignedMessage, CryptoError> {
        // Create a message to sign that includes timestamp to prevent replay attacks
        let message_to_sign = format!("{message}:{timestamp}");
        let signature = self.signing_key.sign(message_to_sign.as_bytes());
        
        Ok(SignedMessage {
            message: message.to_string(),
            signature: signature.to_bytes().to_vec(),
            public_key: self.verifying_key.to_bytes().to_vec(),
            signer_id: self.identity.peer_id.clone(),
            signer_name: self.identity.name.clone(),
            timestamp,
        })
    }

    /// Verify a signed message
    pub async fn verify_message(&self, signed_msg: &SignedMessage) -> Result<bool, CryptoError> {
        // Check if we know the signer's public key
        let verifying_key = {
            let known_keys = self.known_keys.read().await;
            if let Some(key) = known_keys.get(&signed_msg.signer_id) {
                *key
            } else {
                // Try to reconstruct the key from the message
                let public_key_array: [u8; 32] = signed_msg.public_key.as_slice()
                    .try_into()
                    .map_err(|_| CryptoError::InvalidPublicKey)?;
                let key = VerifyingKey::from_bytes(&public_key_array)
                    .map_err(|_| CryptoError::InvalidPublicKey)?;
                
                // Cache this key for future use
                drop(known_keys); // Release read lock
                self.known_keys.write().await.insert(
                    signed_msg.signer_id.clone(),
                    key,
                );
                
                key
            }
        };

        // Reconstruct the message that was signed
        let message_to_verify = format!("{}:{}", signed_msg.message, signed_msg.timestamp);
        
        // Convert signature bytes back to Signature
        let signature_array: [u8; 64] = signed_msg.signature.as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;
        let signature = Signature::from_bytes(&signature_array);

        // Verify the signature
        Ok(verifying_key.verify(message_to_verify.as_bytes(), &signature).is_ok())
    }

    /// Add a known peer's public key to the cache
    pub async fn add_known_peer(&self, peer_id: String, public_key: Vec<u8>) -> Result<(), CryptoError> {
        let public_key_array: [u8; 32] = public_key.as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_array)
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        
        self.known_keys.write().await.insert(peer_id, verifying_key);
        Ok(())
    }

    /// Check if a message is recent (within a reasonable time window)
    pub fn is_message_recent(&self, timestamp: u64, max_age_seconds: u64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        current_time.saturating_sub(timestamp) <= max_age_seconds
    }

    /// Get the number of known peer keys
    pub async fn known_peers_count(&self) -> usize {
        self.known_keys.read().await.len()
    }
}

/// Errors that can occur during cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid public key format")]
    InvalidPublicKey,
    #[error("Invalid signature format")]
    InvalidSignature,
    #[error("Message verification failed")]
    VerificationFailed,
    #[error("Message is too old")]
    MessageTooOld,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::Unknown(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_crypto_manager_creation() {
        let manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
        assert_eq!(manager.identity.peer_id, "test-peer");
        assert_eq!(manager.identity.name, "TestPeer");
        assert!(!manager.get_public_key().is_empty());
    }

    #[test]
    fn test_message_signing_and_verification() {
        let manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
        let message = "Hello, world!";
        let timestamp = 1234567890;
        
        let signed_msg = manager.sign_message(message, timestamp).unwrap();
        assert_eq!(signed_msg.message, message);
        assert_eq!(signed_msg.timestamp, timestamp);
        assert!(!signed_msg.signature.is_empty());
        
        // Test verification
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let is_valid = manager.verify_message(&signed_msg).await.unwrap();
            assert!(is_valid);
        });
    }

    #[test]
    fn test_message_tampering_detection() {
        let manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
        let message = "Hello, world!";
        let timestamp = 1234567890;
        
        let mut signed_msg = manager.sign_message(message, timestamp).unwrap();
        signed_msg.message = "Hello, tampered!".to_string();
        
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let is_valid = manager.verify_message(&signed_msg).await.unwrap();
            assert!(!is_valid);
        });
    }

    #[test]
    fn test_message_age_validation() {
        let manager = CryptoManager::new("test-peer".to_string(), "TestPeer".to_string());
        
        // Old message
        let old_timestamp = 1234567890;
        assert!(!manager.is_message_recent(old_timestamp, 3600)); // 1 hour max age
        
        // Recent message
        let recent_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(manager.is_message_recent(recent_timestamp, 3600));
    }
}
