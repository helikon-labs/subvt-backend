use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub type NFTCollection = HashMap<NFTChain, Vec<NFT>>;

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq)]
pub enum NFTChain {
    #[serde(rename = "acala")]
    Acala,
    #[serde(rename = "karura")]
    Karura,
    #[serde(rename = "rmrk1")]
    RMRK1,
    #[serde(rename = "rmrk2")]
    RMRK2,
    #[serde(rename = "statemine")]
    Statemine,
    #[serde(rename = "statemint")]
    Statemint,
    #[serde(rename = "unique")]
    Unique,
}

impl Display for NFTChain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Self::Acala => "acala",
            Self::Karura => "karura",
            Self::RMRK1 => "rmrk1",
            Self::RMRK2 => "rmrk2",
            Self::Statemine => "statemine",
            Self::Statemint => "statemint",
            Self::Unique => "unique",
        };
        write!(f, "{}", display)
    }
}

impl FromStr for NFTChain {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "acala" => Ok(Self::Acala),
            "karura" => Ok(Self::Karura),
            "rmrk1" => Ok(Self::RMRK1),
            "rmrk2" => Ok(Self::RMRK2),
            "statemine" => Ok(Self::Statemine),
            "statemint" => Ok(Self::Statemint),
            "unique" => Ok(Self::Unique),
            _ => panic!("Unknown NFT chain: {}", s),
        }
    }
}

impl NFTChain {
    pub fn name(&self) -> String {
        match self {
            NFTChain::Acala => "Acala",
            NFTChain::Karura => "Karura",
            NFTChain::RMRK1 => "RMRK1",
            NFTChain::RMRK2 => "RMRK2",
            NFTChain::Statemine => "Statemine",
            NFTChain::Statemint => "Statemint",
            NFTChain::Unique => "UNIQUE",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NFT {
    #[serde(deserialize_with = "str_or_i64")]
    pub id: String,
    pub content_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename(deserialize = "link"))]
    pub url: Option<String>,
    #[serde(rename(deserialize = "image"))]
    pub image_url: Option<String>,
}

fn str_or_i64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrI64<'a> {
        Str(&'a str),
        I64(i64),
    }

    Ok(match StrOrI64::deserialize(deserializer)? {
        StrOrI64::Str(v) => v.to_string(),
        StrOrI64::I64(v) => v.to_string(),
    })
}
