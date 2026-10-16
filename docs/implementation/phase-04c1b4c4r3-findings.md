# R3-NG-B01 — legal Bytes guard cannot be encoded

Outcome **B**, severity **P1**, OPEN/BLOCKING. Not a false-Proven finding.
The defect is present at starting HEAD `a0ef89e71ac31405ba05df619f585b5830aa8c92`;
`crates/af-effect/src/canonical.rs` is byte-unchanged from the initial tree.
This is a fresh current build using that exact unchanged source, not a claim
that the entire a0 revision was separately rebuilt for this new vector.

Normative authority: `spec/idl/semantic-core.afidl` declares
`PrimitiveConstant.bytes: bytes`; `spec/idl/effect-signature.afidl` imports
that exact constant for InputEquals; AF-ACE1-0011 / ACE1 §7 admits canonical
unpadded base64url including empty bytes. AF-CONTRACT-0022 permits matching
primitive guard domains; AF-SEMANTIC-0245 requires genuine canonical bytes.

Minimal effect-level input and complete native inputs are preserved in
`phase-04c1b4c4r3-findings.json` and the independent Expected/Actual corpus.
Payloads `""` and `"AQ"` both reproduce. One effect atom with a bytes guard
suffices; no K action, T3 collector or Root evaluator is necessary to trigger
the effect encoder defect.

At `af-effect/src/canonical.rs:544`, `constant_value` constructs:

```rust
PrimitiveConstant::Bytes(value) => ("bytes", Value::String(value.clone())),
```

The typed ACE1 encoder accepts `(Shape::Bytes, Value::Bytes(_))`, not String.
Structural validation succeeds and direct typed core encoding succeeds, but
the effect encoder rejects with `expected bytes, got string`. The full native
declaration also passes canonical structural validation; its later canonical
byte materialization fails with `ContractError{rule_id:AF-CONTRACT-0003,
code:ace1_encode,reference:None}`. This is not a missing schema or invalid
base64url input, nor an actually executed Native Guard failure branch.

Independent Expected keeps legal bytes accepted. Actual remains rejected.
The default Oracle exits1. The two diagnostic regressions passing in Cargo
mean the contradiction was detected, not that conformance became successful.

No repair is made: changing lowering would change accepted canonical inputs,
which is beyond the authorized semantics-preserving T3 adaptation. Per §39
Outcome B, R3-NG and downstream R3-ALL stop. A separate authorization must
decide the production repair and its complete regression requirements. No
normative/registry/Expected change is proposed merely to fit Actual.
