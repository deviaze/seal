use std::num::NonZeroU32;

use crate::{table_helpers::TableBuilder, LuaValueResult, colors};
use base64::Engine;
use mluau::prelude::*;

use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use ring::digest::{Context, SHA256, SHA256_OUTPUT_LEN};
use pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePublicKey};

use rsa::{pkcs8::{self, DecodePrivateKey}, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rand::rngs::OsRng;

use base64::engine::general_purpose::STANDARD as base64_standard;

fn generate_aes_key(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    // 32 bytes = 256 bits len for AES-256 key
    const KEY_LEN: usize = 32;
    let mut key_buff = [0u8; KEY_LEN];
    let rng = SystemRandom::new();
    match rng.fill(&mut key_buff) {
        Ok(_) => {},
        Err(_err) => {
            return wrap_err!("crypt: error creating aes key (filling the buffer)");
        }
    };
    // let key_encoded64 = base64::engine::GeneralPurpose::encode(key_buff);
    let key_encoded64 = base64_standard.encode(key_buff);
    Ok(LuaValue::String(luau.create_string(key_encoded64)?))
}

fn aes_encrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let plaintext = match multivalue.pop_front() {
        Some(LuaValue::String(plaintext)) => plaintext.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.encrypt: expected plaintext to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.encrypt: expected plaintext, got nothing");
        }
    };
    let aes_key = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.encrypt: expected second argument (AES key) to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.encrypt: expected second argument (AES key), got nothing.");
        }
    };

    let aes_key_bytes = match base64_standard.decode(aes_key) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.aes.encode: unable to decode AES key from base64");
        }
    };

    if aes_key_bytes.len() != 32 {
        return wrap_err!("crypt.aes.encrypt: AES key must be 32 bytes to encrypt AES-256, got {}", aes_key_bytes.len());
    }

    let encrypted_text = match simple_crypt::encrypt(plaintext.as_bytes(), &aes_key_bytes) {
        Ok(encrypted_bytes) => base64_standard.encode(encrypted_bytes),
        Err(err) => {
            return wrap_err!("crypt.aes.encrypt: unable to encrypt: {}", err)
        } 
    };
    Ok(LuaValue::String(
        luau.create_string(encrypted_text)?
    ))
}

fn aes_decrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let encrypted_text = match multivalue.pop_front() {
        Some(LuaValue::String(text)) => text.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.decrypt: expected encrypted text to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.decrypt: expected encrypted text, got nothing");
        }
    };

    let encrypted_bytes = match base64_standard.decode(&encrypted_text) {
        Ok(bytes) => bytes,
        Err(_err) => {
            return wrap_err!("crypt.aes.decrypt: unable to decode ciphertext from base64");
        }
    };

    let aes_key = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.decrypt: expected second argument (AES key) to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.decrypt: expected second argument (AES key), got nothing.");
        }
    };

    let aes_key_bytes = match base64_standard.decode(aes_key) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.aes.decrypt: cannot decode AES key from base64");
        }
    };

    if aes_key_bytes.len() != 32 {
        return wrap_err!("crypt.aes.decrypt: AES key must be 32 bytes to decrypt AES-256, got {}", aes_key_bytes.len());
    }

    let plainbytes = match simple_crypt::decrypt(&encrypted_bytes, &aes_key_bytes) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_err!("crypt.aes.decrypt: unable to decrypt ciphertext: {}", err);
        }
    };

    Ok(LuaValue::String(
        luau.create_string(plainbytes)?
    ))
}

pub fn create_aes(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("generatekey", generate_aes_key)?
        .with_function("encrypt", aes_encrypt)?
        .with_function("decrypt", aes_decrypt)?
        .build_readonly()
}

fn generate_rsa_keys(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    let mut rng = OsRng;
    let key_length = 2048;

    let private_key = match RsaPrivateKey::new(&mut rng, key_length) {
        Ok(key) => key,
        Err(err) => {
            return wrap_err!("crypt.rsa: error generating private key: {}", err);
        }
    };
    let public_key = RsaPublicKey::from(&private_key);

    let private_key_encoded = match private_key.to_pkcs8_pem(pkcs8::LineEnding::LF) {
        Ok(key) => key.to_string(),
        Err(err) => {
            return wrap_err!("crypt.rsa: error encoding private key to pkcs1_pem: {}", err);
        }
    };
    let public_key_encoded = match public_key.to_public_key_pem(pkcs8::LineEnding::LF) {
        Ok(key) => key,
        Err(err) => {
            return wrap_err!("crypt.rsa: error encoding public key to pkcs1_pem: {}", err);
        }
    };

    Ok(LuaValue::Table(
        TableBuilder::create(luau)?
            .with_value("private", private_key_encoded.into_lua(luau)?)?
            .with_value("public", public_key_encoded.into_lua(luau)?)?
            .build_readonly()?
    ))
}

fn rsa_encrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let plaintext = match multivalue.pop_front() {
        Some(LuaValue::String(plaintext)) => plaintext.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.encrypt: expected plaintext to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.encrypt: expected plaintext, got nothing");
        }
    };

    let public_key_pem = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.encrypt: expected second argument (public key) to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.encrypt: expected second argument (public key), got nothing.");
        }
    };

    let public_key = match RsaPublicKey::from_public_key_pem(&public_key_pem) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.rsa.encrypt: unable to decode public key from PEM");
        }
    };

    let mut rng = OsRng;
    let encrypted_data = match public_key.encrypt(&mut rng, Pkcs1v15Encrypt, plaintext.as_bytes()) {
        Ok(data) => data,
        Err(_err) => {
            return wrap_err!("crypt.rsa.encrypt: encryption failed");
        }
    };

    let encoded64 = base64_standard.encode(&encrypted_data);
    Ok(LuaValue::String(
        luau.create_string(&encoded64)?
    ))
}

fn rsa_decrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let encrypted_text = match multivalue.pop_front() {
        Some(LuaValue::String(text)) => text.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.decrypt: expected encrypted text to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.decrypt: expected encrypted text, got nothing");
        }
    };

    let private_key_pem = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.decrypt: expected RSA key to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.decrypt: expected RSA key, got nothing");
        }
    };

    let private_key = match RsaPrivateKey::from_pkcs8_pem(&private_key_pem) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.rsa.decrypt: unable to decode private key from PEM (pkcs8_pem)");
        }
    };

    let encrypted_bytes = match base64_standard.decode(encrypted_text) {
        Ok(bytes) => bytes,
        Err(_err) => {
            return wrap_err!("crypt.rsa.decrypt: unable to decode encrypted text from base64");
        }
    };

    let plainbytes = match private_key.decrypt(Pkcs1v15Encrypt, &encrypted_bytes) {
        Ok(bytes) => bytes,
        Err(_err) => {
            return wrap_err!("crypt.rsa.decrypt: decryption failed");
        }
    };

    Ok(LuaValue::String(
        luau.create_string(plainbytes)?
    ))
}
pub fn create_rsa(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("generatekeys", generate_rsa_keys)?
        .with_function("encrypt", rsa_encrypt)?
        .with_function("decrypt", rsa_decrypt)?
        .build_readonly()
}

fn hash_sha2_256(luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::String(plaintext) => {
            let mut context = Context::new(&SHA256);
            let plaintext = plaintext.to_string_lossy();
            let plaintext_bytes = plaintext.as_ref();
            context.update(plaintext_bytes);
            let rust_buffy: [u8; 32] = {
                let hash_result = context.finish();
                let hash_result = hash_result.as_ref();
                let mut rust_buffy = [0u8; 32];
                rust_buffy.copy_from_slice(hash_result);
                rust_buffy
            };
            let result_buffy = luau.create_buffer(rust_buffy)?;
            Ok(LuaValue::Buffer(result_buffy))
        },
        other => {
            wrap_err!("crypt.hash.sha2: expected plaintext to be a string, got: {:?}", other)
        }
    }
}

pub fn create_hash(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("sha2", hash_sha2_256)?
        .build_readonly()
}

const PBKDF2_ITERATIONS: NonZeroU32 = NonZeroU32::new(100_000).unwrap();

fn password_hash(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let password_raw: String = match value {
        LuaValue::String(password_raw) => {
            password_raw.to_string_lossy().to_string()
        },
        other => {
            return wrap_err!("Expected password to hash to be a string, got: {:?}", other);
        }
    };
    let rng = ring::rand::SystemRandom::new();
    let mut salt = [0u8; 16];
    match rng.fill(&mut salt) {
        Ok(_salt) => (),
        Err(_err) => {
            return wrap_err!("crypt.password.hash: unable to salt password");
        }
    };

    let mut hash = vec![0u8; SHA256_OUTPUT_LEN];
    pbkdf2::derive(
        PBKDF2_HMAC_SHA256,
        PBKDF2_ITERATIONS, 
        &salt, 
        password_raw.as_bytes(), 
        &mut hash,
    );

    let salt_buffy = luau.create_buffer(salt)?;
    let hash_buffy = luau.create_buffer(hash)?;
    Ok(LuaValue::Table(TableBuilder::create(luau)?
        .with_value("salt", salt_buffy)?
        .with_value("hash", hash_buffy)?
        .build_readonly()?
    ))
}

fn password_verify(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let verify_options = match value {
        LuaValue::Table(options) => {
            options
        },
        other => {
            return wrap_err!("crypt.password.verify: Expected VerifyPasswordOptions: {{ raw_password: string, hashed_password: HashedPassword }}, got: {:#?}", other);
        }
    };
    let raw_password = match verify_options.raw_get("raw_password")? {
        LuaValue::String(password) => {
            password.to_string_lossy()
        },
        LuaNil => {
            return wrap_err!("crypt.password.verify: expected VerifyPasswordOptions to contain field 'raw_password', got nil.");
        },
        other => {
            return wrap_err!("crypt.password.verify: expected VerifyPasswordOptions.raw_password to be a string, got: {:#?}", other);
        },
    };
    let (salt_buffer, hash_buffer) = match verify_options.raw_get("hashed_password")? {
        LuaValue::Table(hashed_password) => {
            let salt_buffer = match hashed_password.raw_get("salt")? {
                LuaValue::Buffer(salt) => salt,
                other => {
                    return wrap_err!("crypt.password.verify: expected VerifyPasswordOptions.hashed_password.salt to be a buffer, got: {:?}", other);
                }
            };
            let hash_buffer = match hashed_password.raw_get("hash")? {
                LuaValue::Buffer(hash_buffer) => hash_buffer,
                other => {
                    return wrap_err!("crypt.password.verify: expected VerifyPasswordOptions.hashed_password.hash to be a buffer, got: {:?}", other);
                }
            };
            (salt_buffer, hash_buffer)
        },
        other => {
            return wrap_err!("crypt.password.verify: expected VerifyPasswordOptions.hashed_password to be be a table with fields 'salt', and 'hash' (both buffers), got: {:?}", other);
        }
    };
    let salt = salt_buffer.to_vec();
    let hashed_password = hash_buffer.to_vec();

    Ok(LuaValue::Boolean(
        pbkdf2::verify(
            PBKDF2_HMAC_SHA256,
            PBKDF2_ITERATIONS,
            &salt,
            raw_password.as_bytes(),
            &hashed_password
        ).is_ok()
    ))
}

pub fn create_password(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("hash", password_hash)?
        .with_function("verify", password_verify)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("aes", create_aes(luau)?)?
        .with_value("rsa", create_rsa(luau)?)?
        .with_value("hash", create_hash(luau)?)?
        .with_value("password", create_password(luau)?)?
        .build_readonly()
}