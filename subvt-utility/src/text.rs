pub fn get_condensed_address(address: &str) -> String {
    format!("{}...{}", &address[..5], &address[(address.len() - 5)..],)
}

pub fn get_condensed_session_keys(session_keys: &str) -> String {
    format!(
        "{}...{}",
        &session_keys[..8],
        &session_keys[(session_keys.len() - 6)..],
    )
}
