use crate::substrate::error::DecodeError;
use crate::substrate::legacy::{LegacyDispatchError, LegacyDispatchInfo};
use frame_metadata::{v14::StorageHasher, RuntimeMetadataV14};
use frame_support::dispatch::{DispatchInfo, DispatchResult, Weight};
use frame_support::weights::OldWeight;
use parity_scale_codec::{Compact, Decode};
use scale_info::form::PortableForm;
use scale_info::{Type, TypeDefPrimitive, Variant};
use sp_core::U256;
use sp_runtime::DispatchError;
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
    &metadata
        .types
        .types
        .iter()
        .find(|metadata_ty| metadata_ty.id == type_id)
        .unwrap()
        .ty
}

pub fn print_metadata_type_codes(metadata: &RuntimeMetadataV14) -> anyhow::Result<()> {
    for pallet in &metadata.pallets {
        println!("{}", pallet.name);
        if let Some(pallet_event_type) = &pallet.event {
            let event_type = metadata
                .types
                .types
                .iter()
                .find(|ty| ty.id == pallet_event_type.ty.id)
                .unwrap();
            match &event_type.ty.type_def {
                scale_info::TypeDef::Variant(variant) => {
                    println!("    {} events", variant.variants.len());
                    for event_variant in &variant.variants {
                        println!(
                            "        {}.{} {}",
                            pallet.name,
                            event_variant.name,
                            get_variant_type_code(metadata, event_variant)?,
                        );
                    }
                }
                _ => {
                    return Err(DecodeError::Error(format!(
                        "Unexpected non-variant event type: {:?}",
                        event_type.ty.type_def,
                    ))
                    .into())
                }
            };
        } else {
            println!("    0 events");
        }
        if let Some(pallet_call_type) = &pallet.calls {
            let call_type = metadata
                .types
                .types
                .iter()
                .find(|ty| ty.id == pallet_call_type.ty.id)
                .unwrap();
            match &call_type.ty.type_def {
                scale_info::TypeDef::Variant(variant) => {
                    println!("    {} calls", variant.variants.len());
                    for event_variant in &variant.variants {
                        println!(
                            "        {}.{} {}",
                            pallet.name,
                            event_variant.name,
                            get_variant_type_code(metadata, event_variant)?,
                        );
                    }
                }
                _ => {
                    return Err(DecodeError::Error(format!(
                        "Unexpected non-variant call type: {:?}",
                        call_type.ty.type_def,
                    ))
                    .into())
                }
            };
        }
    }
    Ok(())
}

pub(crate) fn decode_field(
    metadata: &RuntimeMetadataV14,
    field_type: &Type<PortableForm>,
    bytes: &mut &[u8],
    is_compact: bool,
) -> anyhow::Result<()> {
    match &field_type.type_def {
        scale_info::TypeDef::Primitive(primitive_type_def) => {
            if is_compact {
                decode_compact_primitive(primitive_type_def, bytes)?;
            } else {
                decode_primitive(primitive_type_def, bytes)?;
            }
        }
        scale_info::TypeDef::Composite(composite_type_def) => {
            for field in &composite_type_def.fields {
                let field_type = get_metadata_type(metadata, field.ty.id);
                decode_field(metadata, field_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Array(array_type_def) => {
            let array_type = get_metadata_type(metadata, array_type_def.type_param.id);
            for _ in 0..array_type_def.len {
                decode_field(metadata, array_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Tuple(tuple_type_def) => {
            for field_type_id in &tuple_type_def.fields {
                let field_type = get_metadata_type(metadata, field_type_id.id);
                decode_field(metadata, field_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Compact(compact_type_def) => {
            let compact_type = get_metadata_type(metadata, compact_type_def.type_param.id);
            decode_field(metadata, compact_type, bytes, true)?;
        }
        scale_info::TypeDef::Variant(variant_type_def) => {
            let index: u8 = Decode::decode(bytes)?;
            let variant = &variant_type_def
                .variants
                .iter()
                .find(|v| v.index == index)
                .unwrap();
            for field in &variant.fields {
                let field_type = get_metadata_type(metadata, field.ty.id);
                decode_field(metadata, field_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::Sequence(sequence_type_def) => {
            // get length (usize?)
            let compact_length: Compact<u32> = Decode::decode(bytes)?;
            let length = compact_length.0;
            let sequence_type = get_metadata_type(metadata, sequence_type_def.type_param.id);
            for _ in 0..length {
                decode_field(metadata, sequence_type, bytes, is_compact)?;
            }
        }
        scale_info::TypeDef::BitSequence(bit_sequence) => {
            let bit_store_type = &metadata.types.types[bit_sequence.bit_store_type.id as usize].ty;
            let bit_order_type = &metadata.types.types[bit_sequence.bit_order_type.id as usize].ty;
            decode_bit_sequence(bit_store_type, bit_order_type, bytes)?;
        }
    }
    Ok(())
}

fn decode_bit_sequence(
    bit_store_type: &Type<PortableForm>,
    bit_order_type: &Type<PortableForm>,
    bytes: &mut &[u8],
) -> anyhow::Result<()> {
    let bit_order_type_path = bit_order_type.path.segments.join("::");
    match &bit_store_type.type_def {
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
                _ => {
                    return Err(DecodeError::Error(format!(
                        "Unexpected bit sequence primitive: {:?}",
                        ty,
                    ))
                    .into())
                }
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
                _ => {
                    return Err(DecodeError::Error(format!(
                        "Unexpected bit sequence primitive: {:?}",
                        ty,
                    ))
                    .into())
                }
            },
            _ => {
                return Err(DecodeError::Error(format!(
                    "Unexpected bit sequence order: {}",
                    bit_order_type_path,
                ))
                .into())
            }
        },
        _ => {
            return Err(
                DecodeError::Error("Non-primitive type fed for bit sequence.".to_string()).into(),
            )
        }
    }
    Ok(())
}

fn decode_primitive(type_def: &TypeDefPrimitive, bytes: &mut &[u8]) -> anyhow::Result<()> {
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

fn decode_compact_primitive(type_def: &TypeDefPrimitive, bytes: &mut &[u8]) -> anyhow::Result<()> {
    match type_def {
        TypeDefPrimitive::Bool => {
            return Err(DecodeError::Error("No compact for Bool.".to_string()).into());
        }
        TypeDefPrimitive::Char | TypeDefPrimitive::U8 => {
            let _value: Compact<u8> = Decode::decode(bytes)?;
        }
        TypeDefPrimitive::Str => {
            return Err(DecodeError::Error("No compact for Str.".to_string()).into());
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
            return Err(DecodeError::Error("No compact for U256.".to_string()).into());
        }
        TypeDefPrimitive::I8 => {
            return Err(DecodeError::Error("No compact for I8.".to_string()).into());
        }
        TypeDefPrimitive::I16 => {
            return Err(DecodeError::Error("No compact for I16.".to_string()).into());
        }
        TypeDefPrimitive::I32 => {
            return Err(DecodeError::Error("No compact for I32.".to_string()).into());
        }
        TypeDefPrimitive::I64 => {
            return Err(DecodeError::Error("No compact for I64.".to_string()).into());
        }
        TypeDefPrimitive::I128 => {
            return Err(DecodeError::Error("No compact for I128.".to_string()).into());
        }
        TypeDefPrimitive::I256 => {
            return Err(DecodeError::Error("No compact for I256.".to_string()).into());
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

pub fn get_variant_type_code(
    metadata: &RuntimeMetadataV14,
    variant: &Variant<PortableForm>,
) -> Result<String, DecodeError> {
    let mut code = String::new();
    code.push('{');
    for (i, field) in variant.fields.iter().enumerate() {
        let field_type = get_metadata_type(metadata, field.ty.id);
        code.push_str(&get_type_code(metadata, field_type)?);
        if i < (variant.fields.len() - 1) {
            code.push(',');
        }
    }
    code.push('}');
    Ok(code)
}

fn get_type_code(
    metadata: &RuntimeMetadataV14,
    ty: &Type<PortableForm>,
) -> Result<String, DecodeError> {
    let mut code = String::new();
    match &ty.type_def {
        scale_info::TypeDef::Primitive(primitive_type_def) => {
            code.push_str(&get_primitive_type_code(primitive_type_def)?);
        }
        scale_info::TypeDef::Composite(composite_type_def) => {
            code.push('{');
            for (i, field) in composite_type_def.fields.iter().enumerate() {
                let field_type = get_metadata_type(metadata, field.ty.id);
                let field_type_path = field_type.path.segments.join("::");
                if field_type_path.is_empty() {
                    code.push_str(&get_type_code(metadata, field_type)?);
                } else if field_type_path.ends_with("Call") {
                    code.push_str("Call");
                } else {
                    code.push_str(&field_type_path);
                }
                if i < (composite_type_def.fields.len() - 1) {
                    code.push(',');
                }
            }
            code.push('}');
        }
        scale_info::TypeDef::Array(array_type_def) => {
            code.push('[');
            let array_type = get_metadata_type(metadata, array_type_def.type_param.id);
            let array_type_path = array_type.path.segments.join("::");
            if array_type_path.is_empty() {
                code.push_str(&get_type_code(metadata, array_type)?);
            } else if array_type_path.ends_with("Call") {
                code.push_str("Call");
            } else {
                code.push_str(&array_type_path);
            }
            code.push_str(format!(";{}]", array_type_def.len).as_str());
        }
        scale_info::TypeDef::Tuple(tuple_type_def) => {
            code.push('(');
            for (i, field_type_id) in tuple_type_def.fields.iter().enumerate() {
                let field_type = get_metadata_type(metadata, field_type_id.id);
                let field_type_path = field_type.path.segments.join("::");
                if field_type_path.is_empty() {
                    code.push_str(&get_type_code(metadata, field_type)?);
                } else if field_type_path.ends_with("Call") {
                    code.push_str("Call");
                } else {
                    code.push_str(&field_type_path);
                }
                if i < (tuple_type_def.fields.len() - 1) {
                    code.push(',');
                }
            }
            code.push(')');
        }
        scale_info::TypeDef::Compact(compact_type_def) => {
            code.push_str("compact<");
            let compact_type = get_metadata_type(metadata, compact_type_def.type_param.id);
            let compact_type_path = compact_type.path.segments.join("::");
            if compact_type_path.is_empty() {
                code.push_str(&get_type_code(metadata, compact_type)?);
            } else if compact_type_path.ends_with("Call") {
                code.push_str("Call");
            } else {
                code.push_str(&compact_type_path);
            }
            code.push('>');
        }
        scale_info::TypeDef::Variant(variant_type_def) => {
            code.push_str("var{");
            for (i, variant) in variant_type_def.variants.iter().enumerate() {
                code.push_str(&variant.name);
                if !variant.fields.is_empty() {
                    code.push('{');
                    for (j, field) in variant.fields.iter().enumerate() {
                        let field_type = get_metadata_type(metadata, field.ty.id);
                        let field_type_path = field_type.path.segments.join("::");
                        if field_type_path.is_empty() {
                            code.push_str(&get_type_code(metadata, field_type)?);
                        } else if field_type_path.ends_with("Call") {
                            code.push_str("Call");
                        } else {
                            code.push_str(&field_type_path);
                        }
                        if j < (variant.fields.len() - 1) {
                            code.push(',');
                        }
                    }
                    code.push('}');
                }
                if i < (variant_type_def.variants.len() - 1) {
                    code.push(',');
                }
            }
            code.push('}');
        }
        scale_info::TypeDef::Sequence(sequence_type_def) => {
            code.push_str("seq<");
            let sequence_type = get_metadata_type(metadata, sequence_type_def.type_param.id);
            let sequence_type_path = sequence_type.path.segments.join("::");
            if sequence_type_path.is_empty() {
                code.push_str(&get_type_code(metadata, sequence_type)?);
            } else if sequence_type_path.ends_with("Call") {
                code.push_str("Call");
            } else {
                code.push_str(&sequence_type_path);
            }
            code.push('>');
        }
        scale_info::TypeDef::BitSequence(bit_sequence) => {
            let bit_store_type = &metadata.types.types[bit_sequence.bit_store_type.id as usize].ty;
            let bit_order_type = &metadata.types.types[bit_sequence.bit_order_type.id as usize].ty;
            code.push_str(&get_bit_sequence_type_code(bit_store_type, bit_order_type)?);
        }
    }
    Ok(code)
}

fn get_primitive_type_code(type_def: &TypeDefPrimitive) -> Result<String, DecodeError> {
    let code = match type_def {
        TypeDefPrimitive::Bool => "bool",
        TypeDefPrimitive::Str => "string",
        TypeDefPrimitive::Char => "char",
        TypeDefPrimitive::U8 => "u8",
        TypeDefPrimitive::U16 => "u16",
        TypeDefPrimitive::U32 => "u32",
        TypeDefPrimitive::U64 => "u64",
        TypeDefPrimitive::U128 => "u128",
        TypeDefPrimitive::U256 => "u256",
        TypeDefPrimitive::I8 => "i8",
        TypeDefPrimitive::I16 => "i16",
        TypeDefPrimitive::I32 => "i32",
        TypeDefPrimitive::I64 => "i64",
        TypeDefPrimitive::I128 => "i128",
        TypeDefPrimitive::I256 => "i256",
    };
    Ok(code.to_string())
}

fn get_bit_sequence_type_code(
    bit_store_type: &Type<PortableForm>,
    bit_order_type: &Type<PortableForm>,
) -> Result<String, DecodeError> {
    let mut code = String::new();
    code.push_str("bs<");
    let bit_order_type_path = bit_order_type.path.segments.join("::");
    match bit_order_type_path.as_str() {
        "bitvec::order::Lsb0" => code.push_str("lsb0,"),
        "bitvec::order::Msb0" => code.push_str("msb0,"),
        _ => {
            return Err(DecodeError::Error(format!(
                "Unexpected bit sequence order: {}",
                bit_order_type_path
            )))
        }
    }
    match &bit_store_type.type_def {
        scale_info::TypeDef::Primitive(ty) => match ty {
            TypeDefPrimitive::U8 => code.push_str("u8"),
            TypeDefPrimitive::U16 => code.push_str("u16"),
            TypeDefPrimitive::U32 => code.push_str("u32"),
            TypeDefPrimitive::U64 => code.push_str("u64"),
            TypeDefPrimitive::U128 => code.push_str("u128"),
            _ => {
                return Err(DecodeError::Error(format!(
                    "Unexpected bit sequence primitive: {:?}",
                    ty
                )))
            }
        },
        _ => {
            return Err(DecodeError::Error(
                "Non-primitive type fed for bit sequence.".to_string(),
            ))
        }
    }
    code.push('>');
    Ok(code)
}

pub fn is_dispatch_error_legacy(runtime_version: u32) -> bool {
    runtime_version < 9190
}

pub fn is_weight_legacy(runtime_version: u32) -> bool {
    runtime_version <= 9300
}

pub fn decode_dispatch_result(
    runtime_version: u32,
    bytes: &mut &[u8],
) -> Result<DispatchResult, DecodeError> {
    if is_dispatch_error_legacy(runtime_version) {
        let legacy_result: Result<(), LegacyDispatchError> =
            Decode::decode(&mut *bytes).map_err(|error| {
                DecodeError::Error(format!("Cannot decode legacy dispatch result: {error:?}",))
            })?;
        let dispatch_result: DispatchResult = match legacy_result {
            Ok(()) => Ok(()),
            Err(legacy_error) => Err(legacy_error.into()),
        };
        Ok(dispatch_result)
    } else {
        Ok(DispatchResult::decode(&mut *bytes).map_err(|error| {
            DecodeError::Error(format!("Cannot decode dispatch result: {error:?}",))
        })?)
    }
}

pub fn decode_dispatch_error(
    runtime_version: u32,
    bytes: &mut &[u8],
) -> anyhow::Result<DispatchError, DecodeError> {
    if is_dispatch_error_legacy(runtime_version) {
        match LegacyDispatchError::decode(&mut *bytes) {
            Ok(legacy_dispatch_error) => Ok(legacy_dispatch_error.into()),
            Err(error) => Err(DecodeError::Error(format!(
                "Cannot decode legacy dispatch error: {:?}",
                error
            ))),
        }
    } else {
        match DispatchError::decode(&mut *bytes) {
            Ok(dispatch_error) => Ok(dispatch_error),
            Err(error) => Err(DecodeError::Error(format!(
                "Cannot decode dispatch error: {:?}",
                error
            ))),
        }
    }
}

pub fn decode_dispatch_info(
    runtime_version: u32,
    bytes: &mut &[u8],
) -> anyhow::Result<DispatchInfo, DecodeError> {
    if is_weight_legacy(runtime_version) {
        let legacy_dispatch_info: LegacyDispatchInfo =
            Decode::decode(&mut *bytes).map_err(|error| {
                DecodeError::Error(format!("Cannot decode legacy dispatch info: {error:?}",))
            })?;
        let dispatch_info = DispatchInfo {
            weight: legacy_dispatch_info.weight.into(),
            class: legacy_dispatch_info.class,
            pays_fee: legacy_dispatch_info.pays_fee,
        };
        Ok(dispatch_info)
    } else {
        Ok(DispatchInfo::decode(&mut *bytes).map_err(|error| {
            DecodeError::Error(format!("Cannot decode dispatch info: {error:?}",))
        })?)
    }
}

pub fn decode_weight(
    runtime_version: u32,
    bytes: &mut &[u8],
) -> anyhow::Result<Weight, DecodeError> {
    if is_weight_legacy(runtime_version) {
        let old_weight: OldWeight = Decode::decode(bytes)?;
        Ok(old_weight.into())
    } else {
        Ok(Decode::decode(bytes)?)
    }
}
