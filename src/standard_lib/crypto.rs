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
        "blake2b" => {
            use blake2::{Blake2b512, Digest};
            let mut hasher = Blake2b512::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        _ => {
            return Err(CorvoError::invalid_argument(
                "Unsupported hash algorithm. Use md5, sha256, sha512, or blake2b",
            ))
        }
    };

    Ok(Value::String(result))
}

pub fn encrypt(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let secret = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.encrypt requires a secret"))?;

    let value = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.encrypt requires a value"))?;

    let key_bytes: Vec<u8> = secret
        .bytes()
        .take(32)
        .chain(std::iter::repeat(0u8))
        .take(32)
        .collect();
    let data_bytes = value.as_bytes();

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
    let secret = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.decrypt requires a secret"))?;

    let value = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.decrypt requires a value"))?;

    let encrypted = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, value)
        .map_err(|_| CorvoError::invalid_argument("Invalid base64 data"))?;

    let key_bytes: Vec<u8> = secret
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

pub fn hash_file(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let algorithm = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.hash_file requires an algorithm"))?;

    let path = args
        .get(1)
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.hash_file requires a file path"))?;

    let data = std::fs::read(path).map_err(|e| CorvoError::file_system(e.to_string()))?;

    let result = match algorithm.as_str() {
        "md5" => {
            use md5::{Digest, Md5};
            let mut hasher = Md5::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        }
        "sha256" => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        }
        "sha512" => {
            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        }
        "blake2b" => {
            use blake2::{Blake2b512, Digest};
            let mut hasher = Blake2b512::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        }
        _ => {
            return Err(CorvoError::invalid_argument(
                "Unsupported hash algorithm. Use md5, sha256, sha512, or blake2b",
            ))
        }
    };

    Ok(Value::String(result))
}

pub fn checksum(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.checksum requires a file path"))?;

    let data = std::fs::read(path).map_err(|e| CorvoError::file_system(e.to_string()))?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(Value::String(format!("{:x}", hasher.finalize())))
}

fn make_cksum_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256u32 {
        let mut crc = i << 24;
        for _ in 0..8 {
            if crc & 0x8000_0000 != 0 {
                crc = (crc << 1) ^ 0x04C1_1DB7;
            } else {
                crc <<= 1;
            }
        }
        table[i as usize] = crc;
    }
    table
}

/// Compute the GNU cksum CRC-32 for a file.
/// Returns a map with keys "crc" (u32 as f64) and "size" (byte count as f64).
pub fn crc32_file(args: &[Value], _named_args: &HashMap<String, Value>) -> CorvoResult<Value> {
    let path = args
        .first()
        .and_then(|v| v.as_string())
        .ok_or_else(|| CorvoError::invalid_argument("crypto.crc32_file requires a file path"))?;

    let data = std::fs::read(path).map_err(|e| CorvoError::file_system(e.to_string()))?;
    let size = data.len() as u64;
    let table = make_cksum_crc_table();

    let mut crc: u32 = 0;
    for byte in &data {
        crc = table[((crc >> 24) ^ *byte as u32) as usize] ^ (crc << 8);
    }
    // Append length bytes (LSB first, until zero)
    let mut len = size;
    loop {
        crc = table[((crc >> 24) ^ (len & 0xff) as u32) as usize] ^ (crc << 8);
        len >>= 8;
        if len == 0 {
            break;
        }
    }
    crc ^= 0xffff_ffff;

    let mut result = std::collections::HashMap::new();
    result.insert("crc".to_string(), Value::Number(crc as f64));
    result.insert("size".to_string(), Value::Number(size as f64));
    Ok(Value::Map(result))
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
            Value::String(key.to_string()),
            Value::String(data.to_string()),
        ];
        let encrypted = encrypt(&enc_args, &empty_args()).unwrap();

        let dec_args = vec![Value::String(key.to_string()), encrypted];
        let decrypted = decrypt(&dec_args, &empty_args()).unwrap();
        assert_eq!(decrypted, Value::String(data.to_string()));
    }

    #[test]
    fn test_encrypt_wrong_key() {
        let enc_args = vec![
            Value::String("correct_key".to_string()),
            Value::String("data".to_string()),
        ];
        let encrypted = encrypt(&enc_args, &empty_args()).unwrap();

        let dec_args = vec![Value::String("wrong_key".to_string()), encrypted];
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

    #[test]
    fn test_hash_file_sha256() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"hello").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let args = vec![Value::String("sha256".to_string()), Value::String(path)];
        let result = hash_file(&args, &empty_args()).unwrap();
        match result {
            Value::String(h) => assert_eq!(h.len(), 64),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_hash_file_md5() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"hello").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let args = vec![Value::String("md5".to_string()), Value::String(path)];
        let result = hash_file(&args, &empty_args()).unwrap();
        match result {
            Value::String(h) => assert_eq!(h, "5d41402abc4b2a76b9719d911017c592"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_hash_file_matches_hash() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"corvo test content").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let file_args = vec![Value::String("sha256".to_string()), Value::String(path)];
        let file_result = hash_file(&file_args, &empty_args()).unwrap();

        let data_args = vec![
            Value::String("sha256".to_string()),
            Value::String("corvo test content".to_string()),
        ];
        let data_result = hash(&data_args, &empty_args()).unwrap();

        assert_eq!(file_result, data_result);
    }

    #[test]
    fn test_hash_file_invalid_algorithm() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"data").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let args = vec![Value::String("sha1".to_string()), Value::String(path)];
        assert!(hash_file(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_hash_file_missing_file() {
        let args = vec![
            Value::String("sha256".to_string()),
            Value::String("/nonexistent/path/file.txt".to_string()),
        ];
        assert!(hash_file(&args, &empty_args()).is_err());
    }

    #[test]
    fn test_checksum_sha256() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"hello").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let args = vec![Value::String(path)];
        let result = checksum(&args, &empty_args()).unwrap();
        match result {
            Value::String(h) => assert_eq!(h.len(), 64), // SHA-256 produces 64 hex chars
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_checksum_matches_hash_file_sha256() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"checksum test").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let checksum_args = vec![Value::String(path.clone())];
        let checksum_result = checksum(&checksum_args, &empty_args()).unwrap();

        let hash_file_args = vec![Value::String("sha256".to_string()), Value::String(path)];
        let hash_file_result = hash_file(&hash_file_args, &empty_args()).unwrap();

        assert_eq!(checksum_result, hash_file_result);
    }

    #[test]
    fn test_checksum_missing_file() {
        let args = vec![Value::String("/nonexistent/path/file.txt".to_string())];
        assert!(checksum(&args, &empty_args()).is_err());
    }
}
