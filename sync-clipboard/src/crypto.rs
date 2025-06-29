use aes_gcm::{AeadCore,
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm,
    Nonce, // Or `Aes128Gcm`
};
use sha2::{Digest, Sha256};

// We need a key of a specific size. We'll use SHA-256 to hash the
// user-provided key into a 32-byte key.
pub fn key_from_password(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(key.into());
    // The nonce must be unique for every encryption with the same key.
    // We'll generate a random nonce, and prepend it to the ciphertext.
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data)?; // NOTE: handle this error properly in production code!
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new(key.into());
    // The nonce is prepended to the ciphertext.
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext)
}
