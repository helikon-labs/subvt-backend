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
