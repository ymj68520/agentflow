# Phase 4C.1B.1 — Production Closed Decoder

`af_portable::PortableProofCodec` exposes the stage boundaries directly. Every
stage wrapper has private construction and read-only access.

| Stage | Production entry | Result | Representative failures |
| --- | --- | --- | --- |
| D0 | `decode_d0(bytes)` | closed raw Bundle | UnknownField, MissingField, DuplicateField, UnknownVariant, InvalidOption, NumericRange, InvalidUtf8, ResourceLimit |
| D1 | `validate_d1(decoded,bytes)` | canonical bytes and ordered collections | NoncanonicalEncoding, NoncanonicalString, NoncanonicalInteger, DuplicateSetMember, NoncanonicalOrder |
| D2 | `validate_references(canonical)` | reference-closed raw Bundle | MissingAssertion/Context/Resolver, WrongDigestDomain, DuplicateIdentity, MissingReferencedContent, ReferenceCycle, UnreferencedAsset, ResourceLimit |
| D3 | `validate_contents(referenced)` | inline content-validated raw Bundle | InvalidTypeSchema, ContentIdentityMismatch, SourceContentMismatch, RegistryContentMismatch |
| D4 | `validate_semantics(content)` | `SupportedPortableStaticInputs` | UnsupportedAttributionProfile, MissingSemanticRole, ConflictingSemanticContent, UnsupportedSemantics |

D0 performs an explicit duplicate-member scan before calling the existing
ACE1 decoder. It enforces the registered closed Shape and scalar widths. D1
re-encodes all received bytes, compares the exact input and checks every
registered set key. Received sets are never sorted or repaired.

D2 resolves local table IDs, root pins, scoped resolver references and asset
reachability. StructuralType and assertion-target graphs are checked before
expansion. D3 verifies every inline Schema/IR/contract hash and every plain
checksum/source/registry relationship. D4 compares the complete compiler
content with the implementation-owned current profile.

Input bytes, scalar lengths, container sizes, nesting, total typed nodes and
combined assertion/structural graph work have finite limits. An exceeded limit
yields ResourceLimit and no later-stage capability. D5 and beyond are not
invoked.
