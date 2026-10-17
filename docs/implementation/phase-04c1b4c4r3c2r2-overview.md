# C2R2 — registered root subset, not full-field closure

Outcome D. No complete Raw LocalTypedFactSeed, full-field independent Oracle,
canonical FactKey, CompleteView, Scope encoder or RootExport capability is claimed.
Start: `cee9b10e965dfd0bd16ed73ded3e72eca16cb7d8`.

The authoritative machine inventory is
[field-and-stage-audit.json](phase-04c1b4c4r3c2r2-field-and-stage-audit.json).
It contains the exact registered descriptors, per-event initial and final root
slots, a finite descriptor reference graph, direct nested obligations, independently
derived subset Expected, comparison and mutation records. A descriptor graph is
not represented as a runtime nested-value closure.

## Exact registered Fact roster

All root slots belong to `agentflow.computation.proof-report.model@5/Fact`.
The JSON preserves the actual descriptor expressions, nominal identity, source
descriptor path, requiredness, dependencies, source code and comparator status.

| Field | Exact registered type | C2R2 comparison |
| --- | --- | --- |
| key | FactKey | Incomplete |
| role | Role | Registered subset |
| evaluation | Evaluation | Registered subset |
| status | Status | Registered subset |
| prerequisites | Set&lt;PrerequisiteKey&gt;, minimum 0 | Registered local empty relation |
| references | Set&lt;EvidenceRef&gt;, minimum 0 | Incomplete |
| primary | Opt&lt;EvidenceRef&gt; | Incomplete |
| contradiction | Opt&lt;TypedContradiction&gt; | Incomplete; subject-bound witness input only |
| cause_fact_keys | Set&lt;FactKey&gt;, minimum 0 | Registered local empty relation |
| companion_evidence_status | Opt&lt;ProofVerdict&gt; | Registered None |
| observations | Set&lt;ObservationSite&gt;, minimum 1 | Incomplete |
| substitution_basis | Opt&lt;SubstitutionBasis&gt; | Registered None |
| native_guard_basis | Opt&lt;NativeGuardBasis&gt; | Registered None |
| raw_validation_basis | Opt&lt;RawPortCheckBasis&gt; | Borrowed local mapping; incomplete |

An option field is required even when its value is None. An empty local relation
is not a fabricated established prerequisite. No schema-resolution premise is
queried before the port test (raw-validation RC-LINKS / AF-SEMANTIC-0267).

## Initial and remaining gaps

The unchanged 59 primary Capture CaseIds yield 125 reached events. R2-A enumerates
1,750 root instances: 1,250 initially unmaterialized and 500 locally mapped, not
fully conformant. All initial instances remain in the ledger; none is removed.

The new eight-field comparison covers 1,000 registered root-subset instances:
625 initially missing slots and 375 previously partial slots. Remaining:
625 unmaterialized slots and 125 partial basis slots, hence **750 incomplete root
instances**. This is not a reduction of the formal 14-field requirement.

The initial nested baseline lists 7,125 direct slot obligations separately from
the full descriptor reference graph. Collection lengths, variants and complete
compiler/context/reference value closures not yet evaluated are explicitly
undetermined, not zero. Fully required nested instance count remains unknown.

## Checkpoint disposition

R2-A completed. R2-B has partial measured progress, not closure. R2-C–R2-H are
not accepted. Subset mutations and input sensitivity are development validation
for R2-B; they do not claim entry or acceptance of the full later checkpoints.
Engineering/history/generator regressions do not repair that semantic gap.

The runtime receipt in `target/phase-04c1b4c4r3c2r2-final-runtime-results.json`
records actual exits, source HEADs, final HEAD and final-tree byte maps. No prior
phase's execution is counted as a fresh current-phase gate.
