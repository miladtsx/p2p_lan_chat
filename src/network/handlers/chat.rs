//! Chat helper functions to handle chat messages
//!
//! This module is responsible for managing chat messages, including
//! verifying signatures and broadcasting messages to peers.

use crate::crypto::SignedMessage;
use crate::peer::Message;
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn handle_chat_message(
    message: Message,
    message_sender: &broadcast::Sender<String>,
    crypto_manager: &Arc<crate::crypto::CryptoManager>,
) {
    // Check if message has cryptographic signature
    if let (Some(signature), Some(public_key)) = (&message.signature, &message.public_key) {
        // Verify the signature if we have crypto capabilities
        println!(
            "🔍 Verifying message from {} with signature length: {}",
            message.from_name,
            signature.len()
        );

        let signed_msg = &SignedMessage {
            message: message.content.clone(),
            signature: signature.clone(),
            public_key: public_key.clone(),
            signer_id: message.from_id.clone(),
            signer_name: message.from_name.clone(),
            timestamp: message.timestamp,
        };

        _verify_and_display(signed_msg, message_sender, crypto_manager).await;
    } else {
        // No crypto manager, display as unsigned message
        let display_msg = format!(
            "📝 {} says (unsigned): {}",
            message.from_name, message.content
        );
        let _ = message_sender.send(display_msg);
    }
}

pub async fn handle_signed_chat(
    signed_message: SignedMessage,
    message_sender: &broadcast::Sender<String>,
    crypto_manager: &Arc<crate::crypto::CryptoManager>,
) {
    {
        _verify_and_display(&signed_message, message_sender, crypto_manager).await;
    }
}

fn _format_verified(name: &str, content: &str) -> String {
    format!("🔐 {name} says (verified): {content}")
}

async fn _verify_and_display(
    signed_message: &SignedMessage,
    message_sender: &broadcast::Sender<String>,
    crypto_manager: &Arc<crate::crypto::CryptoManager>,
) {
    match crypto_manager.verify_message(signed_message).await {
        Ok(true) => {
            let _ = message_sender.send(_format_verified(
                &signed_message.signer_name,
                &signed_message.message,
            ));
        }
        Ok(false) => {
            let _ = message_sender.send(format!(
                "⚠️  {} says (INVALID SIGNATURE): {}",
                signed_message.signer_name, signed_message.message
            ));
        }
        Err(e) => {
            let _ = message_sender.send(format!(
                "❓ {} says (verification failed: {}): {}",
                signed_message.signer_name, e, signed_message.message
            ));
        }
    }
}
