# R3-F1 — Bytes guard lowering repair

Scope: repair R3-NG-B01 only. Starting HEAD
`be04c05e07d3a72a7c795d289e689281bceaa65f`; repair `c020aef`; complete
production/conformance code checkpoint `325b1b465fc00b62b5c753788927f7c779bd0176`.
Final acceptance and HEAD are tied to the actual runtime gate ledger, not
inferred from a passing Cargo test or a prefilled generator result.

## Root cause and complete call chain

`PrimitiveConstant::Bytes(String)` stores **canonical unpadded base64url text**,
not raw bytes and not arbitrary Unicode text. Its existing `validate` calls
`af_core::base64url::decode`, which rejects padding, foreign alphabet, impossible
lengths and nonzero trailing bits by strict decode/reencode. For `"AQ"`, the
semantic payload is `[1]`, not the UTF-8 bytes `[65,81]` of the source carrier.
Empty text means empty bytes; it is not a decode-error fallback.

The registered `PrimitiveConstant` AFIDL case is `bytes: bytes`. The frozen
ACE1 typed encoder requires `Shape::Bytes` with `Value::Bytes(Vec<u8>)` and
reencodes those raw bytes to the canonical base64url scalar. It correctly rejects
`Value::String` at that nominal position. No encoder/grammar/model change is needed.

The actual call chain is:

`canonicalize_contract -> CanonicalComponentContract::ace1_bytes -> contract_value
 -> EffectSignature::to_value -> row_value -> entry_value -> guard_value
 -> constant_value -> af_core::encode`.

The old branch simply cloned the source String carrier into `Value::String`;
that implemented the wrong nominal payload. This describes the implementation,
not an assertion about its author's intent. Both original failing inputs pass
through the **same** branch. They pass canonical structural validation, then
fail byte materialization with `AF-CONTRACT-0003 / ace1_encode / expected bytes,
got string`. Before applying the repair, a fresh53-input run reproduced51 matches
and these exact two failures, with default semantic gate exit1.

## Minimal production change

Only `crates/af-effect/src/canonical.rs` changes production behavior. Bytes now
strictly decodes to `Value::Bytes`; decode failures are propagated as errors,
not panics, empty bytes or String fallback. Four private lowering helpers carry
`Result` so this fallibility reaches the existing public `to_value` boundary.
The public signature is unchanged. No other helper body outside these five
reviewed functions changes. String/Decimal/Timestamp/Duration and the other
constant branches retain their exact existing mappings.

Guard containment, native branch selection, verdict/basis/reachability, native
source binding/collector, ComponentContract/SemanticIR meanings and HashV1 are
unchanged. No Standalone, Root, Control/Leaf or other T3 implementation resumes.

## Evidence and typed distinctions

`original-two.json` retains each original CaseId, full original declaration,
frozen Expected, original Actual, original encoding stage and first differing
field. The original source authoring carrier is a complete typed JSON object;
Expected canonical-input hex is explicitly Expected, not fabricated successful
pre-fix output. The previous historical transport stream was not retained and
is not claimed reconstructed. Raw frozen source/Expected hashes bind the input
objects used in the fresh pre-fix reproduction.

`resolved-two.json` retains complete post-fix Actual and actual pipeline stages.
The exact same two sources now canonicalize and encode; their actual Native
result remains `Violation / TypedCountermodel`. Full Contract Check still
rejects them. Nothing was changed to obtain a Checked token.

The unchanged original Python Oracle and unchanged frozen Expected both match
all53 new Actual inputs. The original51 Actual objects also remain exactly
equal, including their diagnostic reasons, to the immutable pre-fix capture.
Three genuine early canonical rejections remain early rejections, not invented
Native results. Empty binding sets still do not imply complete proof.

String and Bytes have different source cases, typed payloads and full Guard
canonical encodings (`{"string":"AQ"}` versus `{"bytes":"AQ"}`). A bare
scalar can be the same text under two **different** nominal shapes; ACE1 §3
explicitly uses schema/variant identity to disambiguate. That is not an
unexpressible canonical identity or a reason to invent another grammar.

## Targeted coverage, not full T3 closure

Five new low-level Rust test groups cover18 binary inputs (including empty,
one/multi-byte, zero, non-UTF8, UTF8-as-Bytes, URL alphabet, base64 block edges,
and decoded lengths786431/786432/786433 around the unchanged1MiB scalar-text
threshold),12 malformed source representations,8 non-Bytes constant mappings,
closed/open-known paths, deterministic encode and typed decode/reencode.
Core scalar encoding has no Portable D0 policy; a legal Core value above that
text threshold does not assert admission through Portable's unchanged default.

Independent Python supplementary checks cover26 lowering and36 actual Native
pipeline input IDs. They compute exact raw bytes using Python base64, not the
Rust helper. Full typed payloads and canonical outputs are compared, not only
digests. Positive contained Bytes cases obtain genuine Checked contracts;
countermodel/type-mismatch/invalid cases preserve genuine rejection.

An initial new fixture incorrectly inferred Checked from a zero-potential Native
P. The actual existing AF-CONTRACT-0016 attempt-floor check is independent of
guard coverage. The F1 fixture premise now explicitly calculates both conditions:
possible activation requires an attempt, and native compatibility must hold.
Zero-attempt cases retain Native P **and** whole-contract rejection. This is a
reviewed fixture acceptance premise, not a new independent whole-D6 Oracle and
not a modification of the frozen original53-case Oracle or Expected.

Forty F1-only comparator mutations include Bytes/String substitution with equal
scalar text, changed raw content and assignments, and both-sided deletion.
The14 countermodel fields and3 assignment fields come from exact frozen
selected-owner model descriptors. Imported presence/variant checks run before
value comparison. These40 are not the unfinished whole-R3 comparator target.

The Cargo total is determined from the fresh log, not from input-ID
counts; see the runtime results. Twenty-one reused fixture Rust tests are not
relabeled as new capture cases. The181 old lexical anchors are not a closed
unique semantic producer count. Full R3-NG seed conformance remains pending.

## Freeze and authority

F1's immutable initial manifest covers all1987 starting paths. Exactly two
existing paths are authorized exceptions: the one production lowering file
and its directly dependent native diagnostic test harness. The remaining1985
paths—including every old specification, registry, Expected, original Oracle,
historical evidence, D9/D10 data, limits, native producer/collector code—remain
raw-byte equal. The old R3 byte lock on the intentionally repaired production
file is historical evidence; it is not silently edited or applied as F1 policy.
`audit-bytes-guard-repair.py` verifies the explicit narrow exception and all
other frozen bytes, plus97 selected imports/90 live declarations/19 snapshots.

All54 specifications/468 rules/467 covered/1 exception/0 missing,775 ABI,
17 AFIDL and14 hash domains remain unchanged. Exact historical builds use their
own original source/Registry/Expected. Current889 registration IDs remain
registration—not typed capture conformance. Direct/native/private/alternate
RootExport and default/explicit-bounded Portable D4 denial are rerun unchanged.

On complete successful runtime gates only, R3-NG-B01 becomes
**CLOSED — PRODUCTION LOWERING FIX VERIFIED**. This closes no other G01/G02/G03,
F03/F04 or B01–B07 issue. RootExport remains NotAuthorized; T4 remains ineligible.
F1 success permits requesting R3 resumption, not automatically performing it.
