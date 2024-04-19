//! Substrate storage RPC access helper functions.
use frame_metadata::{
    v14::{StorageEntryType, StorageHasher},
    RuntimeMetadataV14,
};
use jsonrpsee::core::params::ArrayParams;
use parity_scale_codec::Encode;
use serde_json::Value as JsonValue;
use subvt_types::substrate::metadata::hash;

/// Get storage key in hex string format for a plain storage type.
pub fn get_storage_plain_key(module_name: &str, storage_name: &str) -> String {
    let hasher = StorageHasher::Twox128;
    let mut storage_hash: Vec<u8> = Vec::new();
    let mut module_name_hash = hash(&hasher, module_name.as_bytes());
    storage_hash.append(&mut module_name_hash);
    let mut storage_name_hash = hash(&hasher, storage_name.as_bytes());
    storage_hash.append(&mut storage_name_hash);
    let storage_key_hex: String = hex::encode(storage_hash);
    format!("0x{storage_key_hex}")
}

/// Get JSONRPSee parameters for a plain storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn get_rpc_storage_plain_params<'a>(
    module: &'a str,
    name: &'a str,
    block_hash: Option<&'a str>,
) -> ArrayParams {
    //let mut params: Vec<JsonValue> = vec![.into()];
    let mut params = ArrayParams::new();
    params.insert(get_storage_plain_key(module, name)).unwrap();
    if let Some(block_hash) = block_hash {
        //params.push(block_hash.into());
        params.insert(block_hash).unwrap();
    }
    params
}

/// Get JSONRPSee parameters for a plain storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn get_rpc_paged_keys_params<'a>(
    module: &'a str,
    name: &'a str,
    count: usize,
    start_key: Option<&'a str>,
    block_hash: Option<&'a str>,
) -> ArrayParams {
    let mut params = ArrayParams::new();
    params.insert(get_storage_plain_key(module, name)).unwrap();
    params.insert(count).unwrap();
    if let Some(start_key) = start_key {
        params.insert(start_key).unwrap();
    } else {
        params.insert(JsonValue::Null).unwrap();
    }
    if let Some(block_hash) = block_hash {
        params.insert(block_hash).unwrap();
    } else {
        params.insert(JsonValue::Null).unwrap();
    }
    params
}

fn get_map_key_hash<T>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key: &T,
) -> Vec<u8>
where
    T: Encode,
{
    let storage_entry_type = &metadata
        .pallets
        .iter()
        .find(|p| p.name == module_name)
        .unwrap()
        .storage
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .find(|s| s.name == storage_name)
        .unwrap()
        .ty;
    let key_hash = {
        let maybe_hasher = match storage_entry_type {
            StorageEntryType::Map { hashers, .. } => hashers.first(),
            _ => {
                panic!("Unexpected storage entry type. Expected map, got: {storage_entry_type:?}",)
            }
        };
        if let Some(hasher) = maybe_hasher {
            hash(hasher, &key.encode())
        } else {
            panic!("Cannot get hasher for map storage {module_name}.{storage_name}.",);
        }
    };
    key_hash
}

/// Get storage key in hex string format for a map storage type.
pub fn get_storage_map_key<T>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key: &T,
) -> String
where
    T: Encode,
{
    let storage_key_hex = get_storage_plain_key(module_name, storage_name);
    let map_key_hash = get_map_key_hash(metadata, module_name, storage_name, key);
    let map_key_hex: String = hex::encode(map_key_hash);
    format!("{storage_key_hex}{map_key_hex}")
}

/// Get storage key in hex string format for a double map storage type.
pub fn get_storage_double_map_key<T, U>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
) -> String
where
    T: Encode,
    U: Encode,
{
    let storage_key_hex = get_storage_plain_key(module_name, storage_name);
    let (map_key_1_hash, map_key_2_hash) =
        get_double_map_key_hash(metadata, module_name, storage_name, key_1, key_2);
    let map_key_1_hex: String = hex::encode(map_key_1_hash);
    let map_key_2_hex: String = hex::encode(map_key_2_hash);
    format!("{storage_key_hex}{map_key_1_hex}{map_key_2_hex}")
}

pub fn get_rpc_paged_map_keys_params<'a, T>(
    metadata: &RuntimeMetadataV14,
    module_name: &'a str,
    storage_name: &'a str,
    key: &T,
    count: usize,
    start_key: Option<&'a str>,
    block_hash: Option<&'a str>,
) -> ArrayParams
where
    T: Encode,
{
    let mut params = ArrayParams::new();
    params
        .insert(get_storage_map_key(
            metadata,
            module_name,
            storage_name,
            key,
        ))
        .unwrap();
    params.insert(count).unwrap();
    if let Some(start_key) = start_key {
        params.insert(start_key).unwrap();
    } else {
        params.insert(JsonValue::Null).unwrap();
    }
    if let Some(block_hash) = block_hash {
        params.insert(block_hash).unwrap();
    } else {
        params.insert(JsonValue::Null).unwrap();
    }
    params
}

/// Get JSONRPSee parameters for a map storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn get_rpc_storage_map_params<T>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key: &T,
    block_hash: Option<&str>,
) -> ArrayParams
where
    T: Encode,
{
    let mut params = ArrayParams::new();
    params
        .insert(get_storage_map_key(
            metadata,
            module_name,
            storage_name,
            key,
        ))
        .unwrap();
    if let Some(block_hash) = block_hash {
        params.insert(block_hash).unwrap();
    }
    params
}

fn get_double_map_key_hash<T, U>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
) -> (Vec<u8>, Vec<u8>)
where
    T: Encode,
    U: Encode,
{
    let storage_entry_type = &metadata
        .pallets
        .iter()
        .find(|p| p.name == module_name)
        .unwrap()
        .storage
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .find(|s| s.name == storage_name)
        .unwrap()
        .ty;
    let key_hash_pair = {
        let (hasher_1, hasher_2) = match storage_entry_type {
            StorageEntryType::Map { hashers, .. } => {
                (hashers.first().unwrap(), hashers.get(1).unwrap())
            }
            _ => {
                panic!("Unexpected storage entry type. Expected map, got: {storage_entry_type:?}",)
            }
        };
        let key_1_hash = hash(hasher_1, &key_1.encode());
        let key_2_hash = hash(hasher_2, &key_2.encode());
        (key_1_hash, key_2_hash)
    };
    key_hash_pair
}

/// Get storage key in hex string format for a double-map storage type.
fn _get_storage_double_map_key<T, U>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
) -> String
where
    T: Encode,
    U: Encode,
{
    let storage_key_hex = get_storage_plain_key(module_name, storage_name);
    let mut map_keys_hash: Vec<u8> = Vec::new();
    let mut key_hash_pair =
        get_double_map_key_hash(metadata, module_name, storage_name, key_1, key_2);
    map_keys_hash.append(&mut key_hash_pair.0);
    map_keys_hash.append(&mut key_hash_pair.1);
    let map_keys_hash_hex: String = hex::encode(map_keys_hash);
    format!("{storage_key_hex}{map_keys_hash_hex}")
}

/// Get JSONRPSee parameters for a double-map storage type at an optional given block.
/// Will get current storage if `None` is supplied for `block_hash`.
pub fn _get_rpc_storage_double_map_params<T, U>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    storage_name: &str,
    key_1: &T,
    key_2: &U,
    block_hash: Option<&str>,
) -> ArrayParams
where
    T: Encode,
    U: Encode,
{
    let mut params = ArrayParams::new();
    params
        .insert(_get_storage_double_map_key(
            metadata,
            module_name,
            storage_name,
            key_1,
            key_2,
        ))
        .unwrap();
    if let Some(block_hash) = block_hash {
        params.insert(block_hash).unwrap();
    }
    params
}
