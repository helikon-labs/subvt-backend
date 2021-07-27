//! Substrate storage RPC access helper functions.

use crate::metadata::{StorageMetadata, Metadata};
use frame_metadata::v13::{StorageHasher, StorageEntryType};
use jsonrpsee_types::v2::params::JsonRpcParams;
use parity_scale_codec::{Encode};
use jsonrpsee_types::JsonValue;

/// Get storage key in hex string format for a plain storage type.
pub fn get_storage_plain_key(
    module_name: &str,
    storage_name: &str,
) -> String {
    let mut hash = StorageMetadata::hash(
        &StorageHasher::Twox128,
        module_name.as_bytes(),
    );
    let mut storage_name_hash = StorageMetadata::hash(
        &StorageHasher::Twox128,
        storage_name.as_bytes(),
    );
    hash.append(&mut storage_name_hash);
    let storage_key_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}", storage_key_hex)
}

/// Get JSONRPSee parameters for a plain storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn get_rpc_storage_plain_params<'a>(
    module: &'a str,
    name: &'a str,
    block_hash: Option<&'a str>,
) -> JsonRpcParams<'a> {
    let mut params: Vec<JsonValue> = vec![
        get_storage_plain_key(module, name).into()
    ];
    if let Some(block_hash) = block_hash { params.push(block_hash.into()); }
    JsonRpcParams::Array(params)
}

/// Get JSONRPSee parameters for a plain storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn get_rpc_paged_keys_params<'a>(
    module: &'a str,
    name: &'a str,
    count: usize,
    start_key: Option<&'a str>,
    block_hash: Option<&'a str>,
) -> JsonRpcParams<'a> {
    let params: Vec<JsonValue> = vec![
        get_storage_plain_key(module, name).into(),
        count.into(),
        if let Some(start_key) = start_key {
            start_key.into()
        } else {
            JsonValue::Null
        },
        if let Some(block_hash) = block_hash {
            block_hash.into()
        } else {
            JsonValue::Null
        },
    ];
    JsonRpcParams::Array(params)
}

/// Get storage key in hex string format for a map storage type.
pub fn get_storage_map_key<T>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key: &T,
) -> String where T: Encode {
    let storage_metadata = metadata.module(module_name).unwrap()
        .storage.get(storage_name).unwrap();
    let hasher = match &storage_metadata.ty {
        StorageEntryType::Map { hasher, .. } => {
            hasher
        }
        StorageEntryType::DoubleMap { hasher, .. } => {
            hasher
        }
        _ => panic!(
            "Unexpected storage entry type. Expected map, got: {:?}",
            storage_metadata.ty
        )
    };
    let mut hash = StorageMetadata::hash(
        &StorageHasher::Twox128,
        module_name.as_bytes(),
    );
    let mut storage_name_hash = StorageMetadata::hash(
        &StorageHasher::Twox128,
        storage_name.as_bytes(),
    );
    hash.append(&mut storage_name_hash);
    let mut key_hash = StorageMetadata::hash_key(hasher, key);
    hash.append(&mut key_hash);
    let storage_key_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}", storage_key_hex)
}

pub fn get_rpc_paged_map_keys_params<'a, T>(
    metadata: &Metadata,
    module_name: &'a str,
    storage_name: &'a str,
    key: &T,
    count: usize,
    start_key: Option<&'a str>,
    block_hash: Option<&'a str>,
) -> JsonRpcParams<'a> where T: Encode {
    let params: Vec<JsonValue> = vec![
        get_storage_map_key(metadata, module_name, storage_name, key).into(),
        count.into(),
        if let Some(start_key) = start_key {
            start_key.into()
        } else {
            JsonValue::Null
        },
        if let Some(block_hash) = block_hash {
            block_hash.into()
        } else {
            JsonValue::Null
        },
    ];
    JsonRpcParams::Array(params)
}


/// Get JSONRPSee parameters for a map storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn get_rpc_storage_map_params<'a, T>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key: &T,
    block_hash: Option<&'a str>,
) -> JsonRpcParams<'a> where T: Encode {
    let mut params: Vec<JsonValue> = vec![
        get_storage_map_key(metadata, module_name, storage_name, key).into()
    ];
    if let Some(block_hash) = block_hash { params.push(block_hash.into()); }
    JsonRpcParams::Array(params)
}

/// Get storage key in hex string format for a double-map storage type.
fn _get_storage_double_map_key<T, U>(
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    hasher_1: &StorageHasher,
    key_2: &U,
    hasher_2: &StorageHasher,
) -> String where T: Encode, U: Encode {
    let mut hash = StorageMetadata::hash(
        &StorageHasher::Twox128,
        module_name.as_bytes(),
    );
    let mut storage_name_hash = StorageMetadata::hash(
        &StorageHasher::Twox128,
        storage_name.as_bytes(),
    );
    hash.append(&mut storage_name_hash);
    let mut key_1_hash = StorageMetadata::hash_key(hasher_1, key_1);
    hash.append(&mut key_1_hash);
    let mut key_2_hash = StorageMetadata::hash_key(hasher_2, key_2);
    hash.append(&mut key_2_hash);
    let storage_key_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("0x{}", storage_key_hex)
}

/// Get JSONRPSee parameters for a double-map storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn _get_rpc_storage_double_map_params<'a, T, U>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
    block_hash: Option<&'a str>,
) -> JsonRpcParams<'a> where T: Encode, U: Encode {
    let storage_metadata = metadata.module(module_name).unwrap()
        .storage.get(storage_name).unwrap();
    let (hasher_1, hasher_2) = match &storage_metadata.ty {
        StorageEntryType::DoubleMap { hasher, key2_hasher, .. } => {
            (hasher, key2_hasher)
        }
        _ => panic!(
            "Unexpected storage entry type. Expected double map, got: {:?}",
            storage_metadata.ty
        )
    };
    let mut params: Vec<JsonValue> = vec![
        _get_storage_double_map_key(
            module_name,
            storage_name,
            key_1,
            hasher_1,
            key_2,
            hasher_2,
        ).into()
    ];
    if let Some(block_hash) = block_hash { params.push(block_hash.into()); }
    JsonRpcParams::Array(params)
}