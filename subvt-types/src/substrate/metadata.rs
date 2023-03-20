use frame_metadata::{v14::StorageHasher, RuntimeMetadataV14};
use parity_scale_codec::{Compact, Decode};
use scale_info::form::PortableForm;
use scale_info::{Type, TypeDefPrimitive};
use sp_core::U256;
use subxt::utils::bits::{DecodedBits, Lsb0, Msb0};

pub fn hash(hasher: &StorageHasher, bytes: &[u8]) -> Vec<u8> {
    match hasher {
        StorageHasher::Identity => bytes.to_vec(),
        StorageHasher::Blake2_128 => sp_core::blake2_128(bytes).to_vec(),
        StorageHasher::Blake2_128Concat => sp_core::blake2_128(bytes)
            .iter()
            .chain(bytes)
            .cloned()
            .collect(),
        StorageHasher::Blake2_256 => sp_core::blake2_256(bytes).to_vec(),
        StorageHasher::Twox128 => sp_core::twox_128(bytes).to_vec(),
        StorageHasher::Twox256 => sp_core::twox_256(bytes).to_vec(),
        StorageHasher::Twox64Concat => sp_core::twox_64(bytes)
            .iter()
            .chain(bytes)
            .cloned()
            .collect(),
    }
}

pub(crate) fn get_metadata_type(
    metadata: &RuntimeMetadataV14,
    type_id: u32,
) -> &Type<PortableForm> {
    metadata
        .types
        .types()
        .iter()
        .find(|metadata_ty| metadata_ty.id() == type_id)
        .unwrap()
        .ty()
}

pub(crate) fn decode_field(
    metadata: &RuntimeMetadataV14,
    field_type: &Type<PortableForm>,
    bytes: &mut &[u8],
    is_compact: bool,
) -> anyhow::Result<()> {
    match field_type.type_def() {
        scale_info::TypeDef::Primitive(primitive_type_def) => {
            if is_compact {
                decode_compact_primitive(primitive_type_def, bytes)?;
            } else {
                decode_primitive(primitive_type_def, bytes)?;
            }
        }
        scale_info::TypeDef::Composite(composite_type_def) => {
            for field in composite_type_def.fields() {
                let field_type = get_metadata_type(metadata, field.ty().id());
                decode_field(metadata, field_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Array(array_type_def) => {
            let array_type = get_metadata_type(metadata, array_type_def.type_param().id());
            for _ in 0..array_type_def.len() {
                decode_field(metadata, array_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Tuple(tuple_type_def) => {
            for field_type_id in tuple_type_def.fields() {
                let field_type = get_metadata_type(metadata, field_type_id.id());
                decode_field(metadata, field_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Compact(compact_type_def) => {
            let compact_type = get_metadata_type(metadata, compact_type_def.type_param().id());
            decode_field(metadata, compact_type, bytes, true)?;
        }
        scale_info::TypeDef::Variant(variant_type_def) => {
            let index: u8 = Decode::decode(bytes)?;
            let variant = &variant_type_def
                .variants()
                .iter()
                .find(|v| v.index == index)
                .unwrap();
            for field in variant.fields() {
                let field_type = get_metadata_type(metadata, field.ty().id());
                decode_field(metadata, field_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Sequence(sequence_type_def) => {
            // get length (usize?)
            let compact_length: Compact<u32> = Decode::decode(bytes)?;
            let length = compact_length.0;
            let sequence_type = get_metadata_type(metadata, sequence_type_def.type_param().id());
            for _ in 0..length {
                decode_field(metadata, sequence_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::BitSequence(bit_sequence) => {
            let bit_store_type =
                metadata.types.types()[bit_sequence.bit_store_type().id() as usize].ty();
            let bit_order_type =
                metadata.types.types()[bit_sequence.bit_order_type().id() as usize].ty();
            decode_bit_sequence(bit_store_type, bit_order_type, bytes)?;
        }
    }
    Ok(())
}

pub(crate) fn decode_bit_sequence(
    bit_store_type: &Type<PortableForm>,
    bit_order_type: &Type<PortableForm>,
    bytes: &mut &[u8],
) -> anyhow::Result<()> {
    let bit_order_type_path = bit_order_type.path().segments().join("::");
    match bit_store_type.type_def() {
        scale_info::TypeDef::Primitive(ty) => match bit_order_type_path.as_str() {
            "bitvec::order::Lsb0" => match ty {
                TypeDefPrimitive::U8 => {
                    let _: DecodedBits<u8, Lsb0> = Decode::decode(bytes)?;
                }
                TypeDefPrimitive::U16 => {
                    let _: DecodedBits<u16, Lsb0> = Decode::decode(bytes)?;
                }
                TypeDefPrimitive::U32 => {
                    let _: DecodedBits<u32, Lsb0> = Decode::decode(bytes)?;
                }
                TypeDefPrimitive::U64 => {
                    let _: DecodedBits<u64, Lsb0> = Decode::decode(bytes)?;
                }
                _ => panic!("Unexpected bit sequence primitive: {:?}", ty),
            },
            "bitvec::order::Msb0" => match ty {
                TypeDefPrimitive::U8 => {
                    let _: DecodedBits<u8, Msb0> = Decode::decode(bytes)?;
                }
                TypeDefPrimitive::U16 => {
                    let _: DecodedBits<u16, Msb0> = Decode::decode(bytes)?;
                }
                TypeDefPrimitive::U32 => {
                    let _: DecodedBits<u32, Msb0> = Decode::decode(bytes)?;
                }
                TypeDefPrimitive::U64 => {
                    let _: DecodedBits<u64, Msb0> = Decode::decode(bytes)?;
                }
                _ => panic!("Unexpected bit sequence primitive: {:?}", ty),
            },
            _ => panic!("Unexpected bit sequence order: {}", bit_order_type_path),
        },
        _ => panic!("Non-primitive type fed for bit sequence."),
    }
    Ok(())
}

pub(crate) fn decode_primitive(
    type_def: &TypeDefPrimitive,
    bytes: &mut &[u8],
) -> anyhow::Result<()> {
    match type_def {
        TypeDefPrimitive::Bool => {
            let _value: bool = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::Str => {
            let _value: String = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::Char | TypeDefPrimitive::U8 => {
            let _value: u8 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U16 => {
            let _value: u16 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U32 => {
            let _value: u32 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U64 => {
            let _value: u64 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U128 => {
            let _value: u128 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U256 => {
            let _value: U256 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::I8 => {
            let _value: i8 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::I16 => {
            let _value: i16 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::I32 => {
            let _value: i32 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::I64 => {
            let _value: i64 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::I128 => {
            let _value: i128 = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::I256 => {
            let _value: [u8; 32] = Decode::decode(bytes)?;
        }
    }
    Ok(())
}

pub(crate) fn decode_compact_primitive(
    type_def: &TypeDefPrimitive,
    bytes: &mut &[u8],
) -> anyhow::Result<()> {
    match type_def {
        TypeDefPrimitive::Bool => {
            panic!("No Compact for Bool.");
        }
        TypeDefPrimitive::Char | TypeDefPrimitive::U8 => {
            let _value: Compact<u8> = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::Str => {
            panic!("No Compact for Str.");
        }
        TypeDefPrimitive::U16 => {
            let _value: Compact<u16> = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U32 => {
            let _value: Compact<u32> = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U64 => {
            let _value: Compact<u64> = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U128 => {
            let _value: Compact<u128> = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::U256 => {
            panic!("No Compact for U256.");
        }
        TypeDefPrimitive::I8 => {
            panic!("No Compact for I8.");
        }
        TypeDefPrimitive::I16 => {
            panic!("No Compact for I16.");
        }
        TypeDefPrimitive::I32 => {
            panic!("No Compact for I32.");
        }
        TypeDefPrimitive::I64 => {
            panic!("No Compact for I64.");
        }
        TypeDefPrimitive::I128 => {
            panic!("No Compact for I128.");
        }
        TypeDefPrimitive::I256 => {
            panic!("No Compact for I256.");
        }
    }
    Ok(())
}

pub fn get_metadata_constant<V: Decode>(
    metadata: &RuntimeMetadataV14,
    module_name: &str,
    constant_name: &str,
) -> anyhow::Result<V> {
    let bytes = &metadata
        .pallets
        .iter()
        .find(|p| p.name == module_name)
        .unwrap()
        .constants
        .iter()
        .find(|c| c.name == constant_name)
        .unwrap()
        .value;
    Ok(Decode::decode(&mut &bytes[..])?)
}

pub fn get_metadata_expected_block_time_millis(
    metadata: &RuntimeMetadataV14,
) -> anyhow::Result<u64> {
    get_metadata_constant(metadata, "Babe", "ExpectedBlockTime")
}

pub fn get_metadata_epoch_duration_millis(metadata: &RuntimeMetadataV14) -> anyhow::Result<u64> {
    let expected_block_time_millis: u64 = get_metadata_expected_block_time_millis(metadata)?;
    let epoch_duration_blocks: u64 = get_metadata_constant(metadata, "Babe", "EpochDuration")?;
    Ok(epoch_duration_blocks * expected_block_time_millis)
}

pub fn get_metadata_era_duration_millis(metadata: &RuntimeMetadataV14) -> anyhow::Result<u64> {
    let sessions_per_era: u32 = get_metadata_constant(metadata, "Staking", "SessionsPerEra")?;
    let epoch_duration_blocks: u64 = get_metadata_constant(metadata, "Babe", "EpochDuration")?;
    let expected_block_time_millis: u64 = get_metadata_expected_block_time_millis(metadata)?;
    let era_duration_blocks = epoch_duration_blocks * sessions_per_era as u64;
    Ok(era_duration_blocks * expected_block_time_millis)
}
