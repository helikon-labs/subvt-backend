use num_format::{Locale, ToFormattedString};

pub fn format_decimal(value: u128, decimals: usize, decimal_points: usize) -> String {
    let mut formatted = value.to_string();
    if formatted.len() < (decimals + 1) {
        formatted = format!(
            "{}{}",
            "0".repeat(decimals - formatted.len() + 1),
            formatted,
        );
    }
    let decimal_start_index = formatted.len() - decimals;
    let decimal_str = &formatted[decimal_start_index..(decimal_start_index + decimal_points)];
    let integer: u128 = formatted[0..decimal_start_index].parse().unwrap();
    format!(
        "{}{}{}",
        integer.to_formatted_string(&Locale::en),
        Locale::en.decimal(),
        decimal_str,
    )
}
