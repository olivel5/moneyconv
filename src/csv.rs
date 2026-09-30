// Just enough CSV for a two-column amounts file: quoted fields, doubled quotes
// inside them, and no embedded newlines (input is read a line at a time).
pub fn split_record(line: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' && cur.trim().is_empty() {
            cur.clear();
            in_quotes = true;
        } else if c == ',' {
            fields.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    if in_quotes {
        return Err("unterminated quote".to_string());
    }
    fields.push(cur.trim().to_string());
    Ok(fields)
}

// Finds the currency and amount columns by header name, so files exported with
// extra columns or a different column order still work.
pub fn find_columns(header: &[String]) -> Result<(usize, usize), String> {
    let find = |name: &str| header.iter().position(|h| h.eq_ignore_ascii_case(name));
    match (find("currency"), find("amount")) {
        (Some(c), Some(a)) => Ok((c, a)),
        _ => Err(format!(
            "header must contain \"currency\" and \"amount\" columns, got {:?}",
            header
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_simple() {
        assert_eq!(
            split_record("USD, 19.99 ,x").unwrap(),
            vec!["USD", "19.99", "x"]
        );
    }

    #[test]
    fn split_quoted() {
        assert_eq!(
            split_record("\"a,b\",\"say \"\"hi\"\"\"").unwrap(),
            vec!["a,b", "say \"hi\""]
        );
    }

    #[test]
    fn split_empty_fields() {
        assert_eq!(split_record("a,,").unwrap(), vec!["a", "", ""]);
    }

    #[test]
    fn split_unterminated_quote() {
        assert!(split_record("\"abc,def").is_err());
    }

    #[test]
    fn columns_any_order_and_case() {
        let h = split_record("id,Amount,CURRENCY").unwrap();
        assert_eq!(find_columns(&h), Ok((2, 1)));
    }

    #[test]
    fn columns_missing() {
        let h = split_record("code,value").unwrap();
        assert!(find_columns(&h).is_err());
    }
}
