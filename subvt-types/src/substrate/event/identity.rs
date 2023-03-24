use crate::crypto::AccountId;
use crate::substrate::error::DecodeError;
use crate::substrate::event::SubstrateEvent;
use crate::substrate::Balance;
use pallet_identity::RegistrarIndex;
use parity_scale_codec::Decode;

const IDENTITY_CLEARED: &str = "IdentityCleared";
const IDENTITY_KILLED: &str = "IdentityKilled";
const IDENTITY_SET: &str = "IdentitySet";
const JUDGEMENT_GIVEN: &str = "JudgementGiven";
const JUDGEMENT_REQUESTED: &str = "JudgementRequested";
const JUDGEMENT_UNREQUESTED: &str = "JudgementUnrequested";
const SUB_IDENTITY_ADDED: &str = "SubIdentityAdded";
const SUB_IDENTITY_REMOVED: &str = "SubIdentityRemoved";
const SUB_IDENTITY_REVOKED: &str = "SubIdentityRevoked";

#[derive(Clone, Debug)]
pub enum IdentityEvent {
    IdentityCleared {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        returned_balance: Balance,
    },
    IdentityKilled {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
        slashed_balance: Balance,
    },
    IdentitySet {
        extrinsic_index: Option<u32>,
        account_id: AccountId,
    },
    JudgementGiven {
        extrinsic_index: Option<u32>,
        target_account_id: AccountId,
        registrar_index: RegistrarIndex,
    },
    JudgementRequested {
        extrinsic_index: Option<u32>,
        target_account_id: AccountId,
        registrar_index: RegistrarIndex,
    },
    JudgementUnrequested {
        extrinsic_index: Option<u32>,
        target_account_id: AccountId,
        registrar_index: RegistrarIndex,
    },
    SubIdentityAdded {
        extrinsic_index: Option<u32>,
        sub_account_id: AccountId,
        main_account_id: AccountId,
        deposit: Balance,
    },
    SubIdentityRemoved {
        extrinsic_index: Option<u32>,
        sub_account_id: AccountId,
        main_account_id: AccountId,
        freed_deposit: Balance,
    },
    SubIdentityRevoked {
        extrinsic_index: Option<u32>,
        sub_account_id: AccountId,
        main_account_id: AccountId,
        repatriated_deposit: Balance,
    },
}

impl IdentityEvent {
    pub fn get_extrinsic_index(&self) -> Option<u32> {
        match self {
            Self::IdentityCleared {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::IdentityKilled {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::IdentitySet {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::JudgementGiven {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::JudgementRequested {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::JudgementUnrequested {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::SubIdentityAdded {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::SubIdentityRemoved {
                extrinsic_index, ..
            } => *extrinsic_index,
            Self::SubIdentityRevoked {
                extrinsic_index, ..
            } => *extrinsic_index,
        }
    }
}

impl IdentityEvent {
    pub fn decode(
        _runtime_version: u32,
        name: &str,
        extrinsic_index: Option<u32>,
        bytes: &mut &[u8],
    ) -> Result<Option<SubstrateEvent>, DecodeError> {
        let maybe_event = match name {
            IDENTITY_CLEARED => Some(SubstrateEvent::Identity(IdentityEvent::IdentityCleared {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                returned_balance: Decode::decode(bytes)?,
            })),
            IDENTITY_KILLED => Some(SubstrateEvent::Identity(IdentityEvent::IdentityKilled {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
                slashed_balance: Decode::decode(bytes)?,
            })),
            IDENTITY_SET => Some(SubstrateEvent::Identity(IdentityEvent::IdentitySet {
                extrinsic_index,
                account_id: Decode::decode(bytes)?,
            })),
            JUDGEMENT_GIVEN => Some(SubstrateEvent::Identity(IdentityEvent::JudgementGiven {
                extrinsic_index,
                target_account_id: Decode::decode(bytes)?,
                registrar_index: Decode::decode(bytes)?,
            })),
            JUDGEMENT_REQUESTED => Some(SubstrateEvent::Identity(
                IdentityEvent::JudgementRequested {
                    extrinsic_index,
                    target_account_id: Decode::decode(bytes)?,
                    registrar_index: Decode::decode(bytes)?,
                },
            )),
            JUDGEMENT_UNREQUESTED => Some(SubstrateEvent::Identity(
                IdentityEvent::JudgementUnrequested {
                    extrinsic_index,
                    target_account_id: Decode::decode(bytes)?,
                    registrar_index: Decode::decode(bytes)?,
                },
            )),
            SUB_IDENTITY_ADDED => Some(SubstrateEvent::Identity(IdentityEvent::SubIdentityAdded {
                extrinsic_index,
                sub_account_id: Decode::decode(bytes)?,
                main_account_id: Decode::decode(bytes)?,
                deposit: Decode::decode(bytes)?,
            })),
            SUB_IDENTITY_REMOVED => Some(SubstrateEvent::Identity(
                IdentityEvent::SubIdentityRemoved {
                    extrinsic_index,
                    sub_account_id: Decode::decode(bytes)?,
                    main_account_id: Decode::decode(bytes)?,
                    freed_deposit: Decode::decode(bytes)?,
                },
            )),
            SUB_IDENTITY_REVOKED => Some(SubstrateEvent::Identity(
                IdentityEvent::SubIdentityRevoked {
                    extrinsic_index,
                    sub_account_id: Decode::decode(bytes)?,
                    main_account_id: Decode::decode(bytes)?,
                    repatriated_deposit: Decode::decode(bytes)?,
                },
            )),
            _ => None,
        };
        Ok(maybe_event)
    }
}
