# Phase 4B.2F.1 — Adversarial Production Conformance

## Red baseline and repaired outcomes

The starting tree was clean at
`964853529521ea4d22614759afb49493854e20bf`.

| Probe | Pre-fix production result | Post-fix production result |
| --- | --- | --- |
| P30 / F01 | All nine dimensions Proven; overall Proven; checked capability issued. The checked source contract pin was `9361afd304467996e278ade3616df05eff32a998954e5b012a8a67a352f3bccf`, IR pin `33f10b596f76499ca9db09f0848f4f6e4c08c84f771784ac2ee2d5dc4433c3f5`, and top export pin `480cd3e97a7c0de42b1f7a94cf12da97beaaed2b6ddf5b7d43b90fe4c892543d`. | ε Unproven; overall Unproven; no checked capability. The exact source row and both export rows are validated and compared. |
| P31 / F02 | All nine dimensions Proven; overall Proven; checked capability issued; body RequiresEvidence. Root/Snapshot A export pin `2019eeb505c3d1e14fa086a89bd3868a6f1692224fbad44f3651fe493943afc5`; Snapshot B pin `1851278a5faa9a8b239abe89d4191d3f3735b83004124894b791304ba07fc661`; top export C equals A. | `proof-context` preflight Unproven; all nine dimensions Unproven; overall Unproven; no checked capability. Body remains RequiresEvidence. Root's stored target pin equals A; the assertion/current snapshot target pin equals B. |

P30's actual input contains a typed Int64 port 1 and the discriminating
valuation `p1=1`. The repaired checker does not claim that its current
multi-row fallback constructs a global countermodel; it fails closed as
Unproven. This distinction is retained in the corpus.

## Independent corpus and runner

`tools/gen-root-export-soundness-corpus.py` generates 47 independent cases:

- 37 effect containment production inputs, including P30, single-row complete
  witnesses, split-row negatives, atom/selector/qualifier mutations, typed
  guard cases, and one-use row-capacity cases;
- 10 resolver snapshot production inputs, including same-snapshot positive,
  changed export association, fresh derivation under B, changed component-pin
  set, missing export association, same resolver data under a distinct context
  object, and top-export variations.

The Rust runner executes every case through checked ComponentContract
validation, checked Semantic IR, Phase 3C, resolution, root derivation,
RootRecovery, and RootExport analysis/check. It compares stage, all nine
dimension verdicts, overall verdict, capability issuance, body/runtime
boundary, exact checked effect rows, per-row atom/selector/bounds/guard
analysis, and Root(A)/Assertion(B)/Export(C) digest associations. Result:

```text
manifest=47  stage_executed=47  fully_compared=47
mismatches=0  skipped=0  checked_capabilities=29
```

The independent Python oracle does not call Rust or read Rust Actual. The old
RootExport corpus remains at 35/35 and is unchanged. Existing suites remain
separately accounted for in the final gate report.
