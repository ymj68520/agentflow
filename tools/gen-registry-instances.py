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
    imports = afidl.import_table(module)
    rows = []
    for declaration in module["declarations"]:
        schema = afidl.canonical_type(family, declaration, imports)
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
IR_SRC = "spec/semantic/semantic-ir.md"

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

IR_RULES = [
    rule("AF-SEMANTIC-0025", "SEMANTIC", IR_SRC, "ir-scope-static-only",
         "The Semantic IR is a static model: validation, canonicalization and "
         "the document digest only. No interpreter, plan, worker, region, "
         "endpoint, credential, lease or scheduler placement exists in this "
         "layer.",
         ["SUITE-SEMANTIC-IR-NO-RUNTIME"]),
    rule("AF-SEMANTIC-0026", "SEMANTIC", IR_SRC, "ir-no-runtime-semantics",
         "No opcode state machine, runtime command or journal event exists at "
         "the IR layer; runtime semantics belong to the later execution "
         "stages.",
         ["SUITE-SEMANTIC-IR-NO-RUNTIME"]),
    rule("AF-SEMANTIC-0027", "SEMANTIC", IR_SRC, "ir-occurrence-not-in-ir",
         "Runtime occurrence identity (OccurrencePath, AttemptId, "
         "InvocationId) does not exist in the IR: a loop or retry holds one "
         "static body definition, never per-occurrence copies.",
         ["SUITE-SEMANTIC-IR-NO-RUNTIME"]),
    rule("AF-SEMANTIC-0028", "SEMANTIC", IR_SRC, "ir-call-composite-forbidden",
         "CallComposite is authoring/compiler-only and MUST NOT appear in "
         "canonical IR; the construct is lowered before this layer.",
         ["SUITE-SEMANTIC-IR-AUTHORING-ONLY"]),
    rule("AF-SEMANTIC-0029", "SEMANTIC", IR_SRC, "ir-trigger-not-computation",
         "Trigger is not a computation: it belongs to deployment binding and "
         "has no opcode and no node in the IR.",
         ["SUITE-SEMANTIC-IR-AUTHORING-ONLY"]),
    rule("AF-SEMANTIC-0030", "SEMANTIC", IR_SRC, "ir-document-structure",
         "SemanticIrDocument carries the reused Phase 3A tables plus "
         "semantic_refs, contracts, run_parameters, workflow_outputs, "
         "computations and root, with the frozen field numbering.",
         ["invoke/minimal", "call-subflow/minimal"]),
    rule("AF-SEMANTIC-0031", "SEMANTIC", IR_SRC, "ir-semantic-refs",
         "A semantic ref pins an exact canonical SpecificationId: syntax is "
         "validated, references are unique and canonically ordered; registry "
         "knownness belongs to compiler tooling, never to af-ir.",
         ["call-subflow/minimal", "semantic-ref/invalid-syntax",
          "semantic-ref/duplicate"]),
    rule("AF-SEMANTIC-0032", "SEMANTIC", IR_SRC, "ir-reuse-core-tables",
         "The types/values/expressions/resources tables reuse "
         "agentflow.semantic.core@1 objects by identity via the AFIDL import; "
         "redeclaring copies is forbidden, and the tables must pass the Phase "
         "3A validation.",
         ["invoke/minimal", "nested/sequence-invoke-await"]),
    rule("AF-SEMANTIC-0033", "SEMANTIC", IR_SRC, "ir-canonical-order-table",
         "Canonical ordering is frozen for every table and nested list: "
         "identity tables by id, semantic_refs by canonical SpecificationId "
         "bytes, workflow_outputs by NFC name bytes, ports and ordinals "
         "ascending.",
         ["ordering/computations", "ordering/core-tables",
          "ordering/select-cases", "ordering/parallel-branches",
          "ordering/scope-resources", "ordering/run-parameters",
          "ordering/workflow-outputs", "ordering/semantic-refs",
          "ordering/handle-clauses", "ordering/carry",
          "ordering/parallel-output-bindings"]),
    rule("AF-SEMANTIC-0034", "SEMANTIC", IR_SRC, "ir-local-identities",
         "ComputationId, RunParameterId and ContractRefId are independent id "
         "namespaces and PortNo, BranchOrdinal and HandlerOrdinal are "
         "numbering spaces; all are u32 newtypes, values are > 0 and 0 is "
         "rejected by validation.",
         ["document/root-zero", "select/case-ordinal-zero",
          "producer/duplicate-output-port"]),
    rule("AF-SEMANTIC-0035", "SEMANTIC", IR_SRC, "ir-value-producer-uniqueness",
         "Every declared value slot has exactly one producer: 0 producers is "
         "unbound_value_slot and more than one is duplicate_value_producer. "
         "Producers are run parameters, computation outputs, iterate item "
         "slots and iterate carry slots.",
         ["producer/unbound-slot", "producer/param-and-computation-same-slot",
          "producer/two-computation-outputs",
          "producer/iterate-item-and-output"]),
    rule("AF-SEMANTIC-0036", "SEMANTIC", IR_SRC, "ir-expression-not-producer",
         "An expression is pure derived computation and is never a value-slot "
         "producer; expression ids do not materialize slots.",
         ["producer/unbound-slot",
          "SUITE-SEMANTIC-IR-DATAFLOW-OWNERSHIP-SEPARATION"]),
    rule("AF-SEMANTIC-0037", "SEMANTIC", IR_SRC, "ir-run-parameter-producer",
         "A run parameter produces its value slot: parameter.type must equal "
         "the slot's type and no other producer may bind the same slot.",
         ["run-parameter/type-mismatch", "run-parameter/unknown-type",
          "run-parameter/unknown-slot",
          "producer/param-and-computation-same-slot"]),
    rule("AF-SEMANTIC-0038", "SEMANTIC", IR_SRC, "ir-run-parameter-name",
         "A run parameter name is non-empty, NFC-normalized and unique after "
         "NFC; names may be any Unicode and canonical order is by "
         "RunParameterId, never by name.",
         ["run-parameter/name-empty", "run-parameter/name-not-nfc",
          "run-parameter/name-duplicate"]),
    rule("AF-SEMANTIC-0039", "SEMANTIC", IR_SRC, "ir-run-parameter-decl",
         "RunParameterDecl is a static workflow input declaration only: no "
         "default value, admission snapshot, mutable parameter, runtime "
         "update, constraint or secret flag exists in this layer.",
         ["run-parameter/type-mismatch", "invoke/minimal"]),
    rule("AF-SEMANTIC-0040", "SEMANTIC", IR_SRC, "ir-workflow-output",
         "A workflow output is a name plus an expression; the output type is "
         "the expression's result_type and is never duplicated. Output names "
         "follow the run-parameter name rules with canonical order by NFC "
         "name bytes.",
         ["workflow-output/name-duplicate", "workflow-output/name-empty",
          "workflow-output/unknown-expression"]),
    rule("AF-SEMANTIC-0041", "SEMANTIC", IR_SRC, "ir-contract-table-closed",
         "ContractRefKind is closed at exactly component (1) and subflow (2); "
         "entries are canonically ordered by ContractRefId and duplicate ids "
         "are rejected.",
         ["contract/duplicate-id", "contract/invoke-uses-subflow",
          "contract/call-subflow-uses-component"]),
    rule("AF-SEMANTIC-0042", "SEMANTIC", IR_SRC, "ir-component-contract-domain",
         "A component contract reference pins sha256 over the frozen "
         "component-contract domain; any other domain is rejected.",
         ["contract/component-wrong-domain"]),
    rule("AF-SEMANTIC-0043", "SEMANTIC", IR_SRC, "ir-subflow-contract-domain",
         "A subflow contract reference pins sha256 over the frozen "
         "semantic-ir domain: CALL_SUBFLOW references an exact published "
         "Semantic IR identity. No workflow-contract or subflow-contract "
         "domain exists.",
         ["contract/subflow-wrong-domain", "nested/select-retry-call-subflow"]),
    rule("AF-SEMANTIC-0044", "SEMANTIC", IR_SRC, "ir-contract-identity-unique",
         "One exact contract identity (algorithm, domain, hex) has at most "
         "one table entry per document; digest validation covers shape and "
         "domain only - there is no Contract Store in this layer.",
         ["contract/duplicate-identity"]),
    rule("AF-SEMANTIC-0045", "SEMANTIC", IR_SRC, "ir-computation-outputs",
         "A computation's output ports are positive and unique, bound to "
         "declared value slots; the port's type is the slot's type and is "
         "never stored twice.",
         ["producer/duplicate-output-port", "output/unknown-slot"]),
    rule("AF-SEMANTIC-0046", "SEMANTIC", IR_SRC, "ir-child-table",
         "The ownership edge set is exactly the per-opcode child table "
         "(INVOKE/AWAIT/CALL_SUBFLOW: 0 local children; SEQUENCE: N; SELECT: "
         "N cases; PARALLEL/RACE: N branches; ITERATE/RETRY/TIMEOUT/SCOPE/"
         "COMPENSATION_SCOPE: 1; HANDLE: 1 protected child + N handlers); "
         "every child reference must resolve.",
         ["document/child-missing", "handle/unknown-handler",
          "retry/child-missing"]),
    rule("AF-SEMANTIC-0047", "SEMANTIC", IR_SRC, "ir-ownership-tree",
         "The computations form a structured ownership tree: the root exists "
         "and has no parent, every non-root computation has exactly one "
         "parent, nothing is detached, and neither cycles nor unreachable "
         "computations exist.",
         ["document/structural-cycle", "document/multiple-structural-parents",
          "document/unknown-root", "document/unreachable-computation",
          "document/duplicate-child-reference", "document/root-zero"]),
    rule("AF-SEMANTIC-0048", "SEMANTIC", IR_SRC,
         "ir-dataflow-ownership-separation",
         "Dataflow edges (expression reads a value slot) and ownership edges "
         "(a computation owns a child) are different relations handled by "
         "different validator logic; no universal Edge type exists.",
         ["SUITE-SEMANTIC-IR-DATAFLOW-OWNERSHIP-SEPARATION"]),
    rule("AF-SEMANTIC-0049", "SEMANTIC", IR_SRC, "ir-interface-projection",
         "ComputationInterface is a read-only projection port_no -> "
         "SemanticTypeId derived from outputs and slot types; no second "
         "interface table may be stored anywhere.",
         ["SUITE-SEMANTIC-IR-INTERFACE-PROJECTION"]),
    rule("AF-SEMANTIC-0050", "SEMANTIC", IR_SRC, "ir-wrapper-interfaces",
         "Wrapper interface equality is static: SEQUENCE equals its last "
         "child, SELECT every case, RACE every branch, RETRY/TIMEOUT/SCOPE/"
         "COMPENSATION_SCOPE the child, HANDLE the protected child plus every "
         "handler; PARALLEL is checked by its explicit output mapping and "
         "ITERATE by the carry rules.",
         ["sequence/parent-interface-mismatch",
          "select/branch-interface-mismatch", "race/branch-interface-mismatch",
          "retry/interface-mismatch", "timeout/interface-mismatch",
          "handle/handler-interface-mismatch",
          "handle/protected-child-interface-mismatch",
          "scope/interface-mismatch", "compensation-scope/interface-mismatch",
          "parallel/output-type-mismatch"]),
    rule("AF-SEMANTIC-0051", "SEMANTIC", IR_SRC, "ir-leaf-interfaces",
         "INVOKE, AWAIT and CALL_SUBFLOW are leaves: their output interface "
         "is declared only by their own outputs; contract-level checking is "
         "deferred.",
         ["invoke/minimal", "await/minimal", "call-subflow/minimal"]),
    rule("AF-SEMANTIC-0052", "SEMANTIC", IR_SRC, "ir-sequence-order",
         "The SEQUENCE child list order is the semantic execution order: it "
         "is preserved verbatim and never canonically re-sorted; a sequence "
         "has at least one child and its output interface equals the last "
         "child's.",
         ["mutation/sequence-child-order", "sequence/empty"]),
    rule("AF-SEMANTIC-0053", "SEMANTIC", IR_SRC, "ir-select-ordinal",
         "The SELECT selector has type int64 and produces the exact case "
         "ordinal to run: no truthiness, no source-order matching, no "
         "fall-through. Ordinals are positive, unique and canonically "
         "ascending; a select has at least one case.",
         ["select/empty", "select/selector-not-int64",
          "select/case-ordinal-zero", "select/duplicate-case-ordinal"]),
    rule("AF-SEMANTIC-0054", "SEMANTIC", IR_SRC, "ir-select-interfaces",
         "All SELECT case children share the parent's output interface: the "
         "typed merge shell is checked statically; runtime case selection is "
         "not part of this phase.",
         ["select/branch-interface-mismatch"]),
    rule("AF-SEMANTIC-0055", "SEMANTIC", IR_SRC, "ir-parallel-structure",
         "ParallelMode is closed at all (1) and fail_fast (2); a parallel has "
         "at least one branch with positive unique ordinals in canonical "
         "ascending order.",
         ["parallel/empty-branches", "parallel/duplicate-ordinal",
          "parallel/all", "parallel/fail-fast"]),
    rule("AF-SEMANTIC-0056", "SEMANTIC", IR_SRC, "ir-parallel-output-mapping",
         "The explicit parallel output mapping is the only path from branch "
         "outputs into the parent interface: parent ports are unique across "
         "bindings, branch and child ports must exist, and the parent slot's "
         "type equals the child output slot's type.",
         ["parallel/output-unknown-branch",
          "parallel/output-unknown-child-port",
          "parallel/output-type-mismatch", "parallel/fail-fast"]),
    rule("AF-SEMANTIC-0057", "SEMANTIC", IR_SRC, "ir-parallel-max-inflight",
         "max_inflight is a semantic concurrency bound: option<uint32> with "
         "Some(0) invalid. Worker count, tenant quota, region capacity and "
         "scheduler limits are physical concerns and MUST NOT enter the IR.",
         ["parallel/max-inflight-zero", "foreach/max-inflight-zero",
          "SUITE-SEMANTIC-IR-NO-RUNTIME"]),
    rule("AF-SEMANTIC-0058", "SEMANTIC", IR_SRC, "ir-race-structure",
         "A race has at least two branches with positive unique ordinals and "
         "one shared output interface; v1 semantics is exactly FIRST_TERMINAL "
         "with no winner_policy knob.",
         ["race/one-branch", "race/duplicate-ordinal",
          "race/branch-interface-mismatch", "race/minimal"]),
    rule("AF-SEMANTIC-0059", "SEMANTIC", IR_SRC, "ir-iterate-foreach",
         "FOREACH: the source expression has type list<T>, the item slot "
         "exists with type T and its producer is this ITERATE; max_inflight "
         "follows the parallel bound rules.",
         ["foreach/source-not-list", "foreach/item-slot-wrong-type",
          "foreach/item-slot-missing", "foreach/max-inflight-zero",
          "producer/iterate-item-and-output", "iterate/foreach"]),
    rule("AF-SEMANTIC-0060", "SEMANTIC", IR_SRC, "ir-iterate-while",
         "WHILE: the guard has type bool and max_iterations > 0 - v1 has no "
         "unbounded while. The guard may reference carry slots; availability "
         "and dominance analysis is Phase 3C.",
         ["while/guard-not-bool", "while/max-iterations-zero",
          "iterate/while"]),
    rule("AF-SEMANTIC-0061", "SEMANTIC", IR_SRC, "ir-loop-carry",
         "A loop carry binding requires the slot to exist, the initial "
         "expression to exist, the body output port to exist, and slot.type "
         "== initial.result_type == body-output-slot.type; one binding per "
         "carry slot. Loops never mutate: values are new occurrences.",
         ["carry/slot-missing", "carry/initial-type-mismatch",
          "carry/body-output-missing", "carry/next-type-mismatch",
          "carry/duplicate-slot", "iterate/while"]),
    rule("AF-SEMANTIC-0062", "SEMANTIC", IR_SRC, "ir-retry-schedule",
         "RETRY carries an explicit finite delay schedule: maximum occurrence "
         "count is exactly 1 + len(delays); delays are durations >= 0 and "
         "their order is semantic (never sorted); no wall clock, jitter or "
         "runtime policy lookup exists.",
         ["mutation/retry-delay-order", "retry/child-missing",
          "retry/minimal"]),
    rule("AF-SEMANTIC-0063", "SEMANTIC", IR_SRC, "ir-timeout-expression",
         "TIMEOUT bounds one child by a duration-valued expression; there is "
         "no TimerId, TimerState or TimerCommand. AWAIT has no built-in "
         "timeout: compose TIMEOUT(AWAIT(...)).",
         ["timeout/expression-not-duration", "timeout/interface-mismatch",
          "timeout/minimal"]),
    rule("AF-SEMANTIC-0064", "SEMANTIC", IR_SRC, "ir-handle-match",
         "HandleMatch is closed at failure (exact typed TypeSchemaRef match) "
         "and timed_out (unit); there is no cancelled case. Handler ordinals "
         "are positive and unique; failure clauses are pairwise disjoint by "
         "exact schema; at most one timed_out clause exists.",
         ["handle/duplicate-handler-ordinal",
          "handle/duplicate-failure-matcher", "handle/two-timed-out",
          "handle/cancelled-matcher", "handle/failure", "handle/timed-out"]),
    rule("AF-SEMANTIC-0065", "SEMANTIC", IR_SRC, "ir-scope-resources",
         "A scope lists the logical resources it statically owns/allows: they "
         "must exist, be unique, and are canonically ascending. Only the "
         "ownership and lexical boundaries are frozen - authority/budget "
         "attenuation, deadline and cancellation state and ResourceHandle "
         "are later, separately specified tables.",
         ["scope/unknown-resource", "scope/duplicate-resource",
          "scope/minimal"]),
    rule("AF-SEMANTIC-0066", "SEMANTIC", IR_SRC, "ir-compensation-scope",
         "COMPENSATION_SCOPE owns one child and holds no compensation task "
         "list; compensation sources are later derived from committed effect "
         "receipts plus effect compensation contracts, and a "
         "COMPENSATION_SCOPE MUST NOT erase forward computation or effect "
         "history.",
         ["compensation-scope/interface-mismatch",
          "compensation-scope/minimal",
          "SUITE-SEMANTIC-IR-COMPENSATION-HISTORY"]),
    rule("AF-SEMANTIC-0067", "SEMANTIC", IR_SRC, "ir-digest-formula",
         "SemanticIrDigest = HashV1(\"semantic-ir\", "
         "ACE1(CanonicalSemanticIrDocument)); the domain is the Phase 1 "
         "frozen registry entry, no new domain is introduced, and the digest "
         "is computed over the canonical form only.",
         ["invoke/minimal", "ordering/computations", "ordering/core-tables"]),
    rule("AF-SEMANTIC-0068", "SEMANTIC", IR_SRC, "ir-invoke-static",
         "INVOKE calls an exact component contract with semantic inputs and "
         "resource requirements; input and resource ports are positive and "
         "unique and their expressions/resources must exist. No "
         "TaskDefinition, Binding, Invocation, Attempt or Worker exists at "
         "this layer.",
         ["contract/unknown-id", "contract/invoke-uses-subflow",
          "invoke/minimal", "nested/sequence-invoke-await"]),
    rule("AF-SEMANTIC-0069", "SEMANTIC", IR_SRC, "ir-await-static",
         "AWAIT is a passive durable wait with an exact digest-pinned "
         "signal_type and a correlation expression; no mailbox, timer, "
         "webhook or approval state exists in this layer.",
         ["await/minimal", "nested/sequence-invoke-await"]),
    rule("AF-SEMANTIC-0070", "SEMANTIC", IR_SRC, "ir-call-subflow",
         "CALL_SUBFLOW references a subflow-kind contract (an exact "
         "published Semantic IR digest), has zero local children and no "
         "detach/fire-and-forget: the subflow is an owned durable child "
         "boundary.",
         ["call-subflow/wrong-contract-kind",
          "call-subflow/duplicate-input-port",
          "call-subflow/duplicate-resource-port",
          "call-subflow/unknown-expression", "call-subflow/unknown-resource",
          "call-subflow/minimal"]),
    rule("AF-SEMANTIC-0071", "SEMANTIC", IR_SRC, "ir-semantic-order-exceptions",
         "Exactly two lists carry semantic order and are never canonically "
         "re-sorted: SEQUENCE children (execution order) and RETRY delays "
         "(schedule order); changing their order changes the "
         "SemanticIrDigest.",
         ["mutation/sequence-child-order", "mutation/retry-delay-order",
          "mutation/parallel-mode"]),
    # --- Phase 3C: scoped typed dataflow / dominance / availability -------
    rule("AF-SEMANTIC-0072", "SEMANTIC", IR_SRC, "ir-value-availability",
         "A value slot may be read at a use site only if its single producer "
         "dominates that use site in the structured ownership tree: slot "
         "existence (Phase 3B) is not availability, and the analysis adds "
         "value_not_available without touching any Phase 3B error kind.",
         ["sequence/self-output-used-as-input", "sequence/future-sibling-output",
          "sequence/parent-output-used-by-child",
          "SUITE-SEMANTIC-DATAFLOW-AVAILABILITY"]),
    rule("AF-SEMANTIC-0073", "SEMANTIC", IR_SRC, "ir-run-parameter-availability",
         "Run parameters are available from the root entry, in every "
         "descendant computation under every control branch, for the whole "
         "run: they are the immutable workflow inputs.",
         ["value/run-parameter-deep-in-nested", "workflow-output/run-parameter"]),
    rule("AF-SEMANTIC-0074", "SEMANTIC", IR_SRC, "ir-output-boundary",
         "A computation's outputs become available only after that "
         "computation's structured completion boundary: an output MUST NOT "
         "be used at its own entry, a parent output MUST NOT be used inside "
         "its own subtree, and a timeout duration is evaluated at the "
         "timeout entry.",
         ["sequence/self-output-used-as-input",
          "sequence/parent-output-used-by-child",
          "timeout/duration-uses-child-output", "while/guard-uses-body-output"]),
    rule("AF-SEMANTIC-0075", "SEMANTIC", IR_SRC, "ir-export-boundary",
         "Only a computation's own declared outputs cross its ownership "
         "boundary: a raw child, grandchild, case, branch, handler or "
         "loop-body output consumed outside its owner is rejected as "
         "outside_owner_boundary.",
         ["sequence/grandchild-output-leaks-to-later-sibling",
          "select/raw-case-output-used-after-select",
          "parallel/raw-branch-output-used-after-parallel",
          "race/raw-loser-or-winner-output-used-after-race",
          "retry/raw-child-output-used-after-retry",
          "timeout/raw-child-output-used-after-timeout",
          "scope/raw-child-output-used-after-scope",
          "comp-scope/raw-child-output-used-after-comp-scope",
          "handle/raw-handler-output-used-after-handle",
          "sequence/nested-export-via-own-output"]),
    rule("AF-SEMANTIC-0076", "SEMANTIC", IR_SRC, "ir-sequence-dominance",
         "SEQUENCE is the only multi-child control form whose child order is "
         "dataflow: each child's entry environment is the sequence entry "
         "plus every earlier direct sibling's own exports, cumulatively.",
         ["sequence/previous-sibling-output",
          "sequence/nested-export-via-own-output"]),
    rule("AF-SEMANTIC-0077", "SEMANTIC", IR_SRC, "ir-sequence-future",
         "A SEQUENCE child MUST NOT read a later sibling's export: future "
         "reference is use-before-producer and is rejected.",
         ["sequence/future-sibling-output", "sequence/self-output-used-as-input"]),
    rule("AF-SEMANTIC-0078", "SEMANTIC", IR_SRC, "ir-select-isolation",
         "SELECT cases are mutually isolated and start from the select's "
         "entry environment: the selector evaluates before any case, case "
         "ordinals are identity keys (never execution order), and one case "
         "MUST NOT read another case's output.",
         ["select/case-cross-reference", "select/selector-uses-case-output"]),
    rule("AF-SEMANTIC-0079", "SEMANTIC", IR_SRC, "ir-parallel-isolation",
         "PARALLEL branches are mutually isolated: every branch starts from "
         "the parallel's entry environment and no branch may consume another "
         "branch's output, in any completion mode.",
         ["parallel/cross-branch-value"]),
    rule("AF-SEMANTIC-0080", "SEMANTIC", IR_SRC, "ir-race-isolation",
         "RACE branches are mutually isolated: FIRST_TERMINAL gives no "
         "stable branch order, so no branch may consume another branch's "
         "output and no raw branch output (winner or loser) escapes the "
         "race.",
         ["race/cross-branch-value",
          "race/raw-loser-or-winner-output-used-after-race"]),
    rule("AF-SEMANTIC-0081", "SEMANTIC", IR_SRC, "ir-handle-isolation",
         "HANDLE clauses are mutually isolated and see exactly the handle's "
         "entry environment: a handler MUST NOT read the protected child's "
         "outputs (it runs precisely when those outputs may not exist) and "
         "no FailureValue or failure-slot placeholder is created.",
         ["handle/handler-uses-protected-child-output",
          "handle/handler-cross-reference"]),
    rule("AF-SEMANTIC-0082", "SEMANTIC", IR_SRC, "ir-iterate-source-pre-loop",
         "A foreach source is a pre-loop snapshot expression: it evaluates "
         "once from the iterate's entry environment and MUST NOT reference "
         "the item slot, a carry slot or a body output.",
         ["foreach/source-uses-item-slot", "foreach/source-uses-carry-slot"]),
    rule("AF-SEMANTIC-0083", "SEMANTIC", IR_SRC, "ir-carry-initial-pre-loop",
         "Every loop-carry initial evaluates from the same pre-loop "
         "environment: no initial may read the item slot or any carry slot, "
         "so the canonical carry order (by slot id) never becomes an "
         "evaluation order.",
         ["foreach/carry-initial-uses-item-slot",
          "foreach/carry-initial-uses-carry-slot",
          "while/carry-initial-uses-carry-slot"]),
    rule("AF-SEMANTIC-0084", "SEMANTIC", IR_SRC, "ir-foreach-body-environment",
         "A foreach body sees the iterate's entry environment plus the item "
         "slot and every carry slot.",
         ["foreach/source-and-body-item", "foreach/body-carry"]),
    rule("AF-SEMANTIC-0085", "SEMANTIC", IR_SRC, "ir-while-environment",
         "A while guard and body see the iterate's entry environment plus "
         "every carry slot; the guard evaluates at the loop header and "
         "never reads a body computation output directly.",
         ["while/guard-carry", "while/body-carry", "while/guard-uses-body-output"]),
    rule("AF-SEMANTIC-0086", "SEMANTIC", IR_SRC, "ir-iterate-local-escape",
         "Loop-local values cannot escape their iterate: a foreach item "
         "slot, a carry slot and a loop-body output MUST NOT be read "
         "outside the iterate computation; outside it only "
         "Iterate.outputs are visible.",
         ["foreach/item-escapes-loop", "foreach/carry-escapes-loop",
          "foreach/body-output-escapes-loop", "while/carry-escapes-loop",
          "while/body-output-escapes-loop", "workflow-output/iterate-local"]),
    rule("AF-SEMANTIC-0087", "SEMANTIC", IR_SRC, "ir-workflow-output-boundary",
         "Only the root computation's own outputs and the immutable run "
         "parameters are visible to workflow-output expressions: "
         "WorkflowExitValues = RunParameterSlots + Exports(root); a raw "
         "root-descendant output is rejected.",
         ["workflow-output/raw-root-descendant-output",
          "workflow-output/select-case-output", "workflow-output/root-output"]),
    rule("AF-SEMANTIC-0088", "SEMANTIC", IR_SRC, "ir-per-use-site",
         "Expression availability is checked per use site and never cached "
         "per expression: the same ExpressionDef may be legal inside its "
         "owning branch and illegal at a use site outside it.",
         ["expression/reuse-per-use-site", "expression/inside-use-valid"]),
    rule("AF-SEMANTIC-0089", "SEMANTIC", IR_SRC, "ir-unused-expression",
         "An expression referenced by no use site creates no execution and "
         "never fails availability: an expression definition is not an "
         "execution.",
         ["expression/unused-branch-local-reference"]),
    rule("AF-SEMANTIC-0090", "SEMANTIC", IR_SRC, "ir-resource-attenuation",
         "A SCOPE narrows resource visibility: its listed set MUST be a "
         "subset of the inherited environment and its child sees exactly "
         "the listed set (never the union); an omitted resource is "
         "resource_not_visible inside the scope.",
         ["resource/scope-child-uses-listed", "resource/scope-child-uses-omitted"]),
    rule("AF-SEMANTIC-0091", "SEMANTIC", IR_SRC, "ir-resource-no-widen",
         "A nested SCOPE cannot widen resources: it cannot reintroduce a "
         "resource its parent scope omitted.",
         ["resource/nested-scope-narrows", "resource/nested-scope-reintroduces"]),
    rule("AF-SEMANTIC-0092", "SEMANTIC", IR_SRC, "ir-resource-use-sites",
         "InvokeBody.resources and CallSubflowBody.resources are the "
         "resource use sites: every used slot MUST be lexically visible at "
         "the computation's entry environment.",
         ["resource/root-invoke-uses-resource",
          "resource/call-subflow-uses-invisible"]),
    rule("AF-SEMANTIC-0093", "SEMANTIC", IR_SRC, "ir-resource-visibility",
         "EntryResources(root) is every declared resource slot; every "
         "opcode but SCOPE inherits its parent's environment unchanged, "
         "narrowing ends at the scope boundary, and visibility is shared "
         "not owned: parallel branches may use the same visible resource "
         "and no resource_shared_in_parallel error exists.",
         ["resource/scope-exit-restores-sibling",
          "resource/parallel-branches-share-visible"]),
    rule("AF-SEMANTIC-0094", "SEMANTIC", IR_SRC, "ir-analysis-derived",
         "The Phase 3C analysis is derived-only: SemanticDataflowAnalysis "
         "is never ACE1-encoded, never hashed into the SemanticIrDigest, "
         "and absent from AFIDL, the ABI registry and the wire model; it "
         "runs strictly after Phase 3B validation so old error kinds never "
         "move.",
         ["SUITE-SEMANTIC-DATAFLOW-DERIVED-ONLY",
          "SUITE-SEMANTIC-DATAFLOW-PASS-ORDERING"]),
    rule("AF-SEMANTIC-0095", "SEMANTIC", IR_SRC, "ir-wire-freeze",
         "Phase 3C freezes the durable surfaces: semantic-ir.afidl stays "
         "byte-identical (36 TypeSchemaDigests), the 13 ComputationBody "
         "cases stay frozen, and no AFIDL module, ABI row, HashV1 domain "
         "or Rule Area is added.",
         ["SUITE-SEMANTIC-DATAFLOW-WIRE-FREEZE"]),
    # --- Phase 3D: subflow interface derivation / CALL_SUBFLOW checking ---
    rule("AF-SEMANTIC-0096", "SEMANTIC", IR_SRC, "ir-subflow-exact-target",
         "A CALL_SUBFLOW target resolves only by its exact pinned "
         "SemanticIrDigest in an explicit, immutable, in-memory universe "
         "whose keys are full DigestRefs (sha256 / semantic-ir / hex); the "
         "digest is always computed from the target document itself.",
         ["subflow/unknown-target-digest", "subflow/stale-target-digest",
          "SUITE-SUBFLOW-EXACT-TARGET"]),
    rule("AF-SEMANTIC-0097", "SEMANTIC", IR_SRC, "ir-subflow-no-fallback",
         "A missing target digest fails closed: there is no same-name "
         "latest fallback, no filesystem, registry, network, package or "
         "deployment lookup anywhere in the subflow universe.",
         ["subflow/universe-does-not-fallback-latest"]),
    rule("AF-SEMANTIC-0098", "SEMANTIC", IR_SRC, "ir-subflow-interface-derived",
         "The subflow interface is a derived static view of the target's "
         "own run_parameters, workflow_outputs and resources tables: it "
         "never enters AFIDL, the ABI registry or the SemanticIrDigest, "
         "and no SemanticTypeDigest or interface digest exists.",
         ["SUITE-SUBFLOW-INTERFACE-DERIVATION"]),
    rule("AF-SEMANTIC-0099", "SEMANTIC", IR_SRC, "ir-subflow-input-ports",
         "Input ports enumerate the target run parameters in "
         "RunParameterId canonical order, 1-based; source table order "
         "never matters.",
         ["subflow/multiple-inputs-canonical-port-order"]),
    rule("AF-SEMANTIC-0100", "SEMANTIC", IR_SRC, "ir-subflow-output-ports",
         "Output ports enumerate the target workflow outputs in canonical "
         "NFC-name byte order, 1-based; source table order never matters.",
         ["subflow/multiple-outputs-canonical-port-order"]),
    rule("AF-SEMANTIC-0101", "SEMANTIC", IR_SRC, "ir-subflow-resource-ports",
         "Every declared target ResourceSlot is part of the resource "
         "requirement interface (not only the currently used ones); "
         "resource ports enumerate the slots in ResourceSlotId canonical "
         "order, 1-based.",
         ["subflow/resources", "resource/missing-port", "resource/extra-port"]),
    rule("AF-SEMANTIC-0102", "SEMANTIC", IR_SRC, "ir-subflow-port-ordinals",
         "Subflow interface port ordinals are derived external interface "
         "coordinates, not aliases for RunParameterId, workflow-output "
         "names or ResourceSlotId; the three port namespaces are "
         "independent and target names are diagnostic metadata only.",
         ["equivalence/port-order-canonical-keys"]),
    rule("AF-SEMANTIC-0103", "SEMANTIC", IR_SRC, "ir-cross-document-type-equivalence",
         "Cross-document types compare structurally, never by local "
         "SemanticTypeId: primitives compare by keyword and containers "
         "recursively, so the same structure under different local ids is "
         "compatible and the same id with different meanings is not.",
         ["subflow/caller-and-target-use-different-type-table-numbering",
          "input/type-mismatch", "SUITE-SUBFLOW-CROSS-DOCUMENT-TYPES"]),
    rule("AF-SEMANTIC-0104", "SEMANTIC", IR_SRC, "ir-schema-leaf-exact-equality",
         "A schema leaf compares the exact complete TypeSchemaRef - "
         "specification_id, type_name and schema_digest; the same record "
         "name under a different specification version is a different "
         "type.",
         ["input/schema-digest-mismatch", "resource/schema-digest-mismatch"]),
    rule("AF-SEMANTIC-0105", "SEMANTIC", IR_SRC, "ir-subflow-inputs-exact",
         "CALL_SUBFLOW inputs must exactly match the target input "
         "interface: the bound port set equals 1..N (no missing, no extra "
         "port) and every bound expression's resolved type equals the "
         "target port type.",
         ["input/missing-port", "input/extra-port", "input/type-mismatch",
          "input/nested-type-mismatch", "subflow/basic-input-output"]),
    rule("AF-SEMANTIC-0106", "SEMANTIC", IR_SRC, "ir-subflow-outputs-exact",
         "CALL_SUBFLOW outputs must exactly match the target output "
         "interface: the declared port set equals 1..N and every declared "
         "slot's resolved type equals the target port type; an unconsumed "
         "output stays an unused slot, never a partial contract.",
         ["output/missing-port", "output/extra-port", "output/type-mismatch",
          "output/nested-type-mismatch"]),
    rule("AF-SEMANTIC-0107", "SEMANTIC", IR_SRC, "ir-subflow-resources-exact",
         "CALL_SUBFLOW resources must exactly match the target resource "
         "interface: the bound port set equals 1..N and every bound "
         "slot's requirement_type equals the target requirement_type as "
         "an exact TypeSchemaRef.",
         ["resource/type-mismatch", "subflow/resources"]),
    rule("AF-SEMANTIC-0108", "SEMANTIC", IR_SRC, "ir-subflow-interface-immutable",
         "A fixed SemanticIrDigest implies a fixed derived interface: "
         "table order moves neither the digest nor the interface, local "
         "type renumbering moves neither the interface shape, and a "
         "changed target always means a new digest that old callers never "
         "follow silently.",
         ["equivalence/target-table-order", "equivalence/local-type-ids-differ",
          "subflow/same-target-called-twice"]),
    rule("AF-SEMANTIC-0109", "SEMANTIC", IR_SRC, "ir-subflow-pass-order",
         "Subflow checking runs strictly after Phase 3B validation and the "
         "Phase 3C analysis, never changes their error kinds, and a "
         "document without CALL_SUBFLOW passes against an empty universe.",
         ["subflow/document-with-no-call-subflow-and-empty-universe",
          "SUITE-SUBFLOW-INTERFACE-PASS-ORDER"]),
    rule("AF-SEMANTIC-0110", "SEMANTIC", IR_SRC, "ir-subflow-no-transitive-closure",
         "Interface derivation does not validate the target's transitive "
         "subflow closure, and subflow-graph cycles are not judged here: "
         "dependency closure and graph acyclicity belong to the later "
         "Resolver / LockGraph stage.",
         ["SUITE-SUBFLOW-EXACT-TARGET"]),
]
RULES.extend(IR_RULES)


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
    ("agentflow.semantic.ir@1", "schema",
     "spec/idl/semantic-ir.afidl",
     "Normative shape of the canonical Semantic IR: the closed 13-opcode "
     "computation set, structured ownership, run parameters, workflow "
     "outputs, the closed contract table and the SemanticIrDocument. The "
     "first production consumer of the AFIDL1 cross-module import."),
    ("agentflow.semantic.ir.semantics@1", "schema_language",
     "spec/semantic/semantic-ir.md",
     "Canonical Semantic IR semantics: static scope, ownership tree, value "
     "producer rules, interface equality, per-opcode static schemas, "
     "canonical ordering and the SemanticIrDigest."),
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
    "spec/idl/semantic-ir.afidl",
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

SUITE_LIST = [
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
            "suite_id": "SUITE-SEMANTIC-IR-AUTHORING-ONLY",
            "description": (
                "CallComposite and Trigger never reach the canonical IR: the "
                "13-case ComputationBody variant is closed, so an authoring "
                "construct or a deployment binding is structurally "
                "inexpressible. Covers the static absence audit over the "
                "crate source and the schema module."
            ),
            "targets": [
                "af-ir static surface audit (no CallComposite/Trigger)",
                "spec/idl/semantic-ir.afidl (closed 13-case variant)",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-IR-COMPENSATION-HISTORY",
            "description": (
                "COMPENSATION_SCOPE holds no compensation task list and MUST "
                "NOT erase forward computation or effect history: the IR body "
                "is a single child reference, so there is nothing to erase "
                "with. Covers the model shape and the static audit."
            ),
            "targets": [
                "af-ir CompensationScopeBody (one child, no list)",
                "af-ir static surface audit",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-IR-AWAIT-COMPOSITION",
            "description": (
                "AWAIT has no built-in timeout: the AwaitBody schema carries "
                "only signal_type and correlation, so timing is expressed by "
                "composing TIMEOUT(AWAIT(...)). Covers the schema shape and "
                "the static audit."
            ),
            "targets": [
                "af-ir AwaitBody (no timeout field)",
                "spec/idl/semantic-ir.afidl",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-IR-DATAFLOW-OWNERSHIP-SEPARATION",
            "description": (
                "Dataflow (expression reads a value slot) and ownership (a "
                "computation owns a child) are separate validator logics over "
                "separate edge sets: ComputationBody::children walks only "
                "child references, and expression checking stays in the "
                "semantic core. Covers the two code paths and their tests."
            ),
            "targets": [
                "af-ir ComputationBody::children / owned_children",
                "af-semantic expression checking (dataflow)",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-IR-INTERFACE-PROJECTION",
            "description": (
                "ComputationInterface is derived on demand from outputs and "
                "slot types; no second interface table is stored anywhere in "
                "the model or the corpus. Covers the static audit and the "
                "interface equality negatives."
            ),
            "targets": [
                "af-ir CheckedIrDocument::interface",
                "af-ir static surface audit (no interface table)",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-IR-NO-RUNTIME",
            "description": (
                "No runtime object exists in the IR crate: no opcode state "
                "machine, no runtime command, no journal event, no occurrence "
                "identity, no worker/scheduler/reducer, no resource handle, "
                "no clock, no randomness, no network, no af-spec dependency. "
                "Covers the static absence audit over the dependency set and "
                "the source surface."
            ),
            "targets": [
                "af-ir static surface audit",
                "af-ir crate dependency set",
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
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-AVAILABILITY",
            "description": (
                "Value availability is per use site and dominance is the "
                "structured transfer: a producer dominates a use exactly "
                "when the opcode-specific environment places the produced "
                "slot into that use site's EntryValues. Covers the frozen "
                "positive/negative dataflow corpus and the per-use-site "
                "expression reuse fixture."
            ),
            "targets": [
                "af-ir CheckedIrDocument::analyze_dataflow",
                "spec/conformance/semantic-dataflow/",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-BRANCH-ISOLATION",
            "description": (
                "SELECT cases, PARALLEL branches, RACE branches and HANDLE "
                "clauses are mutually isolated: ordinals are identity keys, "
                "never execution order, and a raw child output never "
                "escapes its wrapper. Covers the branch-isolation negative "
                "corpus."
            ),
            "targets": [
                "af-ir dataflow visit (branch transfer functions)",
                "spec/conformance/semantic-dataflow/negative/",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-DERIVED-ONLY",
            "description": (
                "The analysis artifact is derived-only: it is a plain Rust "
                "projection, never ACE1-encoded, never hashed into the "
                "SemanticIrDigest, and absent from AFIDL, the ABI registry "
                "and the wire model. Covers the static audit of the "
                "canonical path and the golden digest invariance run."
            ),
            "targets": [
                "af-ir SemanticDataflowAnalysis (no serde-wire surface)",
                "af-ir canonical.rs (no analysis reference)",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-ITERATE-LOCALS",
            "description": (
                "Loop-local values never escape their iterate: the foreach "
                "source and carry initials evaluate from the pre-loop "
                "environment, the body sees item and carry slots, the "
                "while guard never reads a body output, and item/carry/"
                "body-output slots are invisible outside the loop. Covers "
                "the iterate negative corpus."
            ),
            "targets": [
                "af-ir dataflow visit (loop transfer functions)",
                "spec/conformance/semantic-dataflow/negative/",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-PASS-ORDERING",
            "description": (
                "The analysis runs strictly after Phase 3B validation: "
                "every Phase 3B negative keeps its exact expected error "
                "kind, and every Phase 3C negative passes Phase 3B and is "
                "rejected only by the analysis. Covers both frozen "
                "negative corpora and their rejected_at markers."
            ),
            "targets": [
                "spec/conformance/semantic-ir/negative/ (kinds unchanged)",
                "spec/conformance/semantic-dataflow/negative/",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-RESOURCE-VISIBILITY",
            "description": (
                "Resource lexical visibility: the root sees every declared "
                "slot, SCOPE attenuates to exactly its listed subset, "
                "narrowing ends at the scope boundary, and invoke/"
                "call_subflow uses require lexical visibility. Covers the "
                "resource positive and negative corpus."
            ),
            "targets": [
                "af-ir dataflow visit (resource environments)",
                "spec/conformance/semantic-dataflow/",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-WIRE-FREEZE",
            "description": (
                "Phase 3C froze every durable surface: semantic-ir.afidl "
                "byte-identical at its Phase 3B registry digest, exactly "
                "four AFIDL modules, 232 ABI rows (100 semantic IR), 14 "
                "HashV1 domains, and the 13 ComputationBody cases "
                "unchanged. Covers the static surface audit."
            ),
            "targets": [
                "af-ir static surface audit (wire freeze)",
                "spec/registry/specifications.json content_digest pin",
            ],
        },
        {
            "suite_id": "SUITE-SEMANTIC-DATAFLOW-WORKFLOW-EXIT",
            "description": (
                "Workflow outputs read only run parameters and the root's "
                "own outputs: WorkflowExitValues = RunParameterSlots + "
                "Exports(root), frozen per positive case."
            ),
            "targets": [
                "af-ir dataflow workflow-exit check",
                "spec/conformance/semantic-dataflow/positive/",
            ],
        },
        {
            "suite_id": "SUITE-SUBFLOW-CROSS-DOCUMENT-TYPES",
            "description": (
                "Cross-document types compare structurally: primitives by "
                "keyword, schema leaves by exact TypeSchemaRef (identity "
                "and digest), containers recursively; local SemanticTypeIds "
                "never compare numerically. Covers the frozen corpus cases "
                "for renumbered type tables and same-id different-type "
                "rejections."
            ),
            "targets": [
                "af-ir subflow type expansion (StructuralType)",
                "spec/conformance/subflow-interface/",
            ],
        },
        {
            "suite_id": "SUITE-SUBFLOW-EXACT-TARGET",
            "description": (
                "CALL_SUBFLOW resolves its target only by the exact pinned "
                "SemanticIrDigest in an explicit in-memory universe: fail "
                "closed on a missing digest, no latest fallback, no "
                "filesystem/network/deployment lookup, idempotent "
                "re-insertion of identical bytes. Covers the frozen "
                "target-resolution negatives and the digest pins."
            ),
            "targets": [
                "af-ir SubflowIrUniverse",
                "spec/conformance/subflow-interface/negative/",
            ],
        },
        {
            "suite_id": "SUITE-SUBFLOW-INTERFACE-DERIVATION",
            "description": (
                "The subflow interface is derived, canonical-key ordered "
                "and never durable: inputs from RunParameterId order, "
                "outputs from canonical name order, resources from every "
                "declared ResourceSlot in id order, 1-based independent "
                "namespaces, deterministic for a fixed digest. Covers the "
                "frozen interface projections and the equivalence groups."
            ),
            "targets": [
                "af-ir derive_subflow_interface",
                "spec/conformance/subflow-interface/equivalence/",
            ],
        },
        {
            "suite_id": "SUITE-SUBFLOW-INTERFACE-PASS-ORDER",
            "description": (
                "The subflow pass consumes a checked document plus its own "
                "Phase 3C analysis and runs strictly after them: Phase 3B "
                "and Phase 3C error kinds never move, and a document "
                "without CALL_SUBFLOW passes against an empty universe. "
                "Covers the frozen corpus rejected_at layering and the "
                "zero-drift proofs for both earlier corpora."
            ),
            "targets": [
                "af-ir CheckedIrDocument::check_subflow_interfaces",
                "spec/conformance/subflow-interface/",
            ],
        },
]

SUITE_LIST.sort(key=lambda suite: suite["suite_id"])
write_json("spec/conformance/suites.json", {
    "kind": "agentflow.conformance.suites",
    "suite_revision": 1,
    "suites": SUITE_LIST,
})