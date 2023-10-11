// Portions of this code are derived from the original work by SonuBardai
// https://github.com/SonuBardai
// (Copyright (c) 2022 SonuBardai).

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, Aes256Gcm, KeyInit,
};
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use std::{num::NonZeroU32, path::Path};

use crate::store::Data;

pub fn load<P: AsRef<Path>>(fpath: P, master_pass: &str) -> anyhow::Result<Vec<Data>> {
    create_new_file_if_not_exists(&fpath, master_pass)?;
    let encrypted_file = std::fs::read(&fpath)?;
    let salt = &encrypted_file[..16];
    let cipher = get_cipher(master_pass, salt);
    let nonce = &encrypted_file[16..28];
    let encrypted_data = &encrypted_file[28..];
    let plain_text = cipher
        .decrypt(nonce.into(), encrypted_data.as_ref())
        .map_err(|_| anyhow::anyhow!("Master password incorrect."))?;
    let plain_text = String::from_utf8(plain_text)?;

    Ok(serde_json::from_str::<Vec<Data>>(&plain_text)?)
}

pub fn dump<P: AsRef<Path>>(fpath: P, master_pass: &str, data: Vec<Data>) -> anyhow::Result<()> {
    create_new_file_if_not_exists(&fpath, master_pass)?;
    let encrypted_file = std::fs::read(&fpath)?;
    let salt = &encrypted_file[..16];
    let cipher = get_cipher(master_pass, salt);
    let nonce = &encrypted_file[16..28];
    let plain_text = serde_json::to_string(&data)?;
    let encrypted_text = cipher
        .encrypt(nonce.into(), plain_text.as_ref())
        .map_err(|_| anyhow::anyhow!("Failed to encrypt passwords."))?;
    let mut content = salt.to_vec();
    content.extend(nonce);
    content.extend(encrypted_text);
    std::fs::write(&fpath, content)?;
    Ok(())
}

fn create_new_file_if_not_exists<P: AsRef<Path>>(
    fpath: P,
    master_pass: &str,
) -> anyhow::Result<()> {
    if !fpath.as_ref().exists() {
        let salt = get_random_salt()?;
        let (empty_json, nonce) = encrypt_contents("[]", master_pass, &salt)?;
        let mut content = salt.to_vec();
        content.extend(nonce);
        content.extend(empty_json);
        std::fs::write(&fpath, content)?;
    }
    Ok(())
}

fn get_random_salt() -> anyhow::Result<[u8; 16]> {
    let mut salt = [0u8; 16];
    let r = SystemRandom::new();
    r.fill(&mut salt)
        .map_err(|_| anyhow::anyhow!("Salt Error."))?;
    Ok(salt)
}

fn derive_encryption_key(master_password: &str, salt: &[u8]) -> [u8; 32] {
    let mut enc_key: [u8; 32] = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(100_000).unwrap(),
        salt,
        master_password.as_bytes(),
        &mut enc_key,
    );
    enc_key
}

fn get_cipher(master_password: &str, salt: &[u8]) -> Aes256Gcm {
    let enc_key = derive_encryption_key(master_password, salt);
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&enc_key));
    cipher
}

fn encrypt_contents(
    contents: &str,
    master_password: &str,
    salt: &[u8],
) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let cipher = get_cipher(master_password, salt);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_text = cipher
        .encrypt(&nonce, contents.as_ref())
        .map_err(|_| anyhow::anyhow!("Failed to encrypt passwords."))?;
    Ok((encrypted_text, nonce.to_vec()))
}
