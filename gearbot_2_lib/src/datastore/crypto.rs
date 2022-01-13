use std::borrow::Cow;
use std::fmt;

use aes_gcm::aead::generic_array::{typenum::U32, GenericArray};
use aes_gcm::{
    aead::{Aead, NewAead},
    Aes256Gcm,
};
use rand::{thread_rng, RngCore};

/// An encryption key that is used to secure guild data.
pub struct EncryptionKey<'a>(Cow<'a, GenericArray<u8, U32>>);

impl<'a> EncryptionKey<'a> {
    /// Constructs a new `EncryptionKey` from some bytes,
    /// returning a key tied to the raw values.
    pub fn construct_borrowed(bytes: &'a [u8]) -> Self {
        Self(Cow::Borrowed(GenericArray::from_slice(bytes)))
    }

    /// Constructs a new `EncryptionKey` from some bytes,
    /// returning a key that is free of ownership requirements.
    pub fn construct_owned(bytes: &[u8]) -> Self {
        Self(Cow::Owned(GenericArray::clone_from_slice(bytes)))
    }
}

impl fmt::Debug for EncryptionKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("EncryptionKey")
    }
}

pub fn generate_guild_encryption_key(main_encryption_key: &EncryptionKey, guild_id: u64) -> Vec<u8> {
    let mut csprng = thread_rng();
    let mut guild_encryption_key = [0u8; 32];
    csprng.fill_bytes(&mut guild_encryption_key);

    encrypt_bytes(&guild_encryption_key, main_encryption_key, guild_id)[..32].to_vec()
}

pub fn encrypt_bytes(plaintext: &[u8], key: &EncryptionKey, msg_id: u64) -> Vec<u8> {
    let aead = Aes256Gcm::new(&key.0);

    // Since nonce's only never need to be reused, and Discor's snowflakes for messages
    // are unique, we can use the message id to construct the nonce with its 64 bits, and then
    // pad the rest with zeros.
    let mut nonce_bytes = [0u8; 12];
    let msg_id_bytes = msg_id.to_le_bytes();
    nonce_bytes[..8].copy_from_slice(&msg_id_bytes);
    nonce_bytes[8..].copy_from_slice(&[0u8; 4]);

    let nonce = GenericArray::from_slice(&nonce_bytes);

    aead.encrypt(&nonce, plaintext).expect("Failed to encrypt an object!")
}

pub fn decrypt_bytes(ciphertext: &[u8], key: &EncryptionKey, msg_id: u64) -> Vec<u8> {
    let aead = Aes256Gcm::new(&key.0);

    let mut nonce_bytes = [0u8; 12];
    let msg_id_bytes = msg_id.to_le_bytes();
    nonce_bytes[..8].copy_from_slice(&msg_id_bytes);
    nonce_bytes[8..].copy_from_slice(&[0u8; 4]);

    let nonce = GenericArray::from_slice(&nonce_bytes);

    aead.decrypt(&nonce, ciphertext).expect("Failed to decrypt an object!")
}
