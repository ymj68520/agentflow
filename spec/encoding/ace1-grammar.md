# ACE1 Exact Byte Grammar — v1

> Status: AgentFlow ACE1 v1 repository normative source (ratified in Phase 1A;
> derived from the frozen v1 baseline, `docs/` §24.1, §22.3, §22.13, which wins
> over this document wherever they conflict).
>
> Encoding-changing modifications to this file require an explicit
> specification compatibility/version decision and a frozen-corpus update
> (`corpus_revision`); silent edits that change golden bytes are forbidden.
> The frozen decisions and their provenance are recorded in the Decision
> Register, `docs/implementation/phase-01-open-questions.md`.
>
> Out of scope here: the AFIDL source grammar — that is a separate normative
> artifact (`spec/idl/afidl1-grammar.md`, materialized in Phase 2A). This file
> defines only the canonical byte grammar of encoded values.
>
> This is NOT a copy of `docs/`; it is the exact grammar extracted and made
> precise from it.

## 1. Positioning

<a id="ace1-schema-directed"></a>

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

<a id="ace1-utf8-canonical-bytes"></a>

```text
canonical-bytes := canonical-value        ; UTF-8, no BOM, no trailing bytes
```

An encoder never emits whitespace; a canonical decoder rejects any whitespace
byte outside string content, any trailing data, and any leading data.

## 3. Primitive encodings

<a id="ace1-numeric-as-string"></a>
<a id="ace1-timestamp-duration-micros"></a>

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

<a id="ace1-float64-exact"></a>
<a id="ace1-float64-nonfinite"></a>
<a id="ace1-float64-negative-zero"></a>

IEEE-754 binary64. The canonical text is the 64-bit pattern serialized
big-endian as exactly 16 lowercase hex characters inside a JSON string.

- NaN, `+Inf`, `-Inf`: rejected (encoder and decoder).
- `-0` (`0x8000000000000000`) canonicalizes to `+0`
  (`"0000000000000000"`); the decoder rejects the `-0` bit pattern as
  non-canonical.
- Examples: `1.0` → `"3ff0000000000000"`, `-1.0` → `"bff0000000000000"`.

## 6. Strings

<a id="ace1-string-nfc"></a>

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

<a id="ace1-bytes-base64url"></a>

Unpadded base64url (RFC 4648 §5): alphabet `A-Z a-z 0-9 - _`; `+`, `/` and
`=` never appear. Empty bytes encode as the empty string `""` — exactly one
representation. Decoding is strict: padding, foreign alphabet characters,
impossible lengths and non-canonical trailing bits are rejected.

## 8. Containers

<a id="ace1-map-key-nfc"></a>
<a id="ace1-map-key-ordering"></a>
<a id="ace1-duplicate-key-rejection"></a>
<a id="ace1-closed-enum-variant"></a>

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
- **Option<T>** (ratified Phase 1A): an explicit single-case tagged object.
  `None` encodes as `{"none":null}`; `Some(v)` encodes as `{"some":v}` with
  the payload validating against `T`. Exactly one member is allowed; the key
  MUST be `none` or `some`; the `none` payload MUST be the literal `null`.
  Because the case tag is explicit, ANY legal AFIDL type may sit under an
  option — including `unit`, `null` and nested options
  (`option<option<T>>`, `option<option<null>>`, ...). Literal `null`, empty
  objects, unknown keys, extra members, and non-null `none` payloads are
  rejected (a bare-JSON-number payload under `none` is rejected by the
  generic canonical layer before the option layer).
- **Variant**: a single-key object `{"<case>":payload}` where `<case>` is one
  of the closed case set; unknown cases are rejected.
- **Enum**: the closed value's name as a canonical string; unknown values are
  rejected.

## 9. Missing / null / unit / Option distinctions

<a id="ace1-option-tagging"></a>

`missing`, explicit `null`, `unit`, `Option::None` and `Option::Some(v)` are
five distinct states resolved by the schema-directed typed encoder:

- a record field that is required but not supplied is an error
  (`missing_field`), never silently `null`;
- a field of shape `null` encodes the literal `null`;
- a field of shape `unit` encodes the literal `null` (unit and null are
  distinct nominal types; the schema, not the bytes, disambiguates the
  position — ratified, see Decision Register OQ-2);
- `Option::None` encodes the tagged object `{"none":null}` (ratified Phase 1A,
  replacing the Phase 1 null-sugar encoding — see OQ-3);
- `Option::Some(v)` encodes the tagged object `{"some":v}`; therefore
  `Some(null)` and `Some(unit)` both yield `{"some":null}` — the Option layer
  is explicit in bytes while the inner schema nominally disambiguates the
  payload;
- `Option::Some(Some(...))` nests as tagged objects
  (`{"some":{"none":null}}`, `{"some":{"some":"x"}}`, ...).

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
