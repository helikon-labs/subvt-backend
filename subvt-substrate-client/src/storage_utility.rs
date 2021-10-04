//! Substrate storage RPC access helper functions.

use subvt_types::substrate::metadata::{StorageEntryType, StorageMetadata, Metadata, StorageHasher};
use jsonrpsee_types::v2::params::JsonRpcParams;
use parity_scale_codec::{Encode};
use jsonrpsee_types::JsonValue;

/// Get storage key in hex string format for a plain storage type.
pub fn get_storage_plain_key(
    module_name: &str,
    storage_name: &str,
) -> String {
    let hasher = &StorageHasher::V13(frame_metadata::v13::StorageHasher::Twox128);
    let mut hash: Vec<u8> = Vec::new();
    let mut module_name_hash = StorageMetadata::hash(
        hasher,
        module_name.as_bytes(),
    );
    hash.append(&mut module_name_hash);
    let mut storage_name_hash = StorageMetadata::hash(
        hasher,
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

fn get_map_key_hash<T>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key: &T,
) -> Vec<u8> where T: Encode {
    let storage_metadata = metadata.module(module_name).unwrap()
        .storage.get(storage_name).unwrap();
    let key_hash = match &storage_metadata.ty {
        StorageEntryType::V12(storage_entry_type) => {
            let hasher = match storage_entry_type {
                frame_metadata::v12::StorageEntryType::Map { hasher, .. } => {
                    hasher
                }
                frame_metadata::v12::StorageEntryType::DoubleMap { hasher, .. } => {
                    hasher
                }
                _ => panic!(
                    "Unexpected storage entry type. Expected map, got: {:?}",
                    storage_entry_type
                )
            };
            StorageMetadata::hash_key(
                &StorageHasher::V12(hasher.clone()),
                key,
            )
        }
        StorageEntryType::V13(storage_entry_type) => {
            let hasher = match storage_entry_type {
                frame_metadata::v13::StorageEntryType::Map { hasher, .. } => {
                    hasher
                }
                frame_metadata::v13::StorageEntryType::DoubleMap { hasher, .. } => {
                    hasher
                }
                _ => panic!(
                    "Unexpected storage entry type. Expected map, got: {:?}",
                    storage_entry_type
                )
            };
            StorageMetadata::hash_key(
                &StorageHasher::V13(hasher.clone()),
                key,
            )
        }
    };
    key_hash
}

/// Get storage key in hex string format for a map storage type.
pub fn get_storage_map_key<T>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key: &T,
) -> String where T: Encode {
    let storage_key_hex = get_storage_plain_key(module_name, storage_name);
    let map_key_hash = get_map_key_hash(metadata, module_name, storage_name, key);
    let map_key_hex: String = map_key_hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("{}{}", storage_key_hex, map_key_hex)
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

fn _get_double_map_key_hash<T, U>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
) -> (Vec<u8>, Vec<u8>) where T: Encode, U: Encode {
    let storage_metadata = metadata.module(module_name).unwrap()
        .storage.get(storage_name).unwrap();
    let key_hash_pair = match &storage_metadata.ty {
        StorageEntryType::V12(storage_entry_type) => {
            let (hasher_1, hasher_2) = match storage_entry_type {
                frame_metadata::v12::StorageEntryType::DoubleMap { hasher, key2_hasher, .. } => {
                    (hasher, key2_hasher)
                }
                _ => panic!(
                    "Unexpected storage entry type. Expected double map, got: {:?}",
                    storage_entry_type
                )
            };
            let key_1_hash = StorageMetadata::hash_key(
                &StorageHasher::V12(hasher_1.clone()),
                key_1,
            );
            let key_2_hash = StorageMetadata::hash_key(
                &StorageHasher::V12(hasher_2.clone()),
                key_2,
            );
            (key_1_hash, key_2_hash)
        }
        StorageEntryType::V13(storage_entry_type) => {
            let (hasher_1, hasher_2) = match storage_entry_type {
                frame_metadata::v13::StorageEntryType::DoubleMap { hasher, key2_hasher, .. } => {
                    (hasher, key2_hasher)
                }
                _ => panic!(
                    "Unexpected storage entry type. Expected double map, got: {:?}",
                    storage_entry_type
                )
            };
            let key_1_hash = StorageMetadata::hash_key(
                &StorageHasher::V13(hasher_1.clone()),
                key_1,
            );
            let key_2_hash = StorageMetadata::hash_key(
                &StorageHasher::V13(hasher_2.clone()),
                key_2,
            );
            (key_1_hash, key_2_hash)
        }
    };
    key_hash_pair
}

/// Get storage key in hex string format for a double-map storage type.
fn _get_storage_double_map_key<T, U>(
    metadata: &Metadata,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
) -> String where T: Encode, U: Encode {
    let storage_key_hex = get_storage_plain_key(module_name, storage_name);
    let mut map_keys_hash: Vec<u8> = Vec::new();
    let mut key_hash_pair = _get_double_map_key_hash(
        metadata,
        module_name,
        storage_name,
        key_1,
        key_2,
    );
    map_keys_hash.append(&mut key_hash_pair.0);
    map_keys_hash.append(&mut key_hash_pair.1);
    let map_keys_hash_hex: String = map_keys_hash.iter().map(|b| format!("{:02x}", b)).collect();
    format!("{}{}", storage_key_hex, map_keys_hash_hex)
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
    let mut params: Vec<JsonValue> = vec![
        _get_storage_double_map_key(
            metadata,
            module_name,
            storage_name,
            key_1,
            key_2,
        ).into()
    ];
    if let Some(block_hash) = block_hash { params.push(block_hash.into()); }
    JsonRpcParams::Array(params)
}
