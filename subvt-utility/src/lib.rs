//! Utility functions.

use parity_scale_codec::Decode;

pub fn decode_hex_string<T>(hex_string: &str) -> anyhow::Result<T>
    where
        T: Decode,
{
    let trimmed_hex_string = hex_string.trim_start_matches("0x");
    let mut bytes: &[u8] = &hex::decode(&trimmed_hex_string).unwrap();
    let decoded = Decode::decode(&mut bytes)?;
    Ok(decoded)
}

/*
pub fn get_validator_index(consensus_engine: &str,
                           bytes: &mut &[u8]) -> AccountId {
    match consensus_engine {
        "BABE" => {
            let mut data_vec_bytes: &[u8] = &bytes[..];
            let digest: PreDigest = Decode::decode(&mut data_vec_bytes).unwrap();
            let validator_index = match digest {
                PreDigest::Primary(digest) => digest.authority_index,
                PreDigest::SecondaryPlain(digest) => digest.authority_index,
                PreDigest::SecondaryVRF(digest) => digest.authority_index,
            };
            return validator_addresses[validator_index as usize].clone();
        }
        "aura" | "FRNK" | "pow_" => { // FRNK is GRANDPA
            panic!("Consensus engine [{}] not supported.", consensus_engine);
        }
        _ => {
            panic!("Unknown consensus engine [{}].", consensus_engine);
        }
    }
}
 */
