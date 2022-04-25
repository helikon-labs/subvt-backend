//! Non-Substrate application events storage, such as new validator on network, 1KV rank change,
//! lost/new/changed nomination, etc.
pub mod active;
pub mod active_next_session;
pub mod identity_changed;
pub mod inactive;
pub mod inactive_next_session;
pub mod lost_nomination;
pub mod new_nomination;
pub mod new_validator;
pub mod nomination_amount_change;
pub mod onekv;
pub mod removed_validator;
pub mod session_keys_changed;
