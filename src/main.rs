mod format;

use format::{exponent_for, format_minor, parse_decimal};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::ExitCode;

#[derive(Clone, Copy)]
enum Format {
    Plain,
    Ledger,
}

fn parse_format(s: &str) -> Result<Format, String> {
    match s {
        "plain" => Ok(Format::Plain),
        "ledger" => Ok(Format::Ledger),
        other => Err(format!(
            "unknown format {:?} (expected \"plain\" or \"ledger\")",
            other
        )),
    }
}

fn print_usage() {
    eprintln!("moneyconv - convert currency amounts between decimal and minor-unit formats");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    moneyconv --from <plain|ledger> --to <plain|ledger> [--input FILE] [--output FILE]");
    eprintln!();
    eprintln!("FORMATS:");
    eprintln!("    plain   \"CODE AMOUNT\", e.g. \"USD 19.99\"");
    eprintln!("    ledger  \"CODE MINOR_UNITS\", e.g. \"USD 1999\"");
    eprintln!();
    eprintln!("With no --input (or --input -), reads from stdin.");
    eprintln!("With no --output (or --output -), writes to stdout.");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut from: Option<Format> = None;
    let mut to: Option<Format> = None;
    let mut input_path: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--from" => {
                i += 1;
                let val = match args.get(i) {
                    Some(v) => v,
                    None => {
                        eprintln!("--from requires a value");
                        return ExitCode::FAILURE;
                    }
                };
                from = match parse_format(val) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        eprintln!("{}", e);
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--to" => {
                i += 1;
                let val = match args.get(i) {
                    Some(v) => v,
                    None => {
                        eprintln!("--to requires a value");
                        return ExitCode::FAILURE;
                    }
                };
                to = match parse_format(val) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        eprintln!("{}", e);
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--input" => {
                i += 1;
                match args.get(i) {
                    Some(v) => input_path = Some(v.clone()),
                    None => {
                        eprintln!("--input requires a value");
                        return ExitCode::FAILURE;
                    }
                }
            }
            "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => output_path = Some(v.clone()),
                    None => {
                        eprintln!("--output requires a value");
                        return ExitCode::FAILURE;
                    }
                }
            }
            other => {
                eprintln!("unrecognized argument: {}", other);
                print_usage();
                return ExitCode::FAILURE;
            }
        }
        i += 1;
    }

    let (from, to) = match (from, to) {
        (Some(f), Some(t)) => (f, t),
        _ => {
            eprintln!("both --from and --to are required");
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let reader: Box<dyn BufRead> = match input_path.as_deref() {
        Some("-") | None => Box::new(BufReader::new(io::stdin())),
        Some(path) => match File::open(path) {
            Ok(f) => Box::new(BufReader::new(f)),
            Err(e) => {
                eprintln!("cannot open {}: {}", path, e);
                return ExitCode::FAILURE;
            }
        },
    };

    let mut writer: Box<dyn Write> = match output_path.as_deref() {
        Some("-") | None => Box::new(BufWriter::new(io::stdout())),
        Some(path) => match File::create(path) {
            Ok(f) => Box::new(BufWriter::new(f)),
            Err(e) => {
                eprintln!("cannot create {}: {}", path, e);
                return ExitCode::FAILURE;
            }
        },
    };

    let mut had_error = false;

    for (line_no, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("line {}: read error: {}", line_no + 1, e);
                had_error = true;
                continue;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match convert_line(trimmed, from, to) {
            Ok(out) => {
                if let Err(e) = writeln!(writer, "{}", out) {
                    eprintln!("write error: {}", e);
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                eprintln!("line {}: {}", line_no + 1, e);
                had_error = true;
            }
        }
    }

    if let Err(e) = writer.flush() {
        eprintln!("write error: {}", e);
        return ExitCode::FAILURE;
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn convert_line(line: &str, from: Format, to: Format) -> Result<String, String> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let code = parts.next().unwrap_or("").trim();
    let amount = parts.next().unwrap_or("").trim();

    if code.is_empty() || amount.is_empty() {
        return Err(format!("expected \"CODE AMOUNT\", got {:?}", line));
    }
    if !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!("invalid currency code: {:?}", code));
    }

    let code_upper = code.to_ascii_uppercase();
    let exponent = exponent_for(&code_upper);

    let minor = match from {
        Format::Plain => parse_decimal(amount, exponent)?,
        Format::Ledger => amount
            .parse::<i64>()
            .map_err(|_| format!("not an integer: {:?}", amount))?,
    };

    let formatted = match to {
        Format::Plain => format_minor(minor, exponent),
        Format::Ledger => minor.to_string(),
    };

    Ok(format!("{} {}", code_upper, formatted))
}
