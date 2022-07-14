//! Era and validator report storage and types.
use std::str::FromStr;

pub mod block;
pub mod era;
pub mod era_validator;
pub mod para;
pub mod payouts;
pub mod rewards;

fn parse_maybe_string<T: FromStr>(maybe_string: &Option<String>) -> Result<Option<T>, T::Err> {
    if let Some(string) = maybe_string {
        Ok(Some(string.parse::<T>()?))
    } else {
        Ok(None)
    }
}
