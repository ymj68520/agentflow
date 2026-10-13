# Phase 4C.1B.1 — Malformed and Boundary Input Matrix

The machine-readable fixture manifest is
[portable-proof-codec/manifest.json](../../spec/conformance/portable-proof-codec/manifest.json).
Its 60 cases are generated from one independent full Bundle and exact typed or
raw-byte mutations. Every case has Expected input bytes, stage, error family,
identity/projection field set and trust boundary. The production runner builds
each input, verifies its SHA-256 against the independent oracle, then executes
the public stage APIs.

| D0 class | Frozen matrix examples |
| --- | --- |
| closed records/variants | missing/unknown field, duplicate object key, unknown guard/binding case, unknown enum |
| options and numbers | bare null Option, uint32 overflow, wrong primitive type, invalid UTF-8 |
| bytes | padded/invalid base64url, malformed outer bytes content |
| D1 canonical bytes | leading/trailing whitespace, nonminimal escape, non-NFC text, noncanonical integer |
| D1 collections | reversed typed schema set, duplicate identical asset member |
| D2 references | missing assertion/context/resolver/compiler row, missing inline schema asset, wrong digest domain |
| D2 graphs | cyclic StructuralType and assertion-target identity; bounded graph-work exhaustion |
| D3 integrity | corrupted pinned IR/schema bytes, wrong normative-source/checksum/table/relation content |
| D4 support | wrong semantic label, Profile B/C, missing role, model-missing role claimed defined, changed registry bytes |
| D4 positives | complete current Profile A, None/Some(empty), exact uint64 above 2^53 |

Errors remain stage-specific. Content integrity can pass without establishing
origin trust or semantic checking of IR/contract bodies.
