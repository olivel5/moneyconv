# moneyconv

Systems that deal with money tend to argue with each other about how to
represent an amount. Invoices, reports, and humans want `19.99`. Ledgers,
payment APIs, and anything doing arithmetic on money want `1999` (an integer
count of minor units) so nothing gets silently corrupted by floating point.
`moneyconv` translates between the two, line by line.

## Formats

**plain** - `CODE AMOUNT`, decimal, human-readable:

```
USD 19.99
JPY 500
KWD 12.500
```

**ledger** - `CODE MINOR_UNITS`, integer minor units:

```
USD 1999
JPY 500
KWD 12500
```

The number of decimal places per currency follows ISO 4217: most currencies
use 2 (cents), some use 0 (JPY, KRW, VND, ...), and a few use 3 (KWD, BHD,
OMR, ...). `moneyconv` knows this table and refuses to parse a plain amount
with more decimal places than the currency allows, rather than rounding it
away.

## Usage

Convert a file of human-readable amounts into ledger format:

```
moneyconv --from plain --to ledger --input invoices.txt --output invoices.ledger
```

Pipe amounts through stdin/stdout (no `--input`/`--output` needed):

```
echo "EUR 42.50" | moneyconv --from plain --to ledger
EUR 4250
```

Mix stdin with a file for output, or read a file and print to stdout - `-`
also means stdin/stdout explicitly if you want to be unambiguous in a
script:

```
cat ledger.txt | moneyconv --from ledger --to plain --output report.txt
moneyconv --from ledger --to plain --input - --output -
```

Malformed lines are reported to stderr with their line number and skipped;
the rest of the input is still processed. The process exits non-zero if any
line failed.

### CSV

With `--csv`, the first non-empty line is a header that must name a
`currency` column and an `amount` column (any order, any case; other columns
are dropped). Output is always `currency,amount`:

```
$ printf 'id,currency,amount\n1,usd,19.99\n2,JPY,500\n' | moneyconv --csv --from plain --to ledger
currency,amount
USD,1999
JPY,500
```

Quoted fields are understood, but fields with embedded newlines are not. A
bad header stops the run; bad rows are skipped like bad lines in the plain
formats.

## Building

Standard library only, no dependencies:

```
cargo build --release
```

## Status

Early skeleton. Two formats and one conversion path so far - see the roadmap
for what's planned next.
