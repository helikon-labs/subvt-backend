use parity_scale_codec::{Encode, Decode};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use std::convert::{From, TryInto, TryFrom};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Encode, Default, Decode, Eq, Hash, PartialEq)]
pub struct AccountId([u8; 32]);

impl AccountId {
    pub fn to_ss58_check(&self) -> String {
        sp_core::crypto::AccountId32::new(self.0).to_ss58check()
    }
}

impl Display for AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl FromStr for AccountId {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let array: [u8; 32] = hex::decode(s)?.try_into().unwrap();
        Ok(AccountId(array))
    }
}

impl AccountId {
    pub const fn new(inner: [u8; 32]) -> Self {
        Self(inner)
    }
}

impl From<[u8; 32]> for AccountId {
    fn from(x: [u8; 32]) -> Self {
        Self::new(x)
    }
}

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl<'a> TryFrom<&'a [u8]> for AccountId {
    type Error = ();
    fn try_from(x: &'a [u8]) -> Result<AccountId, ()> {
        if x.len() == 32 {
            let mut r = AccountId::default();
            r.0.copy_from_slice(x);
            Ok(r)
        } else {
            Err(())
        }
    }
}

impl Serialize for AccountId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}