use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

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
    let vk = VerifyingKey::from_bytes(&pk).map_err(|e| format!("Invalid public key: {}", e))?;
    let sig = Signature::from_bytes(&sig);
    Ok(vk.verify(data, &sig).is_ok())
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
}
