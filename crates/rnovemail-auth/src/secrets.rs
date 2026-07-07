use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::AuthError;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EncryptedSecret {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

pub struct SecretBox {
    key: [u8; 32],
}

impl SecretBox {
    pub fn new(mut key: [u8; 32]) -> Self {
        let box_key = key;
        key.zeroize();
        Self { key: box_key }
    }

    pub fn encrypt(&self, secret: &SecretString) -> Result<EncryptedSecret, AuthError> {
        let nonce_bytes = random_nonce();
        let cipher = cipher(&self.key);
        let encrypted = cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                secret.expose_secret().as_bytes(),
            )
            .map_err(|_| AuthError::SecretEncryption)?;
        Ok(EncryptedSecret {
            nonce: nonce_bytes.to_vec(),
            ciphertext: encrypted,
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedSecret) -> Result<SecretString, AuthError> {
        let cipher = cipher(&self.key);
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&encrypted.nonce),
                encrypted.ciphertext.as_ref(),
            )
            .map_err(|_| AuthError::SecretDecryption)?;
        String::from_utf8(plaintext)
            .map(SecretString::new)
            .map_err(|_| AuthError::SecretDecryption)
    }
}

fn cipher(key: &[u8; 32]) -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new(Key::from_slice(key))
}

fn random_nonce() -> [u8; 12] {
    let mut nonce = [0_u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    nonce
}
