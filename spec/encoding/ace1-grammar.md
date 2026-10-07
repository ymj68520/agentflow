# ACE1 Exact Byte Grammar — v1 Materialization

> Status: materialized from the frozen v1 baseline (`docs/` §24.1, §22.3, §22.13).
> This document is the machine-checkable byte grammar this repository's
> implementations agree on. Where the frozen baseline constrains behavior, the
> baseline wins; everything else here is the Phase 1 materialization decision
> recorded in `docs/implementation/phase-01-open-questions.md` (OQ-1..OQ-6) and
> subject to ratification when the normative freeze lands.
>
> This is NOT a copy of `docs/`; it is the exact grammar extracted and made
> precise from it.

## 1. Positioning

ACE1 is AgentFlow v1's **deterministic, schema-directed, UTF-8 canonical
encoding** — a strict subset of JSON text. It is not `JSON.stringify`, not
"sorted-keys serde_json", and not a generic JSON canonicalizer:

- encoding is driven by an explicit machine type (`Shape`), never by
  reflection over arbitrary host objects;
- JSON numbers are **never** the canonical authority for machine numerics;
  every machine numeric primitive is encoded as canonical text inside a JSON
  string;
- there is no insignificant whitespace anywhere in canonical bytes.

## 2. Top-level productions

```text
canonical-bytes := canonical-value        ; UTF-8, no BOM, no trailing bytes
```

An encoder never emits whitespace; a canonical decoder rejects any whitespace
byte outside string content, any trailing data, and any leading data.

## 3. Primitive encodings

| Machine type | Canonical encoding |
|---|---|
| `unit` | the literal `null` |
| `null` | the literal `null` |
| `bool` | the literals `true` / `false` |
| `int32` | string: canonical decimal text, range [-2147483648, 2147483647] |
| `uint32` | string: canonical decimal text, range [0, 4294967295] |
| `int64` | string: canonical decimal text, range [-9223372036854775808, 9223372036854775807] |
| `uint64` | string: canonical decimal text, range [0, 18446744073709551615] |
| `decimal` | string: canonical decimal text (§4) |
| `float64` | string: exactly 16 lowercase hex chars (§5) |
| `string` | string: NFC-normalized UTF-8 with canonical escaping (§6) |
| `bytes` | string: unpadded base64url (§7) |
| `timestamp` | string: canonical int64 text; nominal int64 microseconds |
| `duration` | string: canonical int64 text; nominal int64 microseconds |

Notes:

- `timestamp` and `duration` are **nominal** types distinct from each other
  and from `int64`, despite sharing the same wire width and text form. A
  typed encoder rejects cross-assignment.
- Canonical integer text: optional single `-`, ASCII digits only, no `+`, no
  leading zeros (`0` alone is fine), `-0` rejected. Values never pass through
  IEEE-754.

## 4. Decimal canonicalization

Input grammar (accepted by the parser; ASCII only, no exponent notation):

```text
decimal    := "-"? int-part ( "." frac-part )?
int-part   := "0" | [1-9] [0-9]*
frac-part  := [0-9]+
```

Canonical form (stored and encoded):

1. strip trailing zeros from the fractional part (`1.50` → `1.5`);
2. drop the radix point when the fractional part becomes empty (`1.0` → `1`);
3. negative zero normalizes to `0` (`-0`, `-0.000` → `0`);
4. leading zeros are rejected, not normalized (`01.5` invalid);
5. precision is unbounded; Phase 1 defines identity/canonicalization only,
   not arithmetic.

## 5. Float64

IEEE-754 binary64. The canonical text is the 64-bit pattern serialized
big-endian as exactly 16 lowercase hex characters inside a JSON string.

- NaN, `+Inf`, `-Inf`: rejected (encoder and decoder).
- `-0` (`0x8000000000000000`) canonicalizes to `+0`
  (`"0000000000000000"`); the decoder rejects the `-0` bit pattern as
  non-canonical.
- Examples: `1.0` → `"3ff0000000000000"`, `-1.0` → `"bff0000000000000"`.

## 6. Strings

- Content is Unicode text; the canonical bytes contain NFC-normalized UTF-8.
  Canonically equivalent input sequences must produce identical canonical
  bytes. (A canonical decoder rejects non-NFC content.)
- Escaping table (deterministic, minimal):

```text
U+0022  →  \"         U+0008  →  \b
U+005C  →  \\         U+0009  →  \t
                      U+000A  →  \n
                      U+000C  →  \f
                      U+000D  →  \r
other U+0000..U+001F  →  \u00xx   (lowercase hex)
everything else       →  raw NFC UTF-8 (no U+2028/2029 escaping, no \u for non-controls)
```

- A canonical decoder accepts exactly these escapes: uppercase hex digits,
  `\/`, `\uXXXX` for non-control characters, short-form-able controls in
  `\u` form, and raw control characters are all rejected.

## 7. Bytes

Unpadded base64url (RFC 4648 §5): alphabet `A-Z a-z 0-9 - _`; `+`, `/` and
`=` never appear. Empty bytes encode as the empty string `""` — exactly one
representation. Decoding is strict: padding, foreign alphabet characters,
impossible lengths and non-canonical trailing bits are rejected.

## 8. Containers

```text
record  := "{" ( member "," )* member "}" | "{" "}"
member  := canonical-string ":" canonical-value
```

- **Record** fields are exactly the declared fields of the record shape —
  every declared field appears (there is no "absent field" state), no other
  key may appear, and members are emitted sorted by the unsigned UTF-8 byte
  sequence of the NFC field name. Duplicate fields are rejected.
- **Map<string,T>** has the same object form; keys are NFC-normalized first,
  must be unique *after* normalization (NFC-equivalent duplicate keys are
  rejected, never last-write-wins), and are emitted in unsigned UTF-8 byte
  lexicographic order. Input entry order never affects canonical bytes.
- **List<T>** is a JSON array in order: `[v1,v2,...]`; `[]` when empty.
- **Option<T>**: `None` encodes as the explicit literal `null`; `Some(v)`
  encodes as `v`. `option<option<T>>`, `option<unit>` and `option<null>` are
  rejected as unrepresentable.
- **Variant**: a single-key object `{"<case>":payload}` where `<case>` is one
  of the closed case set; unknown cases are rejected.
- **Enum**: the closed value's name as a canonical string; unknown values are
  rejected.

## 9. Missing / null / unit / Option distinctions

`missing`, explicit `null`, `unit`, `Option::None` and `Option::Some(v)` are
five distinct states resolved by the schema-directed typed encoder:

- a record field that is required but not supplied is an error
  (`missing_field`), never silently `null`;
- a field of shape `null` encodes the literal `null`;
- a field of shape `unit` encodes the literal `null` (unit and null are
  distinct nominal types; the schema, not the bytes, disambiguates the
  position — see OQ-2);
- `Option::None` encodes the explicit literal `null` (see OQ-3);
- `Option::Some(v)` encodes `v`.

## 10. Canonical decoder contract

```text
decode(canonical bytes) → typed value
re-encode               → exact same bytes
```

Anything valid-looking but non-canonical is rejected, never auto-repaired:
whitespace, unsorted/duplicate keys, non-NFC text, non-canonical escapes,
bare JSON numbers, padded base64url, `-0` float bits, non-canonical decimal
text, trailing bytes, unknown/missing record fields, unknown enum values or
variant cases.
