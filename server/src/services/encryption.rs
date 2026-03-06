use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};

/// Encrypt plaintext using AES-256-GCM.
/// Returns (ciphertext_hex, nonce_hex).
pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<(String, String)> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

    Ok((hex::encode(ciphertext), hex::encode(nonce)))
}

/// Decrypt ciphertext using AES-256-GCM.
pub fn decrypt(ciphertext_hex: &str, nonce_hex: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;

    let ciphertext = hex::decode(ciphertext_hex).context("Invalid ciphertext hex")?;
    let nonce_bytes = hex::decode(nonce_hex).context("Invalid nonce hex")?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow::anyhow!("Decryption failed — key mismatch or corrupted data"))?;

    String::from_utf8(plaintext).context("Decrypted data is not valid UTF-8")
}
