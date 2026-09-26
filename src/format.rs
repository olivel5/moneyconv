// Minor-unit exponents per ISO 4217. Anything not listed here defaults to 2,
// which covers the large majority of currencies people actually deal with.
pub fn exponent_for(code: &str) -> u32 {
    match code {
        "BIF" | "CLP" | "DJF" | "GNF" | "ISK" | "JPY" | "KMF" | "KRW" | "PYG" | "RWF" | "UGX"
        | "VND" | "VUV" | "XAF" | "XOF" | "XPF" => 0,
        "BHD" | "IQD" | "JOD" | "KWD" | "LYD" | "OMR" | "TND" => 3,
        _ => 2,
    }
}

// Parses a decimal amount ("19.99", "-3", "0.005") into an integer count of
// minor units for the given exponent. Refuses to parse an amount with more
// fractional digits than the currency allows, rather than silently rounding
// it away.
pub fn parse_decimal(input: &str, exponent: u32) -> Result<i64, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty amount".to_string());
    }

    let (negative, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };

    let mut split = rest.splitn(2, '.');
    let int_part = split.next().unwrap_or("");
    let frac_part = split.next().unwrap_or("");

    if int_part.is_empty() && frac_part.is_empty() {
        return Err(format!("not a number: {:?}", input));
    }
    if !int_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("not a number: {:?}", input));
    }
    if !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("not a number: {:?}", input));
    }
    if frac_part.len() > exponent as usize {
        return Err(format!(
            "{:?} has more decimal places than this currency allows ({})",
            input, exponent
        ));
    }

    let int_value: i64 = if int_part.is_empty() {
        0
    } else {
        int_part
            .parse()
            .map_err(|_| format!("amount out of range: {:?}", input))?
    };

    let mut frac_value: i64 = if frac_part.is_empty() {
        0
    } else {
        frac_part
            .parse()
            .map_err(|_| format!("amount out of range: {:?}", input))?
    };
    for _ in 0..(exponent as usize - frac_part.len()) {
        frac_value *= 10;
    }

    let scale = 10i64.pow(exponent);
    let magnitude = int_value
        .checked_mul(scale)
        .and_then(|v| v.checked_add(frac_value))
        .ok_or_else(|| format!("amount out of range: {:?}", input))?;

    Ok(if negative { -magnitude } else { magnitude })
}

// Formats a count of minor units back into a decimal string for the given
// exponent, e.g. (1999, 2) -> "19.99", (500, 0) -> "500".
pub fn format_minor(minor: i64, exponent: u32) -> String {
    if exponent == 0 {
        return minor.to_string();
    }

    let scale = 10u64.pow(exponent);
    let sign = if minor < 0 { "-" } else { "" };
    let magnitude = minor.unsigned_abs();
    let int_part = magnitude / scale;
    let frac_part = magnitude % scale;

    format!("{}{}.{:0width$}", sign, int_part, frac_part, width = exponent as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_decimal_plain() {
        assert_eq!(parse_decimal("19.99", 2), Ok(1999));
        assert_eq!(parse_decimal("0.05", 2), Ok(5));
        assert_eq!(parse_decimal("500", 0), Ok(500));
        assert_eq!(parse_decimal("12.500", 3), Ok(12500));
    }

    #[test]
    fn parse_decimal_pads_short_fraction() {
        // "19.9" for a 2-exponent currency means 19.90, not 19.09.
        assert_eq!(parse_decimal("19.9", 2), Ok(1990));
        assert_eq!(parse_decimal("19.1", 3), Ok(19100));
    }

    #[test]
    fn parse_decimal_signs() {
        assert_eq!(parse_decimal("-3", 2), Ok(-300));
        assert_eq!(parse_decimal("+3", 2), Ok(300));
        assert_eq!(parse_decimal("-0.01", 2), Ok(-1));
    }

    #[test]
    fn parse_decimal_missing_integer_or_fraction_part() {
        assert_eq!(parse_decimal(".5", 2), Ok(50));
        assert_eq!(parse_decimal("5.", 2), Ok(500));
    }

    #[test]
    fn parse_decimal_whitespace_is_trimmed() {
        assert_eq!(parse_decimal("  19.99  ", 2), Ok(1999));
    }

    #[test]
    fn parse_decimal_rejects_empty() {
        assert!(parse_decimal("", 2).is_err());
        assert!(parse_decimal("   ", 2).is_err());
        assert!(parse_decimal("-", 2).is_err());
        assert!(parse_decimal(".", 2).is_err());
    }

    #[test]
    fn parse_decimal_rejects_garbage() {
        assert!(parse_decimal("nineteen", 2).is_err());
        assert!(parse_decimal("19.9a", 2).is_err());
        assert!(parse_decimal("1,999.99", 2).is_err());
        assert!(parse_decimal("19..99", 2).is_err());
        assert!(parse_decimal("19.9.9", 2).is_err());
    }

    #[test]
    fn parse_decimal_rejects_too_many_fraction_digits() {
        assert!(parse_decimal("19.999", 2).is_err());
        assert!(parse_decimal("500.5", 0).is_err());
    }

    #[test]
    fn parse_decimal_rejects_overflow() {
        assert!(parse_decimal("99999999999999999999", 2).is_err());
    }

    #[test]
    fn format_minor_plain() {
        assert_eq!(format_minor(1999, 2), "19.99");
        assert_eq!(format_minor(5, 2), "0.05");
        assert_eq!(format_minor(500, 0), "500");
        assert_eq!(format_minor(12500, 3), "12.500");
    }

    #[test]
    fn format_minor_negative() {
        assert_eq!(format_minor(-1999, 2), "-19.99");
        assert_eq!(format_minor(-5, 2), "-0.05");
        assert_eq!(format_minor(-500, 0), "-500");
    }

    #[test]
    fn format_minor_zero() {
        assert_eq!(format_minor(0, 2), "0.00");
        assert_eq!(format_minor(0, 0), "0");
    }

    #[test]
    fn parse_then_format_roundtrip() {
        for (amount, exponent) in [("19.99", 2), ("500", 0), ("12.500", 3), ("-3.00", 2)] {
            let minor = parse_decimal(amount, exponent).unwrap();
            assert_eq!(format_minor(minor, exponent), amount);
        }
    }

    #[test]
    fn exponent_table_known_values() {
        assert_eq!(exponent_for("JPY"), 0);
        assert_eq!(exponent_for("KWD"), 3);
        assert_eq!(exponent_for("USD"), 2);
        assert_eq!(exponent_for("XYZ"), 2);
    }
}
