# Specification Registry — Normative Mechanics, v1

> Status: AgentFlow registry mechanics v1 repository normative source
> (materialized in Phase 2B). This document defines the stable symbol system
> of the AgentFlow specification itself: the Specification Registry, the
> Normative Rule Registry (RuleId), the Stable ABI Number Registry, the closed
> rule-area table, and the ConformanceCaseId mechanism.
>
> Registry-shape-changing edits require an explicit specification decision and
> a corpus update (`spec/conformance/spec-registry/`); silent edits are
> forbidden.

## 1. Purpose and authority

From Phase 2B onward, every long-term ABI symbol of AgentFlow — schema,
protocol, opcode, event, command, profile, rule, field number, variant
number, enum value number — is registered here, not named ad hoc in code or
Markdown. The registry answers: what is this object called, which
`SpecificationId` owns it, what is its stable rule identity, is this number
taken, do referenced objects exist, and which rules have test coverage.

It does **not** answer execution questions (workflow, journal events, runtime
commands) and it does not implement a compatibility engine: it manages stable
number allocation and non-reuse only.

The authority for registry *shapes* is the AFIDL module
`spec/idl/specification-registry.afidl` (`agentflow.spec.registry@1`); the
JSON files under `spec/registry/` are instances validated against the rules
below. The JSON instances never define the registry schema itself.

## 2. Registry artifacts

| Artifact | Path |
|---|---|
| Specification Registry | `spec/registry/specifications.json` |
| Normative Rule Registry | `spec/registry/rules.json` |
| Stable ABI Number Registry | `spec/registry/abi-numbers.json` |
| Rule Area closed table | `spec/tables/rule-areas.json` |
| Conformance suite table | `spec/conformance/suites.json` |
| Registry AFIDL schema | `spec/idl/specification-registry.afidl` |
| Registry conformance corpus | `spec/conformance/spec-registry/` |

## 3. Specification Registry

Each entry is a `SpecificationRegistryEntry` with `specification_id`, `kind`,
`status`, `normative_source` (`SpecificationArtifactRef`), and optional
`schema_digest` and `description`. Only specifications that are actually
materialized are registered; unimplemented future modules must never be
pre-registered, and never as `frozen`.

`kind` is the closed enum `SpecificationKind`:

- `encoding` — a deterministic encoding specification (ACE1);
- `digest` — a digest/hash identity specification (HashV1);
- `digest_domain_registry` — the closed digest-domain table;
- `schema_language` — a schema definition language specification (AFIDL1,
  TypeSchema identity);
- `schema` — a concrete AFIDL schema module with frozen TypeSchemaDigests;
- `closed_table` — a closed normative table (primitive universe, rule areas);
- `registry` — a registry mechanics specification.

`status` is the closed enum `SpecificationStatus` (`draft`, `frozen`,
`deprecated`, `withdrawn`). `frozen` means semantic edits require an explicit
spec change; it does not mean the file cannot physically change — adding rule
anchors or clarifying prose is normative maintenance, changing normative
meaning is not.

<a id="spec-specification-id-unique"></a>

A `SpecificationId` is unique across the Specification Registry: two entries
with the same canonical `specification_id` are a registry error, even if all
other fields differ. SpecificationId syntax and semantics are owned by AFIDL1
(`agentflow.idl.afidl@1`); this registry reuses that implementation and never
introduces a parallel id format.

<a id="spec-normative-source-integrity"></a>

Every entry's `normative_source` is a `SpecificationArtifactRef`:
`logical_path` (repo-relative, `/`-separated, no `..`, no backslash, must
exist) plus `content_digest` — an `ArtifactFileDigest`, defined as the plain
`SHA256` of the referenced file's bytes, recorded as lowercase 64-hex with
`algorithm: "sha256"`. The validator recomputes the digest and rejects any
mismatch. An `ArtifactFileDigest` is **repository tooling identity**: it is
NOT a HashV1 semantic digest, MUST NOT be placed in a
`DigestRef(domain=...)`, and never enters any runtime semantic identity.

<a id="spec-frozen-source-not-provisional"></a>

A `frozen` entry's normative source must not carry a provisional marker. The
marker convention: a line whose trimmed form starts (case-insensitively) with
`status:` and continues with `provisional`, or a line whose trimmed form
starts with the uppercase token `PROVISIONAL` followed by `:`, space, or end
of line. Historical prose that merely mentions the word "provisional" (e.g.
describing a superseded file) is not a marker.

## 4. Normative Rule Registry

Each entry is a `NormativeRuleDescriptor` with `rule_id`, `area`,
`normative_source` (a logical path), `source_anchor`, `requirement_level`,
`short_statement`, `conformance_refs`, and optional
`intentionally_uncovered_reason`.

<a id="registry-rule-id-syntax"></a>

`rule_id` syntax is exactly:

```text
RuleId := "AF-" AREA "-" DIGIT4
AREA   := [A-Z][A-Z0-9]*        ; must be an entry of spec/tables/rule-areas.json
DIGIT4 := [0-9]{4}              ; zero-padded, value 0001..9999, never 0000
```

RuleIds MUST NOT depend on document section numbering (`RULE-24.1.3` is
forbidden): document reorganization must never change a rule identity. The
area segment is validated against the closed area table; unknown areas are
rejected. New areas are additive registry additions to
`spec/tables/rule-areas.json`.

<a id="registry-rule-id-unique"></a>

A `RuleId`, once published, is unique and permanent: duplicate rule ids are a
registry error, and a RuleId MUST NOT be reused for a different normative
meaning. Retired rules keep their id.

`requirement_level` is the closed enum `must | must_not | should |
should_not | may`. Rule granularity is one independently violatable,
independently testable normative invariant per RuleId — not one rule per
prose sentence. "Strings MUST be NFC-normalized" and "map keys MUST be sorted
by unsigned UTF-8 bytes" are two rules.

<a id="registry-rule-anchor-exists"></a>

`source_anchor` must exist in the rule's normative source: a Markdown anchor
(`<a id="anchor">`) or a heading whose slug equals the anchor, or — for
AFIDL sources — a declared type name. Line numbers are forbidden as anchors.
Two rules must not bind the same `(logical_path, source_anchor)` pair.

<a id="registry-conformance-ref-resolves"></a>

Every `conformance_refs` entry must resolve to an existing stable
conformance identity (§6). A rule with at least one ref is `covered`. A rule
with no refs and a non-empty `intentionally_uncovered_reason` is
`intentionally_uncovered`; MUST/MUST_NOT rules in this class are exceptions
that must be listed in the frozen coverage summary (`coverage-summary.json`).
A rule with neither refs nor a reason is `missing_coverage` — always an
error.

## 5. Stable ABI Number Registry

Each entry is an `AbiNumberAssignment` with `schema_family` (a
`SpecificationId`), `type_name`, `number_kind` (`field | variant_case |
enum_value`), `number` (positive u32), `symbolic_name` (AFIDL lower-name
grammar), `status` (`active | reserved | retired`), and optional
`first_seen_schema_digest` (a real HashV1 `DigestRef`, domain
`type-schema`).

The registry exists because an AFIDL validator can only see the *current*
compilation unit: it cannot know that field 7 previously existed, was
removed, and is now proposed for reuse. This registry persists history as
status, not as multiple rows: each `(schema_family, type_name, number_kind,
number)` appears at most once.

<a id="registry-abi-number-never-reassigned"></a>

A `retired` number MUST NOT be reassigned to a different symbolic meaning —
within its `(schema_family, type_name, number_kind)` scope, the binding
between a number and its `symbolic_name` is permanent for all statuses. The
same symbolic name MUST NOT appear at two different numbers in the same
scope. Moving a name to a different number, or a number to a different name,
is a rejected reassignment (an explicit breaking specification change would
be required, which Phase 2B tooling does not automate).

<a id="registry-abi-assignment-unique"></a>

Registry integrity: `(schema_family, type_name, number_kind, number)` is
unique; a `symbolic_name` is unique within its `(schema_family, type_name,
number_kind)` scope; identifiers must match the AFIDL1 grammars (`type_name`
is `TYPE-NAME`, `symbolic_name` is a lower-name); `number` is positive.

<a id="registry-abi-registry-matches-schema"></a>

Every AFIDL module under `spec/idl/` is compiled and checked against the
registry for its family:

- every current field/case/value binding must have an `active` registry row
  with the identical symbolic name — unregistered assignments, uses of
  `retired` or `reserved` numbers, and any number/name divergence are
  rejected;
- every `active` registry row must correspond to a current schema binding —
  removing a field from a schema without explicitly retiring (or otherwise
  updating) the registry first is rejected;
- `reserved` rows are exempt from the schema match (they are held for future
  use, unbound).

Retirement is never automatic: `active → retired` transitions happen only by
editing `spec/registry/abi-numbers.json` as an explicit spec change. The
validator only checks consistency; it never rewrites registry files.

## 6. ConformanceCaseId

Conformance references are stable identities independent of filename order
and of Rust test function names (code function names are not normative
identity). One rule may reference many cases and one case may cover many
rules. Three forms exist:

1. **Fixture case id** — the `case_id` string already frozen inside a corpus
   JSON (e.g. `map/nfc-duplicate-key`, `digest/uppercase-hex`); reused as-is,
   never rewritten for format uniformity.
2. **File-based case id** — for corpora whose cases *are* files
   (`parser-invalid/*.afidl`), the manifest-listed file stem (e.g.
   `invalid-schema-id-uppercase`).
3. **Suite id** — `SUITE-<AREA>-<SLUG>` declared in
   `spec/conformance/suites.json`; one suite id may stand for a whole
   property-test suite or a frozen matrix file that has no per-row ids.

The validator scans `spec/conformance/**/*.json` for case ids and manifest
file stems, and `suites.json` for suite ids; unresolvable references are
rejected.

## 7. Canonical form of registry artifacts

<a id="registry-canonical-form"></a>

Registry JSON files are spec tooling, not ACE1 runtime objects, but they are
deterministically formatted and machine-checked:

- 2-space indent, LF line endings, no trailing whitespace, final newline;
- object keys sorted lexicographically at every level (the files must
  round-trip byte-identically through a key-sorting pretty printer);
- arrays in frozen canonical order: `specifications.json` by
  `specification_id`; `rules.json` by `rule_id`; `abi-numbers.json` by
  `(schema_family, type_name, number_kind, number)`; `rule-areas.json` by
  `area`; `suites.json` by `suite_id`. `number_kind` and other enums sort as
  their JSON strings.

Registry JSON bytes do not enter HashV1; a future Specification Release
Manifest phase may define an overall release digest.

## 8. Bootstrap

`agentflow.spec.registry@1` is defined by hand in
`spec/idl/specification-registry.afidl`, parsed with the Phase 2A compiler,
its per-type TypeSchemaDigests frozen in the AFIDL corpus
(`spec/conformance/afidl/`, case `specification-registry`), and its own
field/value numbering registered as `active` rows in `abi-numbers.json`
(with `first_seen_schema_digest` set from those frozen digests). This
self-hosting is not circular: the AFIDL source is the authority; the JSON
registries are instances; the digest domain remains the frozen `type-schema`
— no new HashV1 domain is introduced.

## 9. Validation command

`cargo run -p af-spec --bin spec-check -- check` validates all registries,
closed tables, AFIDL modules, coverage, and cross-references against the real
repository. Any error exits non-zero; `check` never writes, rewrites, or
"fixes" anything.
