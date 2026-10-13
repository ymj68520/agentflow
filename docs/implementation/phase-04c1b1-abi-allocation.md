# Phase 4C.1B.1 — Additive ABI Allocation

The start HEAD is efc2a8d90a65432916a8dbf559510e2936f0d146. Before registry
generation, [baseline.json](../../spec/conformance/portable-proof-codec/baseline.json)
saved the complete 477-row ABI table and all registered schema rows.

The materializer compiled the complete 17-module AFIDL set and identified 298
new numbered members across the portable named records, variants and enums.
Existing imported types retain their previous families and assignments.
Every new symbolic number is enumerated in the [schema materialization audit](phase-04c1b1-schema-materialization-audit.md).

After registration, the generated ABI contains 775 active rows. Each old row
is compared as the full tuple (family, type, kind, number, symbolic name,
status, first-seen digest) and must appear exactly once. A regression test
reads the pre-registration snapshot and checks all 477 tuples. The 298
portable bindings are additive; none reuses an existing slot.

The family is agentflow.portable.proof.inputs@1. The new normative sources
are the registered AFIDL module and portable proof semantics document. All
pre-existing schema files, assignments, digest domains and TypeSchemaDigest
goldens retain their identities. This schema adds no HashV1 domain.

Current registered counts are 43 specifications, 404 rules (403 covered, one
inherited intentional exception, zero missing), 775 active assignments, 17
AFIDL modules and 14 HashV1 domains. The final gate report records post-commit
counts and frozen-source comparisons.
