use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
pub struct KeypairStore {
    pub private_key: Vec<u8>, // 32 bytes
    pub public_key: Vec<u8>,  // 32 bytes
}

impl KeypairStore {
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        Self {
            private_key: signing_key.to_bytes().to_vec(),
            public_key: signing_key.verifying_key().to_bytes().to_vec(),
        }
    }

    pub fn from_bytes(private: &[u8]) -> Result<Self, String> {
        let key: [u8; 32] = private.try_into().map_err(|_| "Invalid key length")?;
        let signing_key = SigningKey::from_bytes(&key);
        Ok(Self {
            private_key: private.to_vec(),
            public_key: signing_key.verifying_key().to_bytes().to_vec(),
        })
    }
}

/// Generate new Ed25519 keypair
pub fn generate_keypair() -> KeypairStore {
    KeypairStore::generate()
}

/// Sign data with Ed25519
pub fn sign_data(private_key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let key_bytes: [u8; 32] = private_key.try_into().map_err(|_| "Invalid private key")?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    Ok(signing_key.sign(data).to_bytes().to_vec())
}

/// Verify Ed25519 signature
pub fn verify_signature(public_key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool, String> {
    if public_key.len() != 32 || signature.len() != 64 {
        return Err("Invalid key/signature length".into());
    }
    let pk: [u8; 32] = public_key.try_into().unwrap();
    let sig: [u8; 64] = signature.try_into().unwrap();
    let vk = VerifyingKey::from_bytes(&pk).map_err(|e| format!("Invalid public key: {e}"))?;
    let sig = Signature::from_bytes(&sig);
    Ok(vk.verify(data, &sig).is_ok())
}

// ─── AES-256-GCM Encryption ───

const PBKDF2_ITERATIONS: u32 = 100_000;
const NONCE_SIZE: usize = 12;

/// Derive a 256-bit AES key from a password using PBKDF2-SHA256
fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Encrypt data with AES-256-GCM using a password.
/// Returns: [salt (16 bytes)][nonce (12 bytes)][ciphertext]
pub fn aes_encrypt(data: &[u8], password: &str) -> Result<Vec<u8>, String> {
    use rand::RngCore;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let key_bytes = derive_key(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    let mut result = Vec::with_capacity(salt.len() + NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(nonce);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data encrypted with `aes_encrypt`.
/// Input: [salt (16 bytes)][nonce (12 bytes)][ciphertext]
pub fn aes_decrypt(encrypted: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if encrypted.len() < 28 {
        return Err("Invalid encrypted data: too short".into());
    }

    let salt: [u8; 16] = encrypted[..16].try_into().unwrap();
    let nonce = Nonce::from_slice(&encrypted[16..28]);
    let ciphertext = &encrypted[28..];

    let key_bytes = derive_key(password, &salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed — wrong password or corrupted data: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify_roundtrip() {
        let kp = generate_keypair();
        let data = b"collectionloom evidence chain";
        let sig = sign_data(&kp.private_key, data).unwrap();
        assert!(verify_signature(&kp.public_key, data, &sig).unwrap());
    }

    #[test]
    fn test_aes_encrypt_decrypt_roundtrip() {
        let original = b"forensic evidence - sensitive data";
        let password = "MasterKey2024!";
        let encrypted = aes_encrypt(original, password).unwrap();
        let decrypted = aes_decrypt(&encrypted, password).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_aes_wrong_password_fails() {
        let encrypted = aes_encrypt(b"secret", "correct").unwrap();
        assert!(aes_decrypt(&encrypted, "wrong").is_err());
    }

    #[test]
    fn test_aes_different_output_each_time() {
        let pw = "test";
        let e1 = aes_encrypt(b"data", pw).unwrap();
        let e2 = aes_encrypt(b"data", pw).unwrap();
        assert_ne!(e1, e2); // Different salt + nonce each time
        assert_eq!(aes_decrypt(&e1, pw).unwrap(), b"data");
        assert_eq!(aes_decrypt(&e2, pw).unwrap(), b"data");
    }
}
