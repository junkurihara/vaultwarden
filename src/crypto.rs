//
// PBKDF2 derivation
//
use std::num::NonZeroU32;

use data_encoding::{Encoding, HEXLOWER};
use ring::{digest, hmac, pbkdf2};
use rust_argon2::{hash_raw, verify_raw, Config, Variant, Version};

static LEGACY_PBKDF2_DIGEST_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const OUTPUT_LEN: usize = digest::SHA256_OUTPUT_LEN;

fn get_argon2_config(iterations: u32, memory: u32, parallelism: u32) -> Config<'static> {
    Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: memory,
        time_cost: iterations,
        lanes: parallelism,
        secret: &[],
        ad: &[],
        hash_length: OUTPUT_LEN as u32,
    }
}

pub fn hash_password(secret: &[u8], salt: &[u8], iterations: u32, memory: u32, parallelism: u32) -> Vec<u8> {
    let iterations = NonZeroU32::new(iterations).expect("iterations must be non-zero");
    // workaround for migration from pbkdf2 to argon2
    if (memory, parallelism) == (0, 0) {
        info!("set_password: using legacy PBKDF2");
        let mut out = vec![0; OUTPUT_LEN];
        pbkdf2::derive(LEGACY_PBKDF2_DIGEST_ALG, iterations, salt, secret, &mut out);
        return out;
    }
    info!("set_password: using argon2");
    let memory = NonZeroU32::new(memory).expect("memory must be non-zero");
    let parallelism = NonZeroU32::new(parallelism).expect("parallelism must be non-zero");
    let config = get_argon2_config(iterations.get(), memory.get(), parallelism.get());
    hash_raw(secret, salt, &config).unwrap()
}

pub fn verify_password_hash(
    secret: &[u8],
    salt: &[u8],
    previous: &[u8],
    iterations: u32,
    memory: u32,
    parallelism: u32,
) -> bool {
    let iterations = NonZeroU32::new(iterations).expect("iterations must be non-zero");
    // workaround for migration from pbkdf2 to argon2
    if (memory, parallelism) == (0, 0) {
        info!("verify_password_hash: using legacy pbkdf2");
        return pbkdf2::verify(LEGACY_PBKDF2_DIGEST_ALG, iterations, salt, secret, previous).is_ok();
    }
    info!("verify_password_hash: using argon2");
    let memory = NonZeroU32::new(memory).expect("memory must be non-zero");
    let parallelism = NonZeroU32::new(parallelism).expect("parallelism must be non-zero");
    let config = get_argon2_config(iterations.get(), memory.get(), parallelism.get());
    verify_raw(secret, salt, previous, &config).unwrap()
}

//
// HMAC
//
pub fn hmac_sign(key: &str, data: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, key.as_bytes());
    let signature = hmac::sign(&key, data.as_bytes());

    HEXLOWER.encode(signature.as_ref())
}

//
// Random values
//

/// Return an array holding `N` random bytes.
pub fn get_random_bytes<const N: usize>() -> [u8; N] {
    use ring::rand::{SecureRandom, SystemRandom};

    let mut array = [0; N];
    SystemRandom::new().fill(&mut array).expect("Error generating random values");

    array
}

/// Encode random bytes using the provided function.
pub fn encode_random_bytes<const N: usize>(e: Encoding) -> String {
    e.encode(&get_random_bytes::<N>())
}

/// Generates a random string over a specified alphabet.
pub fn get_random_string(alphabet: &[u8], num_chars: usize) -> String {
    // Ref: https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
    use rand::Rng;
    let mut rng = rand::rng();

    (0..num_chars)
        .map(|_| {
            let i = rng.random_range(0..alphabet.len());
            alphabet[i] as char
        })
        .collect()
}

/// Generates a random numeric string.
pub fn get_random_string_numeric(num_chars: usize) -> String {
    const ALPHABET: &[u8] = b"0123456789";
    get_random_string(ALPHABET, num_chars)
}

/// Generates a random alphanumeric string.
pub fn get_random_string_alphanum(num_chars: usize) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                              abcdefghijklmnopqrstuvwxyz\
                              0123456789";
    get_random_string(ALPHABET, num_chars)
}

pub fn generate_id<const N: usize>() -> String {
    encode_random_bytes::<N>(HEXLOWER)
}

pub fn generate_send_file_id() -> String {
    // Send File IDs are globally scoped, so make them longer to avoid collisions.
    generate_id::<32>() // 256 bits
}

use crate::db::models::AttachmentId;
pub fn generate_attachment_id() -> AttachmentId {
    // Attachment IDs are scoped to a cipher, so they can be smaller.
    AttachmentId(generate_id::<10>()) // 80 bits
}

/// Generates a numeric token for email-based verifications.
pub fn generate_email_token(token_size: u8) -> String {
    get_random_string_numeric(token_size as usize)
}

/// Generates a personal API key.
/// Upstream uses 30 chars, which is ~178 bits of entropy.
pub fn generate_api_key() -> String {
    get_random_string_alphanum(30)
}

//
// Constant time compare
//
pub fn ct_eq<T: AsRef<[u8]>, U: AsRef<[u8]>>(a: T, b: U) -> bool {
    use subtle::ConstantTimeEq;
    a.as_ref().ct_eq(b.as_ref()).into()
}
