//! Parsing for the optional reproducible firmware build identifier.

pub fn parse_explicit(value: &str) -> Result<u32, String> {
    let value = value.trim();
    let parsed = if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    parsed.map_err(|_| format!("RMK_BUILD_HASH must be a decimal u32 or 0x-prefixed hexadecimal u32, got {value:?}"))
}

#[cfg(test)]
mod tests {
    use super::parse_explicit;

    #[test]
    fn accepts_decimal_and_hex() {
        assert_eq!(parse_explicit("305419896"), Ok(0x1234_5678));
        assert_eq!(parse_explicit("0x12345678"), Ok(0x1234_5678));
        assert_eq!(parse_explicit(" 0XFFFFFFFF "), Ok(u32::MAX));
    }

    #[test]
    fn rejects_invalid_or_out_of_range_values() {
        assert!(parse_explicit("").is_err());
        assert!(parse_explicit("0xgg").is_err());
        assert!(parse_explicit("4294967296").is_err());
    }
}
