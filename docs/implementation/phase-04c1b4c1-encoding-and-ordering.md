# Canonical encoding and total order

**DRAFT — NOT REGISTERED NORMATIVE.** See [draft D11–D13](phase-04c1b4c1-g01-normative-contract-draft.md) for the complete contract, and [ACE1](../../spec/encoding/ace1-grammar.md) for the reused byte grammar. No codec, AFIDL type or digest domain is added in this phase.

The semantic identity's closed report shape is encoded with schema-directed ACE1. It is not generic sorted JSON, Rust Debug or a delimiter-concatenated message. The representation is a string containing canonical ACE1 text; if placed inside a JSON report it is escaped *again by the outer string encoder*, not interpreted as an object or split on a colon.

| Item | Proposed exact convention |
| --- | --- |
| Version | literal draft edition `g01.obligation/1-draft`; unsupported edition fails, no latest fallback |
| Keys | all declared fields, unsigned UTF-8 field-name order; duplicate/unknown/missing key rejected |
| Text | NFC UTF-8; existing ACE1 minimal escapes; malformed/noncanonical input rejected |
| Integers | nominal u32 IDs / u64 ordinals and work counters, canonical decimal inside strings; no IEEE-754 conversion |
| Digest | complete existing algorithm/domain +64 lowercase hex, no implicit domain or truncated digest |
| Schema | specification_id/type_name/complete schema_digest, no type-name-only equivalence |
| Bytes | unpadded canonical base64url of every required byte, not length or checksum |
| Untrusted claims | exact original UTF-8 bytes under raw-claim descriptors; never NFC-repair invalid assertions or sort failing mapping lists |
| Option | explicit `none:null`/`some:payload`; absence is an error, not None |
| Sum / enum | closed declared tag/rank and complete payload; unknown future token fails |
| Records / recursive scope | complete transparent typed snapshots and minimal reachable canonical value-DAG |
| Lists / sets | semantic sequences preserved; formal sets emitted in typed total order |

The declared comparison tuple is scope, nested assertion path, stage, role, dimension, kind, source variant/payload, source ordinal, mapping ordinal, target, coordinate, rule, requirement, origin. Role is obtained from the fact roster and must agree for a shared identity; it is not caller-editable. Field component comparison is typed: mathematical integers, unsigned NFC UTF-8 text, full bytes, None<Some, declared variant rank then payload, tuples/lists lexicographic. The final canonical sequence may differ from lexicographic sorting of its encoded strings. In particular numeric2 precedes10 even though `"10"` sorts before `"2"` as text.

Canonical scope DAG construction: validate finite acyclic graph, intern equal typed node values bottom-up, rank leaves by full typed key, then increasing height/nonrecursive fields/ordered child ranks. Remove unreachable nodes; the canonical decoder rejects redundant/nonminimal graphs and invalid ranks. This preserves sharing-insensitive value equality without exponential expansion. Child order and nested call-site paths never become sets. No new Merkle hash or ObjectId is introduced.

Injectivity argument: ACE1's closed shape makes record members, nominal scalar types and sum/option tags uniquely decodable. Full scope and all identity coordinates are present. Decoding equality implies equality of every identity component; conversely equal typed components produce equal bytes. Presentation-only edits have no identity bytes. A shorthand such as `O(node=2)` in design vectors is not itself canonical output.

Proposed local bounds are1024 assertion nodes/depth96/65536 identities/4194304 work steps/128MiB view bytes. They apply before expansion/output allocation and include repeated inline scope bytes. A stricter implementation declares its limits and rejects with explicit ResourceLimit, never a partial/empty view or a proof verdict. Encoding failures cannot mint a Checked capability. These are future report-local limits, not changes to present Portable limits.
