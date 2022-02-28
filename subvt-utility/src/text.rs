pub fn get_condensed_address(address: &str, side_limit: Option<usize>) -> String {
    if let Some(side_limit) = side_limit {
        format!(
            "{}...{}",
            &address[..side_limit],
            &address[(address.len() - side_limit)..],
        )
    } else {
        format!("{}...{}", &address[..5], &address[(address.len() - 5)..],)
    }
}

pub fn get_condensed_session_keys(session_keys: &str) -> String {
    format!(
        "{}...{}",
        &session_keys[..8],
        &session_keys[(session_keys.len() - 6)..],
    )
}
