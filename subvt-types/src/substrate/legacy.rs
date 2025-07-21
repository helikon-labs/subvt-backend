//! Types to support the older metadata/runtime versions.
use frame_support::dispatch::{DispatchClass, Pays};
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::Decode;
use polkadot_core_primitives::BlockNumber;
use sp_runtime::{ArithmeticError, DispatchError, ModuleError, Perbill, TokenError};

pub type ValidatorIndex = u16;
pub type NominatorIndex = u32;
pub type ExtendedBalance = u128;

#[derive(Clone, Debug, Decode)]
pub struct OldWeight(pub u64);

#[derive(Clone, Debug, Decode)]
pub struct LegacyDispatchInfo {
    pub weight: OldWeight,
    pub class: DispatchClass,
    pub pays_fee: Pays,
}

#[derive(Clone, Debug, Decode)]
pub struct LegacyDispatchInfo2 {
    pub call_weight: Weight,
    pub class: DispatchClass,
    pub pays_fee: Pays,
}

#[derive(Clone, Debug, Decode)]
pub struct LegacyValidatorPrefs {
    #[codec(compact)]
    pub commission: Perbill,
}

#[derive(Clone, Debug, Decode)]
pub enum LegacyDispatchError {
    Other(#[codec(skip)] &'static str),
    /// Failed to lookup some data.
    CannotLookup,
    /// A bad origin.
    BadOrigin,
    /// A custom error in a module.
    Module {
        /// Module index, matching the metadata module index.
        index: u8,
        /// Module specific error value.
        error: u8,
        /// Optional error message.
        #[codec(skip)]
        message: Option<&'static str>,
    },
    /// At least one consumer is remaining so the account cannot be destroyed.
    ConsumerRemaining,
    /// There are no providers so the account cannot be created.
    NoProviders,
    /// There are too many consumers so the account cannot be created.
    TooManyConsumers,
    /// An error to do with tokens.
    Token(TokenError),
    /// An arithmetic error.
    Arithmetic(ArithmeticError),
}

impl From<LegacyDispatchError> for DispatchError {
    fn from(legacy_error: LegacyDispatchError) -> Self {
        match legacy_error {
            LegacyDispatchError::Other(error) => DispatchError::Other(error),
            LegacyDispatchError::CannotLookup => DispatchError::CannotLookup,
            LegacyDispatchError::BadOrigin => DispatchError::BadOrigin,
            LegacyDispatchError::Module {
                index,
                error,
                message,
            } => DispatchError::Module(ModuleError {
                index,
                error: [error, 0, 0, 0],
                message,
            }),
            LegacyDispatchError::ConsumerRemaining => DispatchError::ConsumerRemaining,
            LegacyDispatchError::NoProviders => DispatchError::NoProviders,
            LegacyDispatchError::TooManyConsumers => DispatchError::TooManyConsumers,
            LegacyDispatchError::Token(error) => DispatchError::Token(error),
            LegacyDispatchError::Arithmetic(error) => DispatchError::Arithmetic(error),
        }
    }
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub enum LegacyCoreOccupied {
    /// No candidate is waiting availability on this core right now (the core is not occupied).
    Free,
    /// A para is currently waiting for availability/inclusion on this core.
    Paras(LegacyParasEntry),
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct LegacyParasEntry {
    /// The underlying `Assignment`
    pub assignment: LegacyCoreAssignment,
    /// The number of times the entry has timed out in availability already.
    pub availability_timeouts: u32,
    /// The block height until this entry needs to be backed.
    ///
    /// If missed the entry will be removed from the claim queue without ever having occupied
    /// the core.
    pub ttl: BlockNumber,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct LegacyCoreAssignment {
    /// Assignment's ParaId
    pub para_id: ParaId,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct ParaId(pub u32);
