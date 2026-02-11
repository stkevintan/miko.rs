use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};

fn ensure_key_32(key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(key);
    let result = hasher.finalize();
    let mut key_32 = [0u8; 32];
    key_32.copy_from_slice(&result);
    key_32
}

pub fn encrypt(plain_text: &str, key: &[u8]) -> Result<String> {
    if key.is_empty() {
        return Err(anyhow!("encryption key is empty"));
    }
    let key_32 = ensure_key_32(key);
    let cipher = Aes256Gcm::new_from_slice(&key_32).map_err(|e| anyhow!(e.to_string()))?;

    // In Go version, nonce is prepended to ciphertext
    // nonce := make([]byte, gcm.NonceSize())
    // ciphertext := gcm.Seal(nonce, nonce, []byte(plainText), nil)

    let mut nonce_bytes = [0u8; 12]; // AES-GCM standard nonce size is 12 bytes
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| anyhow!(e.to_string()))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plain_text.as_bytes())
        .map_err(|e| anyhow!(e.to_string()))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(crypto_text: &str, key: &[u8]) -> Result<String> {
    if key.is_empty() {
        return Err(anyhow!("decryption key is empty"));
    }
    let data = general_purpose::STANDARD.decode(crypto_text)?;

    let key_32 = ensure_key_32(key);
    let cipher = Aes256Gcm::new_from_slice(&key_32).map_err(|e| anyhow!(e.to_string()))?;

    if data.len() < 12 {
        return Err(anyhow!("ciphertext too short"));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(String::from_utf8(decrypted)?)
}
