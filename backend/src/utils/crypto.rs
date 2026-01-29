use aes::Aes256;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cbc::{Decryptor, Encryptor};
use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use subtle::ConstantTimeEq;

const SALT_LEN: usize = 16;
const IV_LEN: usize = 16;
const HMAC_LEN: usize = 32;
const KEY_LEN: usize = 32;
const DERIVED_LEN: usize = 64;
const PBKDF2_ITERS: u32 = 100_000;

pub const ENCRYPTION_PREFIX: &str = "v1:";

pub fn encrypt_with_password(plaintext: &str, password: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut iv);

    let mut derived = [0u8; DERIVED_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERS, &mut derived);
    let (enc_key, mac_key) = derived.split_at(KEY_LEN);

    let cipher =
        Encryptor::<Aes256>::new_from_slices(enc_key, &iv).map_err(|_| "invalid key")?;
    let mut buffer = plaintext.as_bytes().to_vec();
    let pos = buffer.len();
    let block_size = 16;
    let pad_len = block_size - (pos % block_size);
    buffer.resize(pos + pad_len, 0);
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| "encryption failed")?
        .to_vec();

    let mut data = Vec::with_capacity(SALT_LEN + IV_LEN + ciphertext.len() + HMAC_LEN);
    data.extend_from_slice(&salt);
    data.extend_from_slice(&iv);
    data.extend_from_slice(&ciphertext);

    let mut mac = Hmac::<Sha256>::new_from_slice(mac_key).map_err(|_| "invalid mac key")?;
    mac.update(&data);
    let tag = mac.finalize().into_bytes();
    data.extend_from_slice(&tag);

    Ok(format!("{}{}", ENCRYPTION_PREFIX, STANDARD.encode(data)))
}

pub fn decrypt_with_password(value: &str, password: &str) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }

    let encoded = value
        .strip_prefix(ENCRYPTION_PREFIX)
        .ok_or_else(|| "unsupported ciphertext".to_string())?;
    let data = STANDARD
        .decode(encoded)
        .map_err(|_| "invalid ciphertext".to_string())?;
    if data.len() <= SALT_LEN + IV_LEN + HMAC_LEN {
        return Err("invalid ciphertext".to_string());
    }

    let tag_index = data.len() - HMAC_LEN;
    let (payload, tag) = data.split_at(tag_index);
    let salt = &payload[..SALT_LEN];
    let iv = &payload[SALT_LEN..SALT_LEN + IV_LEN];
    let ciphertext = &payload[SALT_LEN + IV_LEN..];

    let mut derived = [0u8; DERIVED_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERS, &mut derived);
    let (enc_key, mac_key) = derived.split_at(KEY_LEN);

    let mut mac = Hmac::<Sha256>::new_from_slice(mac_key).map_err(|_| "invalid mac key")?;
    mac.update(payload);
    let expected = mac.finalize().into_bytes();
    if expected.ct_eq(tag).unwrap_u8() != 1 {
        return Err("invalid ciphertext".to_string());
    }

    let cipher =
        Decryptor::<Aes256>::new_from_slices(enc_key, iv).map_err(|_| "invalid key")?;
    let mut buffer = ciphertext.to_vec();
    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| "decryption failed".to_string())?
        .to_vec();
    String::from_utf8(plaintext).map_err(|_| "invalid plaintext".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let password = "admin";
        let text = "secret-api-key";
        let encrypted = encrypt_with_password(text, password).expect("encrypt");
        assert!(encrypted.starts_with(ENCRYPTION_PREFIX));
        let decrypted = decrypt_with_password(&encrypted, password).expect("decrypt");
        assert_eq!(decrypted, text);
    }

    #[test]
    fn decrypt_rejects_tampered_payload() {
        let password = "admin";
        let text = "secret-api-key";
        let encrypted = encrypt_with_password(text, password).expect("encrypt");
        let mut bytes = STANDARD
            .decode(encrypted.strip_prefix(ENCRYPTION_PREFIX).unwrap())
            .unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x1;
        let tampered = format!("{}{}", ENCRYPTION_PREFIX, STANDARD.encode(bytes));
        assert!(decrypt_with_password(&tampered, password).is_err());
    }
}
