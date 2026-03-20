use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

pub fn hash(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let algorithm = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.hash requires an algorithm"))?;

    let data = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.hash requires data"))?;

    let result = match algorithm.as_str() {
        "md5" => {
            use md5::{Digest, Md5};
            let mut hasher = Md5::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha256" => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha512" => {
            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        _ => {
            return Err(CorvoError::invalid_argument(
                "Unsupported hash algorithm. Use md5, sha256, or sha512",
            ))
        }
    };

    Ok(Value::String(result))
}

pub fn encrypt(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let data = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.encrypt requires data"))?;

    let key = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.encrypt requires a key"))?;

    let key_bytes: Vec<u8> = key
        .bytes()
        .take(32)
        .chain(std::iter::repeat(0u8))
        .take(32)
        .collect();
    let data_bytes = data.as_bytes();

    let encrypted: Vec<u8> = data_bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    Ok(Value::String(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &encrypted,
    )))
}

pub fn decrypt(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let data = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.decrypt requires data"))?;

    let key = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.decrypt requires a key"))?;

    let encrypted = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)
        .map_err(|_| CorvoError::invalid_argument("Invalid base64 data"))?;

    let key_bytes: Vec<u8> = key
        .bytes()
        .take(32)
        .chain(std::iter::repeat(0u8))
        .take(32)
        .collect();

    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_bytes.len()])
        .collect();

    String::from_utf8(decrypted)
        .map(Value::String)
        .map_err(|e| CorvoError::invalid_argument(e.to_string()))
}

pub fn uuid(_args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    Ok(Value::String(uuid::Uuid::new_v4().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_args() -> HashMap<String, Value> {
        HashMap::new()
    }

    #[test]
    fn test_hash_md5() {
        let args = vec![
            Value::String("md5".to_string()),
            Value::String("hello".to_string()),
        ];
        let result = hash(&args, &empty_args()).unwrap();
        match result {
            Value::String(h) => assert_eq!(h, "5d41402abc4b2a76b9719d911017c592"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_hash_sha256() {
        let args = vec![
            Value::String("sha256".to_string()),
            Value::String("hello".to_string()),
        ];
        let result = hash(&args, &empty_args()).unwrap();
        match result {
            Value::String(h) => assert_eq!(h.len(), 64), // SHA-256 produces 64 hex chars
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_hash_sha512() {
        let args = vec![
            Value::String("sha512".to_string()),
            Value::String("hello".to_string()),
        ];
        let result = hash(&args, &empty_args()).unwrap();
        match result {
            Value::String(h) => assert_eq!(h.len(), 128), // SHA-512 produces 128 hex chars
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_hash_invalid_algorithm() {
        let args = vec![
            Value::String("sha1".to_string()),
            Value::String("hello".to_string()),
        ];
        assert!(hash(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = "secret message";
        let key = "mykey";

        let enc_args = vec![
            Value::String(data.to_string()),
            Value::String(key.to_string()),
        ];
        let encrypted = encrypt(&enc_args, &empty_args()).unwrap();

        let dec_args = vec![encrypted, Value::String(key.to_string())];
        let decrypted = decrypt(&dec_args, &empty_args()).unwrap();
        assert_eq!(decrypted, Value::String(data.to_string()));
    }

    #[test]
    fn test_encrypt_wrong_key() {
        let enc_args = vec![
            Value::String("data".to_string()),
            Value::String("correct_key".to_string()),
        ];
        let encrypted = encrypt(&enc_args, &empty_args()).unwrap();

        let dec_args = vec![encrypted, Value::String("wrong_key".to_string())];
        let result = decrypt(&dec_args, &empty_args());
        // May succeed but produce garbage, or fail on UTF-8
        // Either way it should NOT equal the original
        if let Ok(val) = result {
            assert_ne!(val, Value::String("data".to_string()));
        }
    }

    #[test]
    fn test_uuid_format() {
        let result = uuid(&[], &empty_args()).unwrap();
        match result {
            Value::String(u) => {
                assert_eq!(u.len(), 36); // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
                assert_eq!(u.chars().nth(8).unwrap(), '-');
            }
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_uuid_unique() {
        let u1 = uuid(&[], &empty_args()).unwrap();
        let u2 = uuid(&[], &empty_args()).unwrap();
        assert_ne!(u1, u2);
    }
}
