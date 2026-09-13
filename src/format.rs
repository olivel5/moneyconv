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
