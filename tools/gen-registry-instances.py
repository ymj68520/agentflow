#!/usr/bin/env python3
"""One-off authoring script for the Phase 2B registry instances.

Writes spec/registry/{specifications,rules,abi-numbers,coverage-summary}.json
and spec/conformance/suites.json in canonical registry form (sorted keys,
2-space indent, LF, final newline) — the same byte form serde_json's sorted
pretty printer emits, which spec-check verifies by round-trip.

Content digests are recomputed from the current normative sources; rerun this
script after editing any registered source file.
"""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def sha256_file(rel: str) -> str:
    return hashlib.sha256((ROOT / rel).read_bytes()).hexdigest()


def source_ref(rel: str) -> dict:
    return {
        "logical_path": rel,
        "content_digest": {"algorithm": "sha256", "hex": sha256_file(rel)},
    }


def write_json(rel: str, payload: dict) -> None:
    path = ROOT / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(payload, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {rel} ({len(text.splitlines())} lines)")


# ---------------------------------------------------------------------------
# rules.json
# ---------------------------------------------------------------------------

ACE1_SRC = "spec/encoding/ace1-grammar.md"
DIGEST_SRC = "spec/digest/hash-v1.md"
AFIDL_SRC = "spec/idl/afidl1-grammar.md"
REG_SRC = "spec/registry/registry-spec.md"

ARCH_UNCOVERED_REASON = (
    "Architectural invariant: not falsifiable by a data-driven fixture. "
    "Enforced by the typed encode(shape, value) API surface and the Phase 1 "
    "no-reflection / no-Any self-checks (docs/implementation/"
    "phase-01-canonical-encoding.md section 5)."
)


def rule(rule_id, area, source, anchor, statement, refs=None, uncovered=None):
    entry = {
        "rule_id": rule_id,
        "area": area,
        "normative_source": source,
        "source_anchor": anchor,
        "requirement_level": "must",
        "short_statement": statement,
    }
    entry["conformance_refs"] = refs or []
    if uncovered:
        entry["intentionally_uncovered_reason"] = uncovered
    return entry


RULES = [
    # --- ACE1 -------------------------------------------------------------
    rule("AF-ACE1-0001", "ACE1", ACE1_SRC, "ace1-schema-directed",
         "Encoding is schema-directed: the encoder is driven by an explicit "
         "machine type (Shape) and never reflects over arbitrary host objects.",
         uncovered=ARCH_UNCOVERED_REASON),
    rule("AF-ACE1-0002", "ACE1", ACE1_SRC, "ace1-utf8-canonical-bytes",
         "Canonical bytes are UTF-8 without BOM, contain no insignificant "
         "whitespace and have no leading or trailing data; decoders reject "
         "all of these.",
         ["dec/whitespace", "dec/trailing-newline", "dec/trailing-bytes"]),
    rule("AF-ACE1-0003", "ACE1", ACE1_SRC, "ace1-numeric-as-string",
         "Machine numerics are encoded as canonical decimal (or float64 hex) "
         "text inside JSON strings; JSON numbers are never the canonical "
         "authority and are rejected by the decoder.",
         ["dec/bare-number", "int32/max", "float64/one-tenth",
          "timestamp/2023-11-14"]),
    rule("AF-ACE1-0004", "ACE1", ACE1_SRC, "ace1-string-nfc",
         "String content is NFC-normalized: canonically equivalent inputs "
         "produce identical canonical bytes, and the decoder rejects non-NFC "
         "text.",
         ["string/nfc-decomposed-input", "string/nfc-hangul-decomposed-input",
          "string/nfc-multiple-combining"]),
    rule("AF-ACE1-0005", "ACE1", ACE1_SRC, "ace1-map-key-nfc",
         "Map keys are NFC-normalized before use.",
         ["map/nfc-key-input", "map/nfc-duplicate-key"]),
    rule("AF-ACE1-0006", "ACE1", ACE1_SRC, "ace1-map-key-ordering",
         "Map keys and record members are emitted in unsigned UTF-8 byte "
         "lexicographic order; input order never affects canonical bytes.",
         ["map/unsorted-input", "map/sorted-input-equivalent",
          "map/byte-order-case", "map/byte-order-multibyte"]),
    rule("AF-ACE1-0007", "ACE1", ACE1_SRC, "ace1-duplicate-key-rejection",
         "Duplicate map keys are rejected, including keys that collide only "
         "after NFC normalization; never last-write-wins.",
         ["map/duplicate-key", "map/nfc-duplicate-key", "dec/duplicate-keys"]),
    rule("AF-ACE1-0008", "ACE1", ACE1_SRC, "ace1-float64-exact",
         "float64 is encoded as exactly 16 lowercase IEEE-754 hex characters "
         "inside a JSON string (big-endian bit pattern).",
         ["float64/one-tenth", "float64/min-subnormal", "float64/max-finite",
          "dec/float-uppercase-hex", "dec/float-short-hex"]),
    rule("AF-ACE1-0009", "ACE1", ACE1_SRC, "ace1-float64-nonfinite",
         "NaN and infinities are rejected by both encoder and decoder.",
         ["float64/nan", "float64/nan-signaling", "float64/positive-infinity",
          "float64/negative-infinity", "dec/float-nan", "dec/float-inf"]),
    rule("AF-ACE1-0010", "ACE1", ACE1_SRC, "ace1-float64-negative-zero",
         "The float64 -0 bit pattern canonicalizes to +0; the decoder rejects "
         "the -0 pattern as non-canonical.",
         ["float64/negative-zero-canonicalized", "dec/float-negative-zero"]),
    rule("AF-ACE1-0011", "ACE1", ACE1_SRC, "ace1-bytes-base64url",
         "bytes are unpadded base64url; padding, the standard alphabet, "
         "impossible lengths and non-canonical trailing bits are rejected.",
         ["bytes/url-safe-alphabet", "bytes/all-byte-values-low",
          "dec/padded-base64url", "dec/standard-base64-alphabet"]),
    rule("AF-ACE1-0012", "ACE1", ACE1_SRC, "ace1-timestamp-duration-micros",
         "timestamp and duration are nominal int64 microsecond values encoded "
         "as canonical int64 text; cross-assignment between them and int64 is "
         "rejected.",
         ["timestamp/one-micro", "timestamp/negative-one", "duration/one-day",
          "duration/max", "nominal/timestamp-into-int64",
          "nominal/duration-into-timestamp"]),
    rule("AF-ACE1-0013", "ACE1", ACE1_SRC, "ace1-option-tagging",
         "Option is explicitly tagged: None encodes {\"none\":null} and "
         "Some(v) encodes {\"some\":v}; the null-sugar encoding and any other "
         "member shape are rejected, and any legal type may sit under an "
         "option.",
         ["option/null/none", "option/null/some-null", "option/unit/none",
          "option/nested-string/some-none", "nominal/null-into-option"]),
    rule("AF-ACE1-0014", "ACE1", ACE1_SRC, "ace1-closed-enum-variant",
         "Enums and variants are closed: unknown enum values and unknown "
         "variant cases are rejected (fail-closed).",
         ["enum/unknown-value", "variant/unknown-case",
          "dec/variant-unknown-case"]),
    # --- DIGEST -----------------------------------------------------------
    rule("AF-DIGEST-0001", "DIGEST", DIGEST_SRC, "digest-formula-exact-prefix",
         "HashV1(domain, payload) = SHA256(\"AF\" || 0x00 || domain || 0x00 "
         "|| payload); the prefix is the exact ASCII bytes AF + NUL.",
         ["prefix-in-payload/plan-core", "empty-payload/value",
          "ascii/plan-core"]),
    rule("AF-DIGEST-0002", "DIGEST", DIGEST_SRC, "digest-domain-separator",
         "The two NUL separators frame the domain byte-exactly, so payloads "
         "containing AF, NUL or full preimages can never be reinterpreted.",
         ["separator-in-payload/value", "prefix-in-payload/plan-core"]),
    rule("AF-DIGEST-0003", "DIGEST", DIGEST_SRC, "digest-sha256-only",
         "SHA-256 is the only HashV1 algorithm in v1; other algorithms are "
         "rejected.",
         ["digest/unknown-algorithm"]),
    rule("AF-DIGEST-0004", "DIGEST", DIGEST_SRC, "digest-ref-domain-retained",
         "A DigestRef retains its domain for its whole life; digests never "
         "degrade to bare hex strings.",
         ["digest/unknown-domain", "utf8/journal-event-payload"]),
    rule("AF-DIGEST-0005", "DIGEST", DIGEST_SRC, "digest-ref-lowercase-hex",
         "DigestRef hex is exactly 64 lowercase hexadecimal characters; "
         "uppercase hex, wrong lengths and non-hex characters are rejected.",
         ["digest/uppercase-hex", "digest/short-hex", "digest/long-hex",
          "digest/non-hex"]),
    rule("AF-DIGEST-0006", "DIGEST", DIGEST_SRC, "digest-domain-registry-closed",
         "The HashV1 domain set is closed and centrally registered; unknown "
         "domains are rejected and adding a domain is an explicit registry "
         "addition.",
         ["digest/unknown-domain"]),
    rule("AF-DIGEST-0007", "DIGEST", DIGEST_SRC, "digest-domain-separation",
         "For a fixed payload, hashing under two different registry domains "
         "yields different digests; the invariant holds over the whole "
         "registry.",
         ["SUITE-DIGEST-DOMAIN-SEPARATION"]),
    # --- AFIDL ------------------------------------------------------------
    rule("AF-AFIDL-0001", "AFIDL", AFIDL_SRC, "afidl-language-major",
         "An AFIDL1 module declares afidl 1; any other language major is "
         "rejected.",
         ["invalid-language-version"]),
    rule("AF-AFIDL-0002", "AFIDL", AFIDL_SRC, "afidl-specification-id-syntax",
         "Schema family identity is a strict SpecificationId: lowercase kebab "
         "namespace segments and a positive major without leading zeros.",
         ["invalid-schema-id-uppercase", "invalid-schema-id-major-zero",
          "invalid-schema-id-no-major", "invalid-schema-id-empty-segment"]),
    rule("AF-AFIDL-0003", "AFIDL", AFIDL_SRC, "afidl-primitive-universe-closed",
         "The primitive universe is closed to the 13 registered builtin "
         "keywords; unknown lowercase identifiers in type position become "
         "named references and must resolve.",
         ["unknown-lowercase-primitive", "primitives-coverage"]),
    rule("AF-AFIDL-0004", "AFIDL", AFIDL_SRC, "afidl-field-numbering-positive",
         "field_no is a positive u32; zero is rejected.",
         ["field-no-zero"]),
    rule("AF-AFIDL-0005", "AFIDL", AFIDL_SRC, "afidl-field-numbering-unique",
         "field_no values are unique within their record.",
         ["duplicate-field-no"]),
    rule("AF-AFIDL-0006", "AFIDL", AFIDL_SRC, "afidl-case-numbering-positive",
         "case_no is a positive u32; zero is rejected.",
         ["case-no-zero"]),
    rule("AF-AFIDL-0007", "AFIDL", AFIDL_SRC, "afidl-case-numbering-unique",
         "case_no values are unique within their variant.",
         ["duplicate-case-no"]),
    rule("AF-AFIDL-0008", "AFIDL", AFIDL_SRC, "afidl-value-numbering-positive",
         "value_no is a positive u32; zero is rejected.",
         ["enum-no-zero"]),
    rule("AF-AFIDL-0009", "AFIDL", AFIDL_SRC, "afidl-value-numbering-unique",
         "value_no values are unique within their enum.",
         ["duplicate-enum-no"]),
    rule("AF-AFIDL-0010", "AFIDL", AFIDL_SRC, "afidl-source-order-nonsemantic",
         "Source formatting, comments, declaration order and member source "
         "order are non-semantic: reordering never changes canonical bytes or "
         "digests.",
         ["record-reordered", "enum-reordered", "variant-reordered",
          "module-order-reordered", "record-multi-field-comments"]),
    rule("AF-AFIDL-0011", "AFIDL", AFIDL_SRC, "afidl-number-binding-semantic",
         "Number and name bindings are semantic: changing a field_no, "
         "case_no, value_no or member name changes the TypeSchemaDigest.",
         ["mut-field-no", "mut-case-no", "mut-enum-no", "mut-field-name"]),
    rule("AF-AFIDL-0012", "AFIDL", AFIDL_SRC, "afidl-named-ref-resolution",
         "Every named reference must resolve to a declared type in the "
         "compilation unit; unknown references are rejected.",
         ["unknown-type-reference"]),
    rule("AF-AFIDL-0013", "AFIDL", AFIDL_SRC, "afidl-alias-cycles",
         "Alias cycles, direct or indirect, are rejected.",
         ["alias-cycle", "alias-self-cycle"]),
    rule("AF-AFIDL-0014", "AFIDL", AFIDL_SRC, "afidl-recursion-restriction",
         "Direct or indirect recursive named references - through record "
         "fields, variant cases, alias expansion or any list/map/option "
         "wrapper - are rejected (Phase 2A scoped restriction).",
         ["recursive-record", "mutual-recursion", "recursion-through-list"]),
    rule("AF-AFIDL-0015", "AFIDL", AFIDL_SRC, "afidl-type-schema-digest-formula",
         "TypeSchemaDigest = SHA256(\"AF\\0type-schema\\0\" || "
         "ACE1(canonical TypeSchema)), computed per named type.",
         ["machine-primitives", "record-multi-field"]),
    # --- SPEC -------------------------------------------------------------
    rule("AF-SPEC-0001", "SPEC", REG_SRC, "spec-specification-id-unique",
         "A SpecificationId is unique across the Specification Registry; "
         "duplicate entries are a registry error.",
         ["spec-registry/duplicate-specification-id"]),
    rule("AF-SPEC-0002", "SPEC", REG_SRC, "spec-normative-source-integrity",
         "Every registered normative source exists at a safe repo-relative "
         "path and its ArtifactFileDigest (plain SHA-256 of file bytes) "
         "matches; an ArtifactFileDigest is repository tooling identity and "
         "never a HashV1 DigestRef.",
         ["spec-registry/missing-source", "spec-registry/content-digest-mismatch"]),
    rule("AF-SPEC-0003", "SPEC", REG_SRC, "spec-frozen-source-not-provisional",
         "A frozen specification's normative source carries no provisional "
         "status marker.",
         ["spec-registry/frozen-source-provisional"]),
    # --- REGISTRY ---------------------------------------------------------
    rule("AF-REGISTRY-0001", "REGISTRY", REG_SRC, "registry-rule-id-syntax",
         "RuleId syntax is AF-<AREA>-<4DIGIT> with the area from the closed "
         "rule-area table and number 0001..9999; ids derived from document "
         "section numbering are forbidden.",
         ["spec-registry/bad-rule-id", "spec-registry/unknown-rule-area"]),
    rule("AF-REGISTRY-0002", "REGISTRY", REG_SRC, "registry-rule-id-unique",
         "RuleIds are unique and are never reused for a different normative "
         "meaning.",
         ["spec-registry/duplicate-rule-id"]),
    rule("AF-REGISTRY-0003", "REGISTRY", REG_SRC, "registry-rule-anchor-exists",
         "Every rule's source_anchor exists in its normative source (Markdown "
         "anchor or heading slug, or an AFIDL type name); two rules never "
         "bind the same (path, anchor) pair; line numbers are forbidden as "
         "anchors.",
         ["spec-registry/missing-anchor"]),
    rule("AF-REGISTRY-0004", "REGISTRY", REG_SRC, "registry-conformance-ref-resolves",
         "Every conformance reference resolves to an existing fixture case "
         "id, file-based case id or declared suite id.",
         ["spec-registry/unknown-conformance-case"]),
    rule("AF-REGISTRY-0005", "REGISTRY", REG_SRC, "registry-abi-number-never-reassigned",
         "A retired ABI number is never reassigned to a different symbolic "
         "meaning, and a symbolic name never moves to a different number, "
         "within its (schema_family, type_name, number_kind) scope.",
         ["spec-registry/retired-number-reuse",
          "spec-registry/name-rebound-to-new-number"]),
    rule("AF-REGISTRY-0006", "REGISTRY", REG_SRC, "registry-abi-registry-matches-schema",
         "Every AFIDL module under spec/idl matches the ABI registry for its "
         "family: all current bindings have active rows with identical names, "
         "all active rows have schema counterparts, retired and reserved "
         "numbers are unused.",
         ["spec-registry/unregistered-abi-assignment",
          "spec-registry/active-assignment-missing",
          "spec-registry/retired-number-in-use"]),
    rule("AF-REGISTRY-0007", "REGISTRY", REG_SRC, "registry-abi-assignment-unique",
         "(schema_family, type_name, number_kind, number) is unique and a "
         "symbolic name is unique within its scope; identifiers follow the "
         "AFIDL grammars and numbers are positive.",
         ["spec-registry/duplicate-abi-assignment"]),
    rule("AF-REGISTRY-0008", "REGISTRY", REG_SRC, "registry-canonical-form",
         "Registry JSON files are deterministically formatted (2-space indent, "
         "LF, final newline, lexicographically sorted keys) and their arrays "
         "follow the frozen canonical orders.",
         ["spec-registry/canonical-order-violation"]),
]

RULES.sort(key=lambda r: r["rule_id"])
write_json("spec/registry/rules.json", {
    "registry_revision": 1,
    "rules": RULES,
})

# ---------------------------------------------------------------------------
# specifications.json
# ---------------------------------------------------------------------------

SPECS = [
    ("agentflow.core.machine-primitives@1", "schema",
     "spec/idl/core/machine-primitives.afidl",
     "The AFIDL1 builtin machine primitive universe expressed as a parseable "
     "nominal-alias module; its 13 alias digests are frozen in the AFIDL "
     "corpus."),
    ("agentflow.digest.hash-domains@1", "digest_domain_registry",
     "spec/tables/hash-domains.json",
     "Closed central registry of HashV1 digest domains."),
    ("agentflow.digest.hash-v1@1", "digest",
     "spec/digest/hash-v1.md",
     "HashV1 digest foundation: exact formula, SHA-256 only, DigestRef and "
     "the domain separation invariant."),
    ("agentflow.encoding.ace1@1", "encoding",
     "spec/encoding/ace1-grammar.md",
     "ACE1: the deterministic, schema-directed, UTF-8 canonical encoding of "
     "AgentFlow values (strict JSON subset)."),
    ("agentflow.idl.afidl-primitives@1", "closed_table",
     "spec/tables/afidl-primitives.json",
     "Closed normative table of the AFIDL1 builtin primitive universe."),
    ("agentflow.idl.afidl@1", "schema_language",
     "spec/idl/afidl1-grammar.md",
     "AFIDL1: the closed schema definition language, its grammar and its "
     "identity model."),
    ("agentflow.idl.type-schema@1", "schema_language",
     "spec/idl/afidl1-grammar.md",
     "The canonical TypeSchema machine representation and the TypeSchemaDigest "
     "identity formula (sections 6-7 of the AFIDL1 grammar)."),
    ("agentflow.registry.core@1", "registry",
     "spec/registry/registry-spec.md",
     "Registry mechanics: Specification Registry, RuleId system, Stable ABI "
     "Number Registry, ConformanceCaseId and canonical registry form."),
    ("agentflow.registry.rule-areas@1", "closed_table",
     "spec/tables/rule-areas.json",
     "Closed table of RuleId areas."),
    ("agentflow.spec.registry@1", "schema",
     "spec/idl/specification-registry.afidl",
     "Normative shape of the Specification, Rule and ABI-number registries "
     "(self-hosting bootstrap module)."),
]

spec_entries = [
    {
        "specification_id": spec_id,
        "kind": kind,
        "status": "frozen",
        "normative_source": source_ref(path),
        "description": description,
    }
    for spec_id, kind, path, description in SPECS
]
spec_entries.sort(key=lambda e: e["specification_id"])
write_json("spec/registry/specifications.json", {
    "registry_revision": 1,
    "specifications": spec_entries,
})

# ---------------------------------------------------------------------------
# abi-numbers.json — the self-hosted numbering of agentflow.spec.registry@1
# ---------------------------------------------------------------------------

REG_FAMILY = "agentflow.spec.registry@1"

ENUM_VALUES = {
    "AbiNumberKind": ["field", "variant_case", "enum_value"],
    "AbiNumberStatus": ["active", "reserved", "retired"],
    "RequirementLevel": ["must", "must_not", "should", "should_not", "may"],
    "SpecificationKind": [
        "encoding", "digest", "digest_domain_registry", "schema_language",
        "schema", "closed_table", "registry",
    ],
    "SpecificationStatus": ["draft", "frozen", "deprecated", "withdrawn"],
}

RECORD_FIELDS = {
    "AbiNumberAssignment": [
        "schema_family", "type_name", "number_kind", "number",
        "symbolic_name", "status", "first_seen_schema_digest",
    ],
    "ArtifactFileDigest": ["algorithm", "hex"],
    "DigestRef": ["alg", "domain", "hex"],
    "NormativeRuleDescriptor": [
        "rule_id", "area", "normative_source", "source_anchor",
        "requirement_level", "short_statement", "conformance_refs",
        "intentionally_uncovered_reason",
    ],
    "SpecificationArtifactRef": ["logical_path", "content_digest"],
    "SpecificationRegistryEntry": [
        "specification_id", "kind", "status", "normative_source",
        "schema_digest", "description",
    ],
}

# Per-type TypeSchemaDigest of agentflow.spec.registry@1, frozen by
# tools/gen-afidl-corpus.py in spec/conformance/afidl/
# (type-schema-digest-golden.json, case `specification-registry`).
TYPE_DIGESTS = {
    "AbiNumberAssignment": "8c0e45ac0562494b4ce3e6a37842cdde65cad1ec8a2f61e70dc39699a280834c",
    "AbiNumberKind": "87f49f06ff0899b60bf65f7476706b7ccee8a65026dd684940f2c4c6a15277ea",
    "AbiNumberStatus": "7068d098e923393e292cf3fa9708b5d9346e16fe8abd61b728051f3ad2336fe5",
    "ArtifactFileDigest": "4d85e05e5ab5aee08fb696e2b4ebf50e69eb14c0c856d4bc4898cf9cfa21f93b",
    "DigestRef": "5a8c24604b918a20e772e87565143c6a83270bf98088a289fda4bc46111adc33",
    "NormativeRuleDescriptor": "5498e71bbe4329a1f2b0c160084a945cd744b8e5cf12d7d14f00d091e4ef93c8",
    "RequirementLevel": "239f0abbb8ef00f1330516d3791ac6347b8e38513c85d9ee121b7d4960bbb4c3",
    "SpecificationArtifactRef": "afdeef15c0a24d0385fac9159b2b81126a776411ed95983a769b619307e2f841",
    "SpecificationKind": "1bd62de836089e1f75fd489e87010fab6cd6e6951c83d515fa38c2aab12fb293",
    "SpecificationRegistryEntry": "091d836ac9107f19813b36c15e1c812779417beb4a960303e474ec981bd3e4e4",
    "SpecificationStatus": "ba664592202004949b4b967c921e667c7765ecaa4e7dfb1254f72b31901ec7f3",
}


def digest_ref(hex_value: str) -> dict:
    return {"alg": "sha256", "domain": "type-schema", "hex": hex_value}


assignments = []
for type_name, values in ENUM_VALUES.items():
    for no, name in enumerate(values, start=1):
        assignments.append({
            "schema_family": REG_FAMILY,
            "type_name": type_name,
            "number_kind": "enum_value",
            "number": no,
            "symbolic_name": name,
            "status": "active",
            "first_seen_schema_digest": digest_ref(TYPE_DIGESTS[type_name]),
        })
for type_name, fields in RECORD_FIELDS.items():
    for no, name in enumerate(fields, start=1):
        assignments.append({
            "schema_family": REG_FAMILY,
            "type_name": type_name,
            "number_kind": "field",
            "number": no,
            "symbolic_name": name,
            "status": "active",
            "first_seen_schema_digest": digest_ref(TYPE_DIGESTS[type_name]),
        })
assignments.sort(key=lambda a: (
    a["schema_family"], a["type_name"], a["number_kind"], a["number"]))
write_json("spec/registry/abi-numbers.json", {
    "registry_revision": 1,
    "assignments": assignments,
})

# ---------------------------------------------------------------------------
# coverage-summary.json — the frozen intentionally-uncovered exception set
# ---------------------------------------------------------------------------

uncovered = [
    {"rule_id": r["rule_id"], "reason": r["intentionally_uncovered_reason"]}
    for r in RULES if r.get("intentionally_uncovered_reason")
]
uncovered.sort(key=lambda e: e["rule_id"])
write_json("spec/registry/coverage-summary.json", {
    "coverage_revision": 1,
    "intentionally_uncovered": uncovered,
    "missing_coverage": [],
})

# ---------------------------------------------------------------------------
# suites.json — stable identities for non-fixture conformance coverage
# ---------------------------------------------------------------------------

write_json("spec/conformance/suites.json", {
    "kind": "agentflow.conformance.suites",
    "suite_revision": 1,
    "suites": [
        {
            "suite_id": "SUITE-DIGEST-DOMAIN-SEPARATION",
            "description": (
                "Cross-domain digest separation: the same payload hashed "
                "under every frozen HashV1 domain yields pairwise-distinct "
                "digests. Covers the af-core property test over the full "
                "registry and the frozen matrix in "
                "spec/conformance/digest/domain-separation.json."
            ),
            "targets": [
                "af-core digest domain-separation property test",
                "spec/conformance/digest/domain-separation.json",
            ],
        },
    ],
})
