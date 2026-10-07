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
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The independent AFIDL1 implementation (tools/gen-afidl-corpus.py): used here
# to *derive* ABI assignments from the normative schema modules instead of
# hand-copying them.
_spec = importlib.util.spec_from_file_location(
    "afidl_corpus", Path(__file__).resolve().parent / "gen-afidl-corpus.py")
afidl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(afidl)


def digest_ref(hex_value: str) -> dict:
    return {"alg": "sha256", "domain": "type-schema", "hex": hex_value}


def derive_assignments(rel_path, status="active"):
    """Derives one assignment row per declared field / enum value / variant
    case of an AFIDL module, plus the per-type TypeSchemaDigest pin."""
    module = afidl.parse_afidl((ROOT / rel_path).read_text(encoding="utf-8"))
    family = f"{module['spec']['namespace']}@{module['spec']['major']}"
    rows = []
    for declaration in module["declarations"]:
        schema = afidl.canonical_type(family, declaration)
        _payload, digest = afidl.type_schema_digest(schema)
        kind = declaration["kind"]
        if kind == "alias":
            continue  # aliases carry no numbering
        if kind == "enum":
            members = [(v["no"], v["name"], "enum_value") for v in declaration["values"]]
        elif kind == "record":
            members = [(f["no"], f["name"], "field") for f in declaration["fields"]]
        else:
            members = [(c["no"], c["name"], "variant_case") for c in declaration["cases"]]
        for number, name, number_kind in members:
            rows.append({
                "schema_family": family,
                "type_name": declaration["name"],
                "number_kind": number_kind,
                "number": number,
                "symbolic_name": name,
                "status": status,
                "first_seen_schema_digest": digest_ref(digest),
            })
    return rows, family


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
SEMANTIC_SRC = "spec/semantic/core-semantic-types.md"

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
    rule("AF-AFIDL-0016", "AFIDL", AFIDL_SRC, "afidl-import-exact-specification-id",
         "An import names an exact SpecificationId, major included; version "
         "ranges (latest, ^1, >=1, 1.x, *), file paths, wildcards, re-exports "
         "and per-type alias imports do not exist in AFIDL1.",
         ["invalid-import-spec-id", "malformed-import",
          "SUITE-AFIDL-IMPORT-EXACT-IDENTITY"]),
    rule("AF-AFIDL-0017", "AFIDL", AFIDL_SRC, "afidl-import-explicit-universe",
         "Import resolution uses an explicit, immutable, in-memory schema "
         "universe supplied by the caller: no registry lookup, no latest-version "
         "resolution, no filesystem read and no network access. An import of a "
         "specification absent from the universe is unknown_import_specification.",
         ["unknown-import-specification", "SUITE-AFIDL-IMPORT-EXPLICIT-UNIVERSE"]),
    rule("AF-AFIDL-0018", "AFIDL", AFIDL_SRC, "afidl-import-alias-grammar",
         "An import alias matches [a-z][a-z0-9_]* and is not a reserved keyword; "
         "aliases are unique within the module.",
         ["reserved-import-alias", "duplicate-import-alias"]),
    rule("AF-AFIDL-0019", "AFIDL", AFIDL_SRC, "afidl-import-specification-unique",
         "A module holds at most one alias per imported specification family, so "
         "one specification has exactly one source spelling.",
         ["duplicate-import-specification"]),
    rule("AF-AFIDL-0020", "AFIDL", AFIDL_SRC, "afidl-import-no-self",
         "Importing the module's own schema id is rejected; local types are "
         "referenced unqualified.",
         ["self-import"]),
    rule("AF-AFIDL-0021", "AFIDL", AFIDL_SRC, "afidl-import-header-position",
         "Import declarations appear only between the schema declaration and the "
         "first type declaration; an import after a declaration is rejected.",
         ["import-after-declaration"]),
    rule("AF-AFIDL-0022", "AFIDL", AFIDL_SRC, "afidl-qualified-ref-not-primitive",
         "A qualified reference names a named AFIDL declaration only: qualified "
         "primitives such as core.int64 are rejected, because builtin primitives "
         "are always spelled bare.",
         ["qualified-primitive-ref"]),
    rule("AF-AFIDL-0023", "AFIDL", AFIDL_SRC, "afidl-external-ref-qualified",
         "An external type is reachable only through <import-alias>.<TypeName>. "
         "An unqualified name never resolves to an imported type (no implicit "
         "import, no shadowing, no resolution-order dependence), and a qualified "
         "reference never falls back to a local reference.",
         ["qualified-ref-without-import", "unknown-import-alias",
          "unknown-imported-type", "import-basic", "import-multiple",
          "import-nested-container-ref"]),
    rule("AF-AFIDL-0024", "AFIDL", AFIDL_SRC, "afidl-imports-excluded-from-identity",
         "Import declarations and import aliases do not enter the canonical "
         "TypeSchema record (schema/name/body); a qualified reference lowers to "
         "the imported specification's full logical identity. An alias rename, an "
         "import reordering or an unused import therefore leaves every canonical "
         "byte and every TypeSchemaDigest unchanged.",
         ["import-alias-rename-a", "import-alias-rename-b", "import-order-a",
          "import-order-b", "import-unused-a", "import-unused-b",
          "SUITE-AFIDL-IMPORT-IDENTITY-ISOLATION"]),
    rule("AF-AFIDL-0025", "AFIDL", AFIDL_SRC, "afidl-cross-module-type-graph-acyclic",
         "The global named-type dependency graph over a module set - through "
         "alias targets, record fields, variant payloads and every list/map/option "
         "wrapper - must be acyclic; imports never bypass the recursion "
         "restriction. The module import graph itself may contain cycles.",
         ["cross-module-direct-cycle", "cross-module-indirect-cycle",
          "mutual-import-without-type-cycle"]),
    rule("AF-AFIDL-0026", "AFIDL", AFIDL_SRC, "afidl-import-graph-may-cycle",
         "Two modules may import each other; only a type dependency between them "
         "is a cycle. Requiring an acyclic import graph would reject legal "
         "schemas, so the recursion restriction is stated over the type graph.",
         ["mutual-import-without-type-cycle"]),
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

SEMANTIC_RULES = [
    rule("AF-SEMANTIC-0001", "SEMANTIC", SEMANTIC_SRC,
         "semantic-workflow-primitive-set-closed",
         "The workflow semantic primitive universe is closed to the nine "
         "registered primitives with value_no 1..9; no other type may enter "
         "the workflow value layer.",
         ["constant/unit", "constant/duration", "primitive/machine-only-alias-int32"]),
    rule("AF-SEMANTIC-0002", "SEMANTIC", SEMANTIC_SRC,
         "semantic-machine-only-primitives-prohibited",
         "int32, uint32, uint64 and float64 are machine-only and MUST NOT "
         "surface as workflow value types at any point.",
         ["primitive/machine-only-alias-float64",
          "primitive/machine-only-alias-uint64",
          "primitive/machine-only-record-field",
          "constant/machine-only-case"]),
    rule("AF-SEMANTIC-0003", "SEMANTIC", SEMANTIC_SRC,
         "semantic-type-dag",
         "Composite workflow types are expressed as a document-local indexed "
         "type DAG whose list/map_string/option cases reference other nodes by "
         "SemanticTypeId; recursive AFIDL schemas are neither required nor "
         "permitted.",
         ["type/nested-list-map-option", "type/unknown-node-reference"]),
    rule("AF-SEMANTIC-0004", "SEMANTIC", SEMANTIC_SRC,
         "semantic-type-dag-ids",
         "Semantic type ids are positive and unique within the document; the "
         "canonical type table is ordered by id ascending and source order "
         "never affects identity.",
         ["type/id-zero", "type/duplicate-id", "canonical/types-sorted",
          "canonical/types-shuffled"]),
    rule("AF-SEMANTIC-0005", "SEMANTIC", SEMANTIC_SRC,
         "semantic-type-dag-acyclic",
         "The semantic type DAG MUST be acyclic: a type that expands through "
         "itself, directly or transitively, is rejected.",
         ["type/self-cycle", "type/two-node-cycle", "type/longer-cycle"]),
    rule("AF-SEMANTIC-0006", "SEMANTIC", SEMANTIC_SRC,
         "semantic-schema-ref-pins-digest",
         "A TypeSchemaRef pins <SpecificationId>/<TypeName> together with the "
         "exact TypeSchemaDigest it was authored against; a published "
         "reference is never re-resolved against a later schema of that name.",
         ["type/schema-ref", "schema/wrong-digest"]),
    rule("AF-SEMANTIC-0007", "SEMANTIC", SEMANTIC_SRC,
         "semantic-schema-ref-digest-must-match",
         "A TypeSchemaRef is valid only when the identity resolves in the "
         "supplied TypeSchemaUniverse and the pinned digest equals that "
         "schema's actual canonical digest; a merely well-formed digest is "
         "not sufficient.",
         ["schema/wrong-digest", "schema/unknown", "schema/digest-bad-hex",
          "schema/digest-bad-algorithm", "schema/digest-wrong-domain"]),
    rule("AF-SEMANTIC-0008", "SEMANTIC", SEMANTIC_SRC,
         "semantic-digest-ref-value-shape",
         "A digest reference is the DigestRefValue record "
         "(algorithm/domain/hex); algorithm is sha256, the domain is a member "
         "of the frozen HashV1 registry (type-schema for a TypeSchemaRef) and "
         "hex is 64 lowercase hexadecimal characters. A bare digest string is "
         "forbidden.",
         ["schema/digest-bad-algorithm", "schema/digest-wrong-domain",
          "schema/digest-bad-hex"]),
    rule("AF-SEMANTIC-0009", "SEMANTIC", SEMANTIC_SRC,
         "semantic-value-slot-immutable",
         "A value slot is an immutable SSA binding: one producer and "
         "zero-or-more consumers, with no mutation operation. Phase 3A "
         "validates slot identity and type only; producer consistency belongs "
         "to the semantic IR dataflow stage.",
         ["expr/value-slot", "value-slot/read-unknown-slot"]),
    rule("AF-SEMANTIC-0010", "SEMANTIC", SEMANTIC_SRC,
         "semantic-local-ids-positive-unique",
         "Local ids in every namespace are > 0 and unique within the "
         "document; 0 is reserved as invalid/unset and no further reserved "
         "ranges exist.",
         ["type/id-zero", "value-slot/id-zero", "expression/id-zero",
          "resource/id-zero", "value-slot/duplicate-id",
          "expression/duplicate-id", "resource/duplicate-id",
          "type/duplicate-id"]),
    rule("AF-SEMANTIC-0011", "SEMANTIC", SEMANTIC_SRC,
         "semantic-namespace-separation",
         "SemanticTypeId, ValueSlotId, ExpressionId and ResourceSlotId are "
         "four independent namespaces: identical numbers in two namespaces "
         "denote unrelated objects, and a value never travels through a "
         "resource slot.",
         ["SUITE-SEMANTIC-NAMESPACE-SEPARATION",
          "resource/unknown-requirement-schema"]),
    rule("AF-SEMANTIC-0012", "SEMANTIC", SEMANTIC_SRC,
         "semantic-primitive-constants-closed",
         "PrimitiveConstant is closed to the nine workflow primitives and its "
         "cases map 1:1 onto SemanticPrimitiveType; bytes payloads are "
         "canonical unpadded base64url.",
         ["constant/unit", "constant/bytes", "constant/machine-only-case",
          "expression/invalid-bytes-constant"]),
    rule("AF-SEMANTIC-0013", "SEMANTIC", SEMANTIC_SRC,
         "semantic-expression-pure",
         "Expressions are pure and deterministic: no I/O, no clock, no "
         "randomness, no environment, no policy or registry query and no "
         "resource access; they never produce effects.",
         ["SUITE-SEMANTIC-PURITY"]),
    rule("AF-SEMANTIC-0014", "SEMANTIC", SEMANTIC_SRC,
         "semantic-expression-closed-set",
         "The expression body set is closed to the fifteen registered "
         "expressions; arithmetic, string operators, indexing, variant "
         "inspection and every fallible operation are absent.",
         ["SUITE-SEMANTIC-CLOSED-EXPRESSION-SET"]),
    rule("AF-SEMANTIC-0015", "SEMANTIC", SEMANTIC_SRC,
         "semantic-expression-dag",
         "Expressions form a DAG: forward references are legal and cycles "
         "(self, two-node or longer) are rejected.",
         ["expression/self-cycle", "expression/two-node-cycle",
          "expression/longer-cycle", "expr/list-construct"]),
    rule("AF-SEMANTIC-0016", "SEMANTIC", SEMANTIC_SRC,
         "semantic-expression-references-valid",
         "Every expression, value slot and semantic type reference must "
         "resolve; unresolved references are rejected.",
         ["expression/unknown-reference", "value-slot/read-unknown-slot",
          "type/unknown-node-reference", "value-slot/unknown-type"]),
    rule("AF-SEMANTIC-0017", "SEMANTIC", SEMANTIC_SRC,
         "semantic-expression-static-type-correctness",
         "The validator implements a real type checker: every expression "
         "derives its result type from its body and the declared result_type "
         "must equal the derived type.",
         ["expr/equals-primitive", "expr/if-then-else-then",
          "expression/result-type-mismatch", "option-some/derivation-mismatch",
          "bool/not-non-bool", "bool/and-non-bool", "bool/or-non-bool",
          "if/non-bool-condition", "if/branch-type-mismatch",
          "equals/type-mismatch", "equals/not-comparable-record",
          "equals/not-comparable-list", "list/element-type-mismatch",
          "map/value-type-mismatch"]),
    rule("AF-SEMANTIC-0018", "SEMANTIC", SEMANTIC_SRC,
         "semantic-field-selection-by-number",
         "Record construction and field_get select fields by field_no; "
         "variant construction and enum selection use case_no and value_no. "
         "Numbers are the durable schema ABI, names remain value-encoding "
         "keys.",
         ["expr/record-construct", "expr/field-get",
          "record/unknown-field-no", "field-get/unknown-field-no",
          "record/duplicate-field", "record/missing-field",
          "record/field-type-mismatch", "record/target-not-record",
          "variant/unknown-case", "variant/payload-mismatch",
          "variant/target-not-variant", "enum/unknown-value",
          "enum/target-not-enum", "field-get/target-not-record"]),
    rule("AF-SEMANTIC-0019", "SEMANTIC", SEMANTIC_SRC,
         "semantic-map-key-rules",
         "Map construction applies the ACE1 key invariants during validation: "
         "keys are NFC-normalized, keys colliding after normalization are "
         "rejected, and entries are stored in unsigned UTF-8 byte order.",
         ["expr/map-construct-nfc-key", "expr/map-construct-empty",
          "map/duplicate-nfc-key"]),
    rule("AF-SEMANTIC-0020", "SEMANTIC", SEMANTIC_SRC,
         "semantic-resource-slot-separate-namespace",
         "A resource slot is a logical requirement declaration in its own "
         "namespace: values never travel through resource slots and a "
         "ResourceSlotId is unrelated to a ValueSlotId with the same number.",
         ["SUITE-SEMANTIC-NAMESPACE-SEPARATION"]),
    rule("AF-SEMANTIC-0021", "SEMANTIC", SEMANTIC_SRC,
         "semantic-resource-slot-immutable",
         "A resource slot's definition is immutable for the life of the "
         "document and its requirement_type pins an exact contract schema.",
         ["SUITE-SEMANTIC-RESOURCE-IMMUTABILITY"]),
    rule("AF-SEMANTIC-0022", "SEMANTIC", SEMANTIC_SRC,
         "semantic-resource-not-runtime-handle",
         "No ResourceHandle, binding snapshot or runtime resource object "
         "exists at the semantic layer; a resource slot is a logical "
         "requirement only.",
         ["SUITE-SEMANTIC-NO-RUNTIME-HANDLE"]),
    rule("AF-SEMANTIC-0023", "SEMANTIC", SEMANTIC_SRC,
         "semantic-canonical-ordering",
         "Type nodes, value slots, expressions and resource slots are "
         "canonically ordered by id ascending; source order never enters "
         "identity.",
         ["canonical/types-sorted", "canonical/types-shuffled",
          "canonical/expressions-sorted", "canonical/expressions-shuffled"]),
    rule("AF-SEMANTIC-0024", "SEMANTIC", SEMANTIC_SRC,
         "semantic-no-new-hash-domain",
         "Phase 3A introduces no new HashV1 domain: type identity comes from "
         "TypeSchemaDigest and the semantic objects are ordinary ACE1 objects.",
         ["SUITE-SEMANTIC-NO-NEW-DOMAIN"]),
]
RULES.extend(SEMANTIC_RULES)

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
     "AFIDL1: the closed schema definition language, its grammar, its "
     "identity model and (Phase 2C) its cross-module import capability. "
     "Imports are an additive AFIDL1 source feature: the language major stays "
     "1 and every module without imports keeps its exact previous meaning "
     "and digests."),
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
    ("agentflow.semantic.core@1", "schema",
     "spec/idl/semantic-core.afidl",
     "Normative shape of the workflow semantic core: primitive universe, "
     "semantic type DAG, digest-pinned references, value slots, pure "
     "expressions and logical resource slots."),
    ("agentflow.semantic.types@1", "schema_language",
     "spec/semantic/core-semantic-types.md",
     "Workflow semantic core semantics: machine vs workflow primitives, "
     "type DAG rules, exact pinning, SSA value slots, expression purity and "
     "type checking, resource slot scope."),
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
# abi-numbers.json — stable numbering, derived from the normative modules
# ---------------------------------------------------------------------------

SCHEMA_MODULES = [
    "spec/idl/specification-registry.afidl",
    "spec/idl/semantic-core.afidl",
]

assignments = []
for module in SCHEMA_MODULES:
    rows, family = derive_assignments(module)
    print(f"derived {len(rows)} ABI assignments from {module} ({family})")
    assignments.extend(rows)

# Deterministic canonical order: (schema_family, type_name, number_kind, number).
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
            "suite_id": "SUITE-AFIDL-IMPORT-EXACT-IDENTITY",
            "description": (
                "An AFIDL import names an exact SpecificationId and nothing "
                "else. Covers the import grammar itself, which admits only "
                "ns-segment ('.' ns-segment)* '@' MAJOR as its target: there is "
                "no syntactic position for a version range, a path, a wildcard "
                "or a per-type alias, so the rejected forms are rejected by the "
                "grammar rather than by a post-hoc check."
            ),
            "targets": [
                "af-idl import grammar (import-decl production)",
                "spec/idl/afidl1-grammar.md section 5.1",
            ],
        },
        {
            "suite_id": "SUITE-AFIDL-IMPORT-EXPLICIT-UNIVERSE",
            "description": (
                "Import resolution is a pure function of the source and an "
                "explicitly supplied in-memory universe: af-idl has no "
                "filesystem, network, registry or clock dependency at all, so a "
                "latest-version or network lookup is not merely discouraged but "
                "unrepresentable in the crate's dependency set."
            ),
            "targets": [
                "af-idl crate dependency set (af-core only)",
                "af_idl::ResolvedSchemaUniverse (explicit, immutable input)",
            ],
        },
        {
            "suite_id": "SUITE-AFIDL-IMPORT-IDENTITY-ISOLATION",
            "description": (
                "Import declarations never enter canonical identity: the "
                "canonical TypeSchema record keeps exactly its three members "
                "schema/name/body, and no import alias can appear in a "
                "NamedRef. Covers the frozen import equivalence groups "
                "(alias rename, import order, unused import) and the structural "
                "audit of the canonical model."
            ),
            "targets": [
                "spec/conformance/afidl/import-golden.json equivalence groups",
                "af-idl canonical TypeSchema record (three members)",
            ],
        },
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
        {
            "suite_id": "SUITE-SEMANTIC-CLOSED-EXPRESSION-SET",
            "description": (
                "The expression body set is closed to the fifteen Phase 3A "
                "expressions and to the workflow primitive universe. Covers "
                "the exhaustive match over ExpressionBody in the Rust checker "
                "and the closed SemanticPrimitiveType enum, whose ALL table "
                "and keyword round-trip are asserted in-tree."
            ),
            "targets": [
                "af-semantic SemanticPrimitiveType::ALL and try_from_keyword",
                "af-semantic expression body match exhaustiveness",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-NAMESPACE-SEPARATION",
            "description": (
                "ValueSlotId and ResourceSlotId are distinct nominal types that "
                "may carry the same number. Covers the Rust newtype separation "
                "test and the corpus case where a value slot 1 and a resource "
                "slot 1 coexist in one document."
            ),
            "targets": [
                "af-semantic ids namespace separation test",
                "spec/conformance/semantic-core/semantic-core-golden.json",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-NO-NEW-DOMAIN",
            "description": (
                "No new HashV1 domain is introduced in Phase 3A: the domain "
                "registry table still has exactly its frozen entries and the "
                "semantic layer computes no digest of its own."
            ),
            "targets": [
                "spec/tables/hash-domains.json",
                "af-semantic (no digest computation outside TypeSchemaDigest)",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-NO-RUNTIME-HANDLE",
            "description": (
                "No ResourceHandle, binding snapshot or runtime object exists "
                "in the semantic layer. Covers the static absence audit over "
                "the crate's public surface and the resource slot model."
            ),
            "targets": [
                "af-semantic resource slot model (logical requirement only)",
                "static self-check: no runtime object in af-semantic",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-PURITY",
            "description": (
                "Expressions are pure: no I/O, clock, randomness, environment, "
                "policy or registry query, resource access or effect. Covers "
                "the static purity audit over the closed expression body set "
                "and the absence of I/O dependencies in the crate."
            ),
            "targets": [
                "af-semantic (no I/O, clock, random or env dependency)",
                "static self-check: closed expression body set",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-RESOURCE-IMMUTABILITY",
            "description": (
                "A resource slot's definition is immutable for the document's "
                "life. Covered by the absence of any mutating operation in the "
                "resource model and by the checked document type being the only "
                "way to reach a resolved requirement."
            ),
            "targets": [
                "af-semantic ResourceSlot (no mutation surface)",
                "CheckedDocument construction path",
            ],
        },
    ],
})
