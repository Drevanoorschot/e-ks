use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit},
};
use rand::{Rng};
use secrecy::{ExposeSecret, SecretBox};

use crate::{AppError, constants::EVENT_ENCRYPTION_KEY_LEN};

const NONCE_LEN: usize = 12;

pub struct EventCrypto {
    key: SecretBox<Vec<u8>>,
}

impl EventCrypto {
    pub fn new(key: SecretBox<Vec<u8>>) -> Result<Self, AppError> {
        if key.expose_secret().len() != EVENT_ENCRYPTION_KEY_LEN {
            return Err(AppError::ConfigLoadError(format!(
                "EVENT_ENCRYPTION_KEY must be {EVENT_ENCRYPTION_KEY_LEN} bytes"
            )));
        }

        Ok(Self { key })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
        let cipher = self.cipher()?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce_bytes);

        let ciphertext = cipher
            .encrypt((&nonce_bytes).into(), plaintext)
            .map_err(|_| AppError::InternalServerError)?;

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    pub fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>, AppError> {
        if payload.len() < NONCE_LEN {
            return Err(AppError::IntegrityViolation);
        }

        let (nonce, ciphertext) = payload.split_at(NONCE_LEN);
        let cipher = self.cipher()?;
        cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|_| AppError::IntegrityViolation)
    }

    fn cipher(&self) -> Result<Aes256Gcm, AppError> {
        Aes256Gcm::new_from_slice(self.key.expose_secret())
            .map_err(|_| AppError::ConfigLoadError("Invalid event encryption key".to_string()))
    }
}
