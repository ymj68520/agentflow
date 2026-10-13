# Phase 4C.1B.1 — ACE1 Encoder and Canonical Bytes

Portable values use `af-core`'s frozen schema-directed ACE1 codec. The new
crate derives its closed Bundle Shape from the compiled, registered AFIDL
module set; it accepts no `Any` or generic JSON escape. The codec adds no
change to existing ACE1 rules.

The encoder emits every required field, tagged Options, closed enum/variant
names, exact int32/uint32/int64/uint64 decimal strings, exact Decimal,
minimal NFC UTF-8 escaping and canonical unpadded base64url. Author-side
identities must already be NFC. Existing HashV1 is used only for the current
TypeSchema, SemanticIrDocument and ComponentContract domains.

All 61 registered collection paths are traversed with their typed ordering
keys. Canonical sets use the typed key and complete canonical row bytes as a
tie-breaker. Identical duplicate rows fail D1; distinct rows colliding on a
declared unique identity fail D2. Numeric keys compare numerically and text
keys compare unsigned UTF-8. Sequence fields retain their supplied order.

The independent Python oracle supplies frozen byte hashes for the 60 unique
inputs and a complete positive Bundle byte fixture. Rust compares the exact
length and SHA-256 of each materialized input and the accepted Bundle output.
Neither implementation passes an Int64/Decimal through binary floating point.
