use crate::crypto::SignedMessage;
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn format_verified(name: &str, content: &str) -> String {
    format!("🔐 {name} says (verified): {content}")
}

pub async fn verify_and_display(
    signed_message: &SignedMessage,
    message_sender: &broadcast::Sender<String>,
    crypto_manager: &Arc<crate::crypto::CryptoManager>,
) {
    match crypto_manager.verify_message(signed_message).await {
        Ok(true) => {
            let _ = message_sender.send(format_verified(
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
