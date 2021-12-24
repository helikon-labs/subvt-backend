use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use std::convert::{From, TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::substrate::error::DecodeError;

#[derive(Clone, Debug, Encode, Default, Decode, Eq, Hash, PartialEq)]
pub struct AccountId([u8; 32]);

impl AccountId {
    pub fn to_ss58_check(&self) -> String {
        sp_core::crypto::AccountId32::new(self.0).to_ss58check()
    }

    pub fn from_ss58_check(address: &str) -> Result<Self, DecodeError> {
        if let Ok(account_id) = sp_core::crypto::AccountId32::from_ss58check(address) {
            let account_id_bytes: [u8; 32] = account_id.into();
            Ok(Self(account_id_bytes))
        } else {
            Err(DecodeError::Error(format!(
                "Cannot get account id from SS58 encoded address {}.",
                address
            )))
        }
    }

    pub fn multisig_account_id(
        signatory: &AccountId,
        other_signatories: &[AccountId],
        threshold: u16,
    ) -> AccountId {
        let mut account_ids = vec![signatory];
        for other_signatory in other_signatories {
            account_ids.push(other_signatory);
        }
        let entropy =
            (b"modlpy/utilisuba", account_ids, threshold).using_encoded(sp_core::blake2_256);
        AccountId::from(entropy)
    }
}

impl Display for AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("0x")?;
        f.write_str(&hex::encode_upper(self.0))
    }
}

impl FromStr for AccountId {
    type Err = hex::FromHexError;

    fn from_str(hex_string: &str) -> Result<Self, Self::Err> {
        let trimmed_hex_string = hex_string.trim_start_matches("0x");
        hex::decode(trimmed_hex_string)?
            .try_into()
            .map(AccountId)
            .map_err(|_| hex::FromHexError::InvalidStringLength)
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
