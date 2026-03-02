use crate::{KrillError, KrillResult};

pub struct StorageKeys;

impl StorageKeys {
    pub fn gen_store_key(identifier: &[u8], op_identifier: &[u8]) -> Vec<u8> {
        let prefix = blake3::hash(identifier).as_bytes().to_vec();

        let mut store_key = Vec::<u8>::new();

        store_key.extend_from_slice(&prefix);
        store_key.extend_from_slice(op_identifier);

        store_key
    }

    pub fn gen_store_key_prefix(identifier: &[u8]) -> blake3::Hash {
        blake3::hash(identifier)
    }

    pub fn get_store_key_prefix(value: &[u8]) -> KrillResult<blake3::Hash> {
        if value.len() < 33usize {
            Err(KrillError::InvalidStoreKeyLength)
        } else {
            let bytes: [u8; 32] = value[..32].try_into().unwrap();

            Ok(bytes.into())
        }
    }

    pub fn get_store_key_suffix(value: &[u8]) -> KrillResult<Vec<u8>> {
        if value.len() < 33usize {
            Err(KrillError::InvalidStoreKeyLength)
        } else {
            Ok(value[32..].to_vec())
        }
    }
}
