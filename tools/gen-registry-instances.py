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

# ---------------------------------------------------------------------------
# Phase 4B.2A computation leaf-resolution rules (area SEMANTIC)
# ---------------------------------------------------------------------------

LEAF_RESOLUTION_SRC = "spec/computation/leaf-contract-resolution.md"

LEAF_RESOLUTION_RULES = [
    rule("AF-SEMANTIC-0111", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-exact-pin",
         "A component target resolves only by the complete exact digest pin "
         "in an explicit snapshot; identity comes from the checked contract, "
         "with no latest or name-only fallback.",
         ["computation-leaf/invoke/exact-contract",
          "computation-leaf/invoke/wrong-contract-digest",
          "SUITE-COMPUTATION-LEAF-INVOKE"]),
    rule("AF-SEMANTIC-0112", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-checked-contract",
         "INVOKE requires a digest-recomputed CheckedComponentContract under "
         "the exact snapshot proof-context object; unresolved targets are "
         "never pure.",
         ["computation-leaf/invoke/exact-contract",
          "computation-leaf/invoke/wrong-contract-digest",
          "SUITE-COMPUTATION-LEAF-CONTEXT"]),
    rule("AF-SEMANTIC-0113", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-public-port-types",
         "Component input/output mappings require exact port numbers, exact "
         "public TypeSchemaRef pins, and existing Phase 3C scope, dominance, "
         "and producer proofs; local types are not public type evidence.",
         ["computation-leaf/invoke/exact-input-type",
          "computation-leaf/invoke/wrong-public-input-type",
          "computation-leaf/invoke/duplicate-required-input",
          "computation-leaf/invoke/missing-required-input",
          "computation-leaf/invoke/unmapped-local-type",
          "computation-leaf/invoke/invalid-output-producer",
          "SUITE-COMPUTATION-LEAF-INVOKE"]),
    rule("AF-SEMANTIC-0114", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-resource-ports",
         "Component resource ports map explicitly to visible ResourceSlotIds "
         "with exact resource type pins; absent attribute evidence remains "
         "Unproven and aliases never reduce bounds or grant authority.",
         ["computation-leaf/mapping/resource-alias-explicit",
          "computation-leaf/mapping/numeric-namespaces-distinct",
          "computation-leaf/mapping/wrong-resource-type",
          "computation-leaf/mapping/required-attributes-without-evidence",
          "SUITE-COMPUTATION-LEAF-MAPPING"]),
    rule("AF-SEMANTIC-0115", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-effect-occurrences",
         "Scoped effects retain exact source contract, full atom qualifiers, "
         "bounds, caller digest and node identity; distinct call sites never "
         "merge by contract digest.",
         ["computation-leaf/invoke/same-contract-two-call-sites",
          "computation-leaf/mapping/effect-qualifiers-preserved",
          "SUITE-COMPUTATION-LEAF-SCOPED-SUMMARIES"]),
    rule("AF-SEMANTIC-0116", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-information-flow",
         "Flow coordinates map only through exact I/O/R/effect mappings; "
         "unknown origins, influence, taints and authority bounds remain "
         "conservative and are never dropped or elevated.",
         ["computation-leaf/mapping/unknown-external-flow-preserved",
          "computation-leaf/mapping/effect-qualifiers-preserved",
          "computation-leaf/await/unknown-signal-provenance",
          "SUITE-COMPUTATION-LEAF-SCOPED-SUMMARIES"]),
    rule("AF-SEMANTIC-0117", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-dimensions",
         "Checked Failure/Effect/Flow/Usage/Recovery/Execution declarations "
         "remain per-invocation summaries; missing dimensions are not empty, "
         "and this phase performs no scaling or workflow placement closure.",
         ["computation-leaf/mapping/missing-usage-not-zero",
          "computation-leaf/mapping/recovery-action-full-effect-key",
          "SUITE-COMPUTATION-LEAF-SCOPED-SUMMARIES"]),
    rule("AF-SEMANTIC-0118", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-subflow-target",
         "CALL_SUBFLOW resolves its exact SemanticIrDigest in the Phase 3D "
         "universe and reuses the Phase 3D call-site interface checker.",
         ["computation-leaf/subflow/exact-export-interface",
          "computation-leaf/subflow/wrong-target-version",
          "SUITE-COMPUTATION-LEAF-SUBFLOW"]),
    rule("AF-SEMANTIC-0119", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-subflow-export",
         "A subflow export requires a complete checked ComponentContract "
         "whose exact I/O/R pins match the derived target interface; derived "
         "I/O/R never synthesizes missing dimensions.",
         ["computation-leaf/subflow/exact-export-interface",
          "computation-leaf/subflow/derived-io-not-full-contract",
          "SUITE-COMPUTATION-LEAF-SUBFLOW"]),
    rule("AF-SEMANTIC-0120", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-subflow-body-proof",
         "Resolving a subflow export declaration does not prove body "
         "conformance; absent a trusted complete proof the relation remains "
         "RequiresEvidence.",
         ["computation-leaf/subflow/declaration-needs-body-proof",
          "SUITE-COMPUTATION-LEAF-SUBFLOW"]),
    rule("AF-SEMANTIC-0121", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-subflow-closure",
         "The explicit exact-digest dependency closure fails closed on cycles "
         "and reports missing targets as Unproven without recursive expansion.",
         ["computation-leaf/subflow/direct-cycle",
          "computation-leaf/subflow/indirect-cycle",
          "computation-leaf/subflow/missing-target-closure",
          "SUITE-COMPUTATION-LEAF-SUBFLOW"]),
    rule("AF-SEMANTIC-0122", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-await",
         "AWAIT preserves its exact signal pin and correlation while missing "
         "adapter dimensions remain partial; it has no implicit timeout, "
         "Effect dispatch, business cancellation failure, or authority uplift.",
         ["computation-leaf/await/typed-signal-partial",
          "computation-leaf/await/unknown-signal-provenance",
          "computation-leaf/await/no-implicit-timeout",
          "computation-leaf/await/cancellation-not-business-failure",
          "SUITE-COMPUTATION-LEAF-AWAIT"]),
    rule("AF-SEMANTIC-0123", "SEMANTIC", LEAF_RESOLUTION_SRC,
         "leaf-resolution-digest-stability",
         "ComponentContractDigest depends on canonical semantic body, not "
         "proof context; checked positive claims remain context-bound and "
         "must be revalidated after context change.",
         ["computation-leaf/context/same-body-different-context",
          "computation-leaf/context/changed-catalog-invalidates-proof",
          "computation-leaf/context/same-version-different-content",
          "computation-leaf/equivalence/presentation-metadata-only",
          "SUITE-COMPUTATION-LEAF-CONTEXT"]),
]
RULES.extend(LEAF_RESOLUTION_RULES)

# ---------------------------------------------------------------------------
# Phase 4B.2B.1 structured control contract algebra (area SEMANTIC)
# ---------------------------------------------------------------------------

CONTROL_ALGEBRA_SRC = "spec/computation/control-contract-algebra.md"

CONTROL_ALGEBRA_RULES = [
    rule("AF-SEMANTIC-0124", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-summary-is-derived-only",
         "A control result is a source-traceable non-wire DerivedControlContractSummary, never a CheckedComponentContract or digest input.",
         ["computation-control/golden/minimal-known-sequence-envelope", "SUITE-COMPUTATION-CONTROL-PHASE-BOUNDARY"]),
    rule("AF-SEMANTIC-0125", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-context-and-leaves",
         "Bottom-up traversal requires the exact current caller SemanticIrDigest, proof-context object, validator-created leaf summaries and Phase 3C evidence.",
         ["computation-control/equivalence/same-snapshot-deterministic", "computation-control/sequence/typed-cross-sibling-edge", "SUITE-COMPUTATION-CONTROL-DIMENSIONS"]),
    rule("AF-SEMANTIC-0126", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-unsupported-not-empty",
         "Unknown or unsupported control operators remain Unproven; known descendants are not discarded and no row defaults empty. The formerly unsupported SCOPE and COMPENSATION_SCOPE now use the additive Phase 4B.2B.3 rules.",
         ["computation-advanced-control/negative/scope-unsupported", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0127", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-sequence-effect",
         "SEQUENCE preserves semantic child order, uses existing Effect accumulation, weakens later-child lower bounds when prior success is not proven, and retains upper bounds/occurrences.",
         ["computation-control/sequence/two-effects-accumulate", "computation-control/sequence/second-lower-conditional", "computation-control/sequence/same-atom-occurrences-distinct", "computation-control/negative/effect-overflow", "SUITE-COMPUTATION-CONTROL-SEQUENCE"]),
    rule("AF-SEMANTIC-0128", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-select-exclusive",
         "SELECT uses the frozen exclusive Effect branch_join and Usage maximum, retaining complete AtomKey qualifiers and guard-sensitive lower bounds.",
         ["computation-control/select/exclusive-effect-join", "computation-control/select/different-guards-lower-zero", "computation-control/select/qualifier-members-distinct", "computation-control/select/usage-maximum", "SUITE-COMPUTATION-CONTROL-SELECT"]),
    rule("AF-SEMANTIC-0129", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-select-coverage-and-influence",
         "SELECT totality is not invented without a default/range proof; selector dependencies remain decision influence and cannot elevate authority.",
         ["computation-control/select/decision-input-influence", "computation-control/select/unmatched-ordinal-unproven", "computation-control/select/exact-constant-case", "computation-control/proof/authority-upper-not-permission", "SUITE-COMPUTATION-CONTROL-SELECT"]),
    rule("AF-SEMANTIC-0130", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-parallel-all",
         "The frozen PARALLEL all mode accumulates every branch's possible Effect and Usage; max_inflight is concurrency only, and outputs use the explicit typed branch mapping.",
         ["computation-control/parallel/all-effects-accumulate", "computation-control/parallel/all-usage-accumulates", "computation-control/parallel/max-inflight-not-total-count", "computation-control/parallel/output-explicit-branch-map", "SUITE-COMPUTATION-CONTROL-PARALLEL"]),
    rule("AF-SEMANTIC-0131", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-parallel-fail-fast",
         "PARALLEL fail_fast retains sibling Effect/Usage/Failure facts despite cancellation and weakens lower bounds whose full execution is not guaranteed.",
         ["computation-control/parallel/fail-fast-sibling-effect-retained", "computation-control/parallel/fail-fast-lower-zero", "computation-control/proof/cancellation-not-failure", "SUITE-COMPUTATION-CONTROL-PARALLEL"]),
    rule("AF-SEMANTIC-0132", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-race-effect-usage",
         "RACE is FIRST_TERMINAL, not a SELECT join: all contestants including losers contribute Effect and Usage upper bounds, with only unguaranteed lower bounds weakened.",
         ["computation-control/race/all-contestants-not-winner-only", "computation-control/race/loser-effect-retained", "computation-control/race/loser-usage-retained", "computation-control/race/cancel-not-no-effect", "SUITE-COMPUTATION-CONTROL-RACE"]),
    rule("AF-SEMANTIC-0133", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-race-recovery",
         "RACE retains every contestant's exact Recovery/K and per-action Usage source facts, including losers; compensation never subtracts forward effects.",
         ["computation-control/race/loser-recovery-retained", "computation-control/proof/compensation-not-subtraction", "SUITE-COMPUTATION-CONTROL-RACE"]),
    rule("AF-SEMANTIC-0134", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-race-outcome-flow",
         "Failure rows union across possible branches; cancellation/timeout are not typed business Failure, and static analysis never selects a physical winner or primary cause.",
         ["computation-control/race/winner-may-fail", "computation-control/race/physical-timing-not-decision", "computation-control/proof/cancellation-not-failure", "SUITE-COMPUTATION-CONTROL-RACE"]),
    rule("AF-SEMANTIC-0135", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-typed-interface",
         "Control outputs reuse checked IR interfaces, Phase 3C producer evidence and explicit Parallel branch/port mapping; local types are not public schema aliases.",
         ["computation-control/sequence/typed-output-last-child", "computation-control/parallel/output-explicit-branch-map", "computation-control/select/typed-merge-mismatch", "SUITE-COMPUTATION-CONTROL-DIMENSIONS"]),
    rule("AF-SEMANTIC-0136", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-flow-may-influence",
         "Flow sources/taints union conservatively, may-carry authority takes its existing upper bound, unknown remains sticky, and selector/sequence/winner influence is not discarded.",
         ["computation-control/select/taints-union", "computation-control/select/decision-input-influence", "computation-control/sequence/typed-cross-sibling-edge", "computation-control/race/winner-output-taints-union", "SUITE-COMPUTATION-CONTROL-DIMENSIONS"]),
    rule("AF-SEMANTIC-0137", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-usage-propagation",
         "Usage uses checked accumulation or exclusive join by (phase,meter); missing coordinates remain Unknown, Forward/Recovery stay separate, and no retry/concurrency multiplier is inferred.",
         ["computation-control/sequence/usage-checked-sum", "computation-control/select/usage-maximum", "computation-control/parallel/all-usage-accumulates", "computation-control/race/loser-usage-retained", "computation-control/proof/missing-usage-not-zero", "SUITE-COMPUTATION-CONTROL-DIMENSIONS"]),
    rule("AF-SEMANTIC-0138", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-recovery-per-child",
         "Recovery contracts, action identities and profiles remain per child; RACE losers do not lose declared recovery obligations.",
         ["computation-control/race/loser-recovery-retained", "computation-control/proof/compensation-not-subtraction", "SUITE-COMPUTATION-CONTROL-RACE"]),
    rule("AF-SEMANTIC-0139", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-resource-port-provenance",
         "Resource ports stay source-scoped; numeric port equality never merges namespaces, and aliases require the exact same caller ResourceSlotId.",
         ["computation-control/parallel/shared-slot-alias-explicit", "computation-control/parallel/different-port-namespaces", "SUITE-COMPUTATION-CONTROL-DIMENSIONS"]),
    rule("AF-SEMANTIC-0140", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-execution-per-child",
         "ExecutionRequirements remain per child and are not summed into a single Worker, placement, admission or capacity requirement.",
         ["computation-control/parallel/execution-per-child", "computation-control/proof/execution-not-worker-sum", "SUITE-COMPUTATION-CONTROL-PHASE-BOUNDARY"]),
    rule("AF-SEMANTIC-0141", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-occurrence-identity",
         "Repeated uses of one contract remain distinct by caller digest, control/leaf node, contract digest and source ordinal even if the envelope algebra combines equal AtomKeys.",
         ["computation-control/sequence/same-atom-occurrences-distinct", "computation-control/parallel/same-contract-occurrences", "computation-control/race/same-atom-distinct-occurrences", "SUITE-COMPUTATION-CONTROL-OCCURRENCE"]),
    rule("AF-SEMANTIC-0142", "SEMANTIC", CONTROL_ALGEBRA_SRC,
         "control-proof-obligation-propagation",
         "Child Violation/Unproven/RequiresEvidence facts remain per-dimension obligations; a computed envelope cannot downgrade child evidence or be issued as a checked ComponentContract.",
         ["computation-control/proof/requires-evidence-propagates", "computation-control/proof/unproven-propagates", "computation-control/proof/known-bound-with-evidence", "computation-control/proof/unknown-effect-row-not-empty", "computation-control/proof/open-effect-row-remains-open", "SUITE-COMPUTATION-CONTROL-PROOF"]),
]
RULES.extend(CONTROL_ALGEBRA_RULES)

ADVANCED_CONTROL_ALGEBRA_SRC = "spec/computation/advanced-control-contract-algebra.md"

ADVANCED_CONTROL_ALGEBRA_RULES = [
    rule("AF-SEMANTIC-0143", "SEMANTIC", ADVANCED_CONTROL_ALGEBRA_SRC,
         "advanced-control-iterate-bounds",
         "ITERATE uses conservative bounded repetition; dynamic FOREACH has no proven total count, max_inflight is not total work, and carry outputs require exact checked mapping.",
         ["computation-advanced-control/iterate/foreach-03", "computation-advanced-control/iterate/foreach-06", "computation-advanced-control/iterate/while-05", "SUITE-COMPUTATION-ADVANCED-CONTROL-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0144", "SEMANTIC", ADVANCED_CONTROL_ALGEBRA_SRC,
         "advanced-control-retry-envelope-and-recovery",
         "RETRY preserves ordered finite delays and separate physical-attempt bounds, conservatively amplifies logical Effect/Usage, and evaluates existing K without runtime evidence or inferred retry policy.",
         ["computation-advanced-control/retry/schedule-04", "computation-advanced-control/negative/retry-effect-overflow", "computation-advanced-control/negative/retry-forbidden-replay", "computation-advanced-control/negative/retry-usage-overflow", "SUITE-COMPUTATION-ADVANCED-CONTROL-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0145", "SEMANTIC", ADVANCED_CONTROL_ALGEBRA_SRC,
         "advanced-control-timeout-outcomes",
         "TIMEOUT preserves child contracts and represents child completion and TimedOut separately; timeout never proves successful cancellation or zero incurred Effect/Usage.",
         ["computation-advanced-control/timeout/boundary-04", "computation-advanced-control/timeout/dynamic-duration", "SUITE-COMPUTATION-ADVANCED-CONTROL-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0146", "SEMANTIC", ADVANCED_CONTROL_ALGEBRA_SRC,
         "advanced-control-handle-residual",
         "HANDLE matches exact Failure schema pins, preserves unmatched residual failures, keeps TimedOut outside Phi, and composes handler Effect/Usage as conditional alternatives.",
         ["computation-advanced-control/handle/typed-clause-03", "computation-advanced-control/negative/handle-unmatched-type", "SUITE-COMPUTATION-ADVANCED-CONTROL-PYTHON-ORACLE"]),
]
RULES.extend(ADVANCED_CONTROL_ALGEBRA_RULES)

SCOPE_CLOSURE_ALGEBRA_SRC = "spec/computation/scope-compensation-contract-algebra.md"

SCOPE_CLOSURE_ALGEBRA_RULES = [
    rule("AF-SEMANTIC-0147", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "scope-resource-attenuation",
         "SCOPE compares exact Phase 3C parent, declared, and child ResourceSlot sets; child visibility is exactly the declared subset and does not imply a ResourceHandle or authorization.",
         ["computation-scope-closure/scope/resource-boundary-01", "computation-scope-closure/nested/tree-10", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0148", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "scope-static-obligations",
         "SCOPE preserves child dimensions and ownership; missing authority/budget ceilings remain Unproven and child settlement before owner terminal RequiresEvidence.",
         ["computation-scope-closure/scope/resource-boundary-04", "computation-advanced-control/negative/scope-unsupported", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0149", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "compensation-forward-history",
         "COMPENSATION_SCOPE preserves Forward Effects, typed outcomes, and Forward Usage; potential compensation is additional and never refunds or rewrites the Forward history.",
         ["computation-scope-closure/compensation/action-bound-01", "computation-scope-closure/nested/tree-04", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0150", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "compensation-exact-action-envelope",
         "Only exact full RecoveryActionIdentity and RecoveryUsageBinding sources contribute; checked forward-occurrence, activation, and attempt bounds produce a conditional new Effect with zero lower bound, and the summed K upper must fit the exact declared action Effect occurrence.",
         ["computation-scope-closure/compensation/action-bound-06", "computation-scope-closure/compensation/action-bound-18", "computation-scope-closure/negative/action-row-underdeclared", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0151", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "compensation-receipt-usage-and-order",
         "Recovery Usage remains separate and is checked against potential action bounds; receipt eligibility and multi-event committed order require evidence, and RecoveryBlocked is not a business Failure.",
         ["computation-scope-closure/compensation/action-bound-06", "computation-scope-closure/nested/tree-05", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0152", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "workflow-control-tree-closure",
         "Workflow-root derivation resolves all reachable leaves and accounts for every checked owned computation, known Effect occurrence/residual, and Recovery source exactly once in semantic order without promoting child obligations.",
         ["computation-scope-closure/nested/tree-01", "computation-scope-closure/nested/tree-15", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0153", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "workflow-root-export-boundary",
         "A root derived summary is not an exported ComponentContract; without a frozen root pin and cross-coordinate mapping, export compatibility remains NotExpressedInSemanticIr and Unproven.",
         ["computation-scope-closure/nested/tree-15", "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0154", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "root-recovery-source-closure",
         "Root Recovery closure compares the complete source, action-binding, forward-occurrence, and exact Usage-profile multisets; source-node coverage alone cannot prove action/profile accounting.",
         ["computation-root-recovery/root/K07", "computation-root-recovery/root/K13", "computation-root-recovery/root/K22", "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0155", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "root-recovery-multiplicity",
         "LocalActionBound and RootActionBound are distinct coordinates; exact caller/control-node/kind multiplicity identities are applied once with checked arithmetic and physical attempts are not multiplied twice.",
         ["computation-root-recovery/root/K01", "computation-root-recovery/root/K02", "computation-root-recovery/root/K03", "computation-root-recovery/root/K04", "computation-root-recovery/root/K05", "computation-root-recovery/root/K19", "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0156", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "root-recovery-operator-algebra",
         "Root Recovery uses operator-specific Effect/Usage algebra, retains typed control-path and validator-created guard evidence separately, keeps both action kinds and RACE losers, and assigns each action to its nearest CompensationScope or typed RootRecovery owner.",
         ["computation-root-recovery/root/K08", "computation-root-recovery/root/K09", "computation-root-recovery/root/K10", "computation-root-recovery/root/K11", "computation-root-recovery/root/K12", "computation-root-recovery/root/K13", "computation-root-recovery/root/K14", "computation-root-recovery/root/K15", "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0157", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "root-recovery-epsilon-coverage",
         "The root K action projection, including a same-atom Forward opportunity exactly once, must fit the already derived root epsilon ceiling and is never added as a second epsilon declaration.",
         ["computation-root-recovery/root/K06", "computation-root-recovery/root/K06-underdeclared", "computation-root-recovery/root/K07", "computation-root-recovery/root/K20", "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0158", "SEMANTIC", SCOPE_CLOSURE_ALGEBRA_SRC,
         "root-recovery-usage-coverage",
         "Root K-required Recovery Usage must fit exact root U.recovery meter coordinates without double addition; missing active-source meters are not zero, overflow is a Violation, and runtime eligibility remains evidence-dependent.",
         ["computation-root-recovery/root/K16", "computation-root-recovery/root/K17", "computation-root-recovery/root/K18", "computation-root-recovery/root/K20", "computation-root-recovery/root/K21", "computation-root-recovery/root/K22", "computation-root-recovery/root/K24", "computation-root-recovery/root/K25-overflow", "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE"]),
]
RULES.extend(SCOPE_CLOSURE_ALGEBRA_RULES)

ROOT_EXPORT_PROOF_SRC = "spec/computation/root-export-contract-proof.md"
ROOT_EXPORT_PROOF_RULES = [
    rule("AF-SEMANTIC-0159", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "exact-assertion-pins",
         "The assertion pins the exact SemanticIrDigest and checked exported ComponentContractDigest.",
         ["computation-root-export/positive/exact-empty-static-envelope", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0160", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "proof-context-manifest",
         "The proof-context manifest commits the complete typed static universes, relation content, and resolver pins.",
         ["computation-root-export/context/relation-content-change", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0161", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "compiler-semantics-identity",
         "Compiler semantics identity includes exact normative content, applied RuleIds, and checker semantics version.",
         ["computation-root-export/context/compiler-semantics-change", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0162", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "root-input-mapping",
         "Root inputs require a complete bijection to exported input ports with exact nominal types and checked use sites.",
         ["computation-root-export/positive/exact-input-output", "computation-root-export/negative/input-source-alias", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0163", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "root-output-mapping",
         "Every exported output maps to an exact named WorkflowOutput with checked producer and nominal type evidence.",
         ["computation-root-export/positive/exact-input-output", "computation-root-export/negative/output-map-missing", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0164", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "root-resource-mapping",
         "Root resources map through exact ResourceSlot and exported requirement-port identities.",
         ["computation-root-export/negative/resource-slot-map-change", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0165", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "failure-containment",
         "Possible root business failures are contained by the exact exported nominal failure row.",
         ["computation-root-export/positive/exact-empty-static-envelope", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0166", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "effect-containment",
         "Root effects and K action possibilities fit the exported epsilon ceiling after exact coordinate mapping.",
         ["computation-root-export/negative/effect-upper-underdeclared", "computation-root-export/negative/effect-qualifier-map-change", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0167", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "information-flow-non-relaxation",
         "Root information flow does not relax the exported flow restrictions; unknown influence is retained.",
         ["computation-root-export/negative/flow-map-missing", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0168", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "usage-containment",
         "Root forward and recovery usage remain within separate exact exported phase, meter, unit, and bound coordinates.",
         ["computation-root-export/negative/recovery-usage-underdeclared", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0169", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "recovery-compatibility",
         "Root recovery sources, actions, profiles, and safety declarations are compared without invented enum ordering.",
         ["computation-root-export/negative/recovery-map-missing", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0170", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "execution-compatibility",
         "Every child execution source meets exported feasibility and security constraints independently.",
         ["computation-root-export/positive/exact-input-output", "computation-root-export/negative/execution-security", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0171", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "subflow-body-separation",
         "Subflow target pins do not prove implementation or body conformance.",
         ["computation-root-export/subflow/body-evidence-absent", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
    rule("AF-SEMANTIC-0172", "SEMANTIC", ROOT_EXPORT_PROOF_SRC,
         "static-runtime-boundary",
         "Static compatibility does not establish authority, budget reservation, or runtime authorization.",
         ["computation-root-export/positive/exact-empty-static-envelope", "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE"]),
]
RULES.extend(ROOT_EXPORT_PROOF_RULES)

# ---------------------------------------------------------------------------
# Effect Signature rules (Phase 4A.1, area EFFECT)
# ---------------------------------------------------------------------------

EFFECT_SRC = "spec/effect/effect-signature.md"

EFFECT_RULES = [
    rule("AF-EFFECT-0001", "EFFECT", EFFECT_SRC, "effect-declaration-not-authorization",
         "An effect signature is a static declaration, never authorization: "
         "Declared != Granted != Dispatched != Observed, "
         "Dispatched ⊆ Granted ⊆ Declared is a Runtime/Effect-Gateway "
         "invariant, and no API converts a signature into authority.",
         ["SUITE-EFFECT-DECLARATION-NOT-AUTHORIZATION",
          "SUITE-EFFECT-NO-RUNTIME"]),
    rule("AF-EFFECT-0002", "EFFECT", EFFECT_SRC, "effect-atom",
         "An effect atom is exactly operation + resource selector + closed "
         "qualifiers; canonical identity includes qualifier family, member "
         "discriminant and semantic payload, never family alone.",
         ["effect/golden/single-write", "effect/golden/multiple-distinct",
          "effect/golden/qualifier-natural-vs-none-distinct",
          "effect/golden/same-operation-different-resource-still-distinct",
          "effect/negative/same-full-qualifier-duplicate-rejected",
          "effect/algebra/qualifier-member-different-contains",
          "effect/algebra/qualifier-identity-guard-remains-analysis",
          "effect/algebra/qualifier-identity-bounds-remain-analysis",
          "effect/algebra/qualifier-member-different-require-contained",
          "effect/algebra/qualifier-member-different-accumulate",
          "effect/algebra/qualifier-member-different-branch-join",
          "effect/algebra/qualifier-member-different-repetition",
          "effect/algebra/qualifier-member-sequence-propagation",
          "effect/algebra/qualifier-member-parallel-propagation",
          "effect/algebra/qualifier-member-race-propagation",
          "effect/algebra/qualifier-identity-excludes-guard-and-bounds",
          "effect/equivalence/qualifier-member-canonical-order"]),
    rule("AF-EFFECT-0003", "EFFECT", EFFECT_SRC, "effect-operation-identity",
         "An operation identity is catalog (a SpecificationId) plus a dotted "
         "lowercase name; resolution against the supplied explicit catalog "
         "universe is fail-closed with no default catalog and no fallback.",
         ["effect/golden/single-read", "effect/negative/unknown-operation",
          "effect/negative/invalid-operation-name",
          "effect/negative/invalid-catalog-identity",
          "SUITE-EFFECT-CATALOG-EXPLICIT-UNIVERSE"]),
    rule("AF-EFFECT-0004", "EFFECT", EFFECT_SRC, "effect-resource-selector",
         "A resource selector is a >= 1 requirement port plus exact-binding "
         "scope constraints; it never carries a raw handle, filesystem root, "
         "database connection, secret or worker endpoint, and scope "
         "containment is exact-binding set inclusion, never a prefix match.",
         ["effect/golden/resource-scope", "effect/negative/zero-requirement-port",
          "effect/negative/duplicate-scope-key"]),
    rule("AF-EFFECT-0005", "EFFECT", EFFECT_SRC, "effect-occurrence-bounds",
         "Occurrence bounds are explicit [lower, upper] intervals with an "
         "explicit unbounded ceiling (never UINT64_MAX); they count logical "
         "effect occurrences, physical attempts are tracked separately, and "
         "impossible intervals are rejected.",
         ["effect/golden/occurrence-bounds",
          "effect/negative/occurrence-lower-above-upper",
          "effect/algebra/repeat-n"]),
    rule("AF-EFFECT-0006", "EFFECT", EFFECT_SRC, "effect-pure",
         "Pure is exactly the closed empty row; zero-occurrence entries are "
         "semantically pure and dropped by the canonical form.",
         ["effect/golden/pure-empty-row", "effect/golden/zero-occurrence-entry",
          "effect/algebra/repeat-zero"]),
    rule("AF-EFFECT-0007", "EFFECT", EFFECT_SRC, "effect-top",
         "The unknown row (⊤) is an unprovable-envelope analysis result: it "
         "is not pure, contaminates compositions upward, carries no "
         "capability material and is never authorization.",
         ["effect/negative/top-is-not-pure",
          "effect/negative/top-is-not-authorization"]),
    rule("AF-EFFECT-0008", "EFFECT", EFFECT_SRC, "effect-open-row",
         "A bounded-open row keeps known effects plus a residual ceiling "
         "that must constrain something (a scope binding or a finite "
         "occurrence upper); an open row that bounds nothing is pure.",
         ["effect/golden/bounded-open-row",
          "effect/negative/unbounded-open-row"]),
    rule("AF-EFFECT-0009", "EFFECT", EFFECT_SRC, "effect-containment",
         "Containment (⊑) must be PROVED per entry (exact operation, equal "
         "qualifiers, narrower scope, contained interval, implied guard) or "
         "returned as NOT PROVEN with a stable reason; unresolved proofs "
         "fail closed as effect_envelope_unprovable.",
         ["effect/algebra/containment-reflexivity",
          "effect/algebra/containment-transitivity"]),
    rule("AF-EFFECT-0010", "EFFECT", EFFECT_SRC, "effect-accumulation",
         "Accumulation (⊕) adds occurrence intervals per atom key with "
         "checked arithmetic, keeps distinct atoms, composes guards "
         "conservatively (lower bounds only across unconditional entries), "
         "and is the identity of the pure row.",
         ["effect/algebra/empty-identity",
          "effect/algebra/accumulation-associativity",
          "effect/algebra/sequence-accumulation",
          "effect/algebra/parallel-accumulation"]),
    rule("AF-EFFECT-0011", "EFFECT", EFFECT_SRC, "effect-branch-join",
         "Branch join (⊔) keeps the possible effects of both mutually "
         "exclusive branches: a guarded lower bound is guaranteed only "
         "under its guard — identical guards keep the conditional minimum, "
         "different conditions widen to unconditional with the lower bound "
         "weakened to zero, and one-sided keys drop to [0, u] with their "
         "guard kept.",
         ["effect/algebra/join-commutativity",
          "effect/algebra/join-idempotence",
          "effect/algebra/join-same-unconditional-guard",
          "effect/algebra/join-same-conditional-guard",
          "effect/algebra/join-different-input-equals-guards",
          "effect/algebra/join-unconditional-vs-conditional",
          "effect/algebra/join-one-sided-guarded-effect",
          "effect/algebra/join-commutativity-with-guards",
          "effect/algebra/join-idempotence-with-guards",
          "effect/algebra/join-associativity-with-guards"]),
    rule("AF-EFFECT-0012", "EFFECT", EFFECT_SRC, "effect-race",
         "RACE is conservative accumulation over ALL contestants: a loser "
         "may have produced effects before cancellation, so loser effects "
         "are never discarded and race is not a branch join.",
         ["effect/algebra/race-loser-retained"]),
    rule("AF-EFFECT-0013", "EFFECT", EFFECT_SRC, "effect-iterate",
         "ITERATE scales the body envelope by the statically proven "
         "repetition bounds; an unprovable repetition count never yields a "
         "finite envelope, and zero repetitions are pure.",
         ["effect/algebra/repeat-one", "effect/algebra/repeat-n"]),
    rule("AF-EFFECT-0014", "EFFECT", EFFECT_SRC, "effect-retry",
         "RETRY separates physical attempt bounds [1, max_attempts] from "
         "the logical effect envelope, which stays conservatively scaled "
         "(no idempotency assumption) and records the explicit obligation "
         "that narrowing requires a recovery contract.",
         ["effect/algebra/retry-amplification",
          "SUITE-EFFECT-RETRY-BOUNDARY"]),
    rule("AF-EFFECT-0015", "EFFECT", EFFECT_SRC, "effect-compensation",
         "Compensation adds the counter-effect to the envelope and never "
         "subtracts forward effects: the original effect and the "
         "counter-effect are both facts.",
         ["effect/algebra/compensation-no-subtraction"]),
    rule("AF-EFFECT-0016", "EFFECT", EFFECT_SRC, "effect-canonicalization",
         "Canonicalization is deterministic and idempotent: entry order, "
         "scope binding order and qualifier order never change the canonical "
         "ACE1 bytes; declared duplicates are rejected (not merged); and no "
         "EffectSignatureDigest or new HashV1 domain is introduced.",
         ["effect/golden/canonical-reordering",
          "effect/equivalence/entry-reordering",
          "effect/equivalence/scope-binding-reordering",
          "effect/equivalence/zero-occurrence-equals-pure",
          "effect/equivalence/residual-reordering"]),
    rule("AF-EFFECT-0017", "EFFECT", EFFECT_SRC, "effect-guards",
         "The guard language is closed (unconditional, typed input "
         "equality): guards narrow when an effect may occur, their grammar "
         "is validated, and inexpressible conditions widen to unconditional "
         "instead of being assumed false.",
         ["effect/golden/guarded", "effect/negative/invalid-guard-port"]),
    rule("AF-EFFECT-0018", "EFFECT", EFFECT_SRC, "effect-overflow",
         "Occurrence arithmetic is checked: overflow is the explicit error "
         "effect_arithmetic_overflow, never a wrap-around and never a "
         "silent saturation into a smaller envelope.",
         ["effect/algebra/overflow"]),
]

RULES.extend(EFFECT_RULES)

# ---------------------------------------------------------------------------
# Failure Row rules (Phase 4A.2, area FAILURE)
# ---------------------------------------------------------------------------

FAILURE_SRC = "spec/failure/failure-row.md"

FAILURE_RULES = [
    rule("AF-FAILURE-0001", "FAILURE", FAILURE_SRC, "fr-nominal-identity",
         "A failure type is nominal: the full (SpecificationId, TypeName, "
         "pinned TypeSchemaDigest) triple; the same short name under a "
         "different specification or with a different digest is a "
         "DIFFERENT failure type, never silently compatible.",
         ["failure/golden/same-short-name-different-family",
          "failure/equivalence/declaration-ordering",
          "SUITE-FAILURE-NOMINAL-NO-STRING-MATCHING"]),
    rule("AF-FAILURE-0002", "FAILURE", FAILURE_SRC, "fr-closed-row",
         "The Failure Row is a closed set of nominal types with no "
         "open/residual/unknown wire state; unknown upper bounds are the "
         "analysis-level Unproven result, never a row variant.",
         ["failure/golden/empty-failure-row",
          "failure/golden/multiple-failure-types"]),
    rule("AF-FAILURE-0003", "FAILURE", FAILURE_SRC, "fr-exact-pinning",
         "Every payload pin resolves exactly against the explicit schema "
         "universe: the type must exist and the pinned digest must match; "
         "there is no latest-schema fallback.",
         ["failure/negative/unknown-schema",
          "failure/negative/wrong-schema-digest"]),
    rule("AF-FAILURE-0004", "FAILURE", FAILURE_SRC, "fr-empty-not-success",
         "The empty row is the algebra identity but does NOT mean the "
         "computation succeeds: TimedOut, Cancelled, RecoveryBlocked, "
         "TaskAttemptLost and RuntimeIncident all remain possible outside "
         "the row.",
         ["failure/golden/empty-failure-row",
          "failure/algebra/await-invents-nothing",
          "SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION"]),
    rule("AF-FAILURE-0005", "FAILURE", FAILURE_SRC, "fr-unknown-not-empty",
         "An unprovable failure upper bound is Unproven and demanding a "
         "row from it fails closed (failure_row_unproven); unknown is "
         "never interpreted as the empty row.",
         ["failure/negative/unproven-failure-containment",
          "SUITE-FAILURE-DEFERRED-CONTRACT-RESOLUTION"]),
    rule("AF-FAILURE-0006", "FAILURE", FAILURE_SRC, "fr-canonical-ordering",
         "Canonical ordering is deterministic: refs sorted by full "
         "nominal identity in byte order; declaration order and map "
         "iteration order never reach the canonical ACE1 bytes.",
         ["failure/golden/canonical-ordering",
          "failure/equivalence/declaration-ordering"]),
    rule("AF-FAILURE-0007", "FAILURE", FAILURE_SRC, "fr-duplicates-rejected",
         "Duplicate declarations of the same nominal type are rejected; "
         "the algebra's union may deduplicate - declaration and algebra "
         "are deliberately separate.",
         ["failure/negative/duplicate-failure-type",
          "failure/negative/ambiguous-duplicate-handled-type"]),
    rule("AF-FAILURE-0008", "FAILURE", FAILURE_SRC, "fr-containment-exact",
         "Containment is exact nominal membership: never inferred from "
         "the same short name, a similar schema or equal record fields; a "
         "future compatibility system needs an explicit proof.",
         ["failure/algebra/containment-reflexivity",
          "failure/algebra/containment-transitivity",
          "SUITE-FAILURE-NOMINAL-NO-STRING-MATCHING"]),
    rule("AF-FAILURE-0009", "FAILURE", FAILURE_SRC, "fr-union-algebra",
         "Union is commutative, associative and idempotent with the empty "
         "row as identity; branch composition (SELECT/PARALLEL/RACE) is "
         "exactly set union.",
         ["failure/algebra/union-identity",
          "failure/algebra/union-commutativity",
          "failure/algebra/union-associativity",
          "failure/algebra/union-idempotence"]),
    rule("AF-FAILURE-0010", "FAILURE", FAILURE_SRC, "fr-handled-declared",
         "A handled failure type must be declared by the child row: a "
         "HANDLE matcher matches an exact TypeSchemaRef the child "
         "declares, and a handled-but-undeclared type is rejected "
         "(failure_not_declared), never pretended matched.",
         ["failure/negative/handle-matcher-not-in-child-row"]),
    rule("AF-FAILURE-0011", "FAILURE", FAILURE_SRC, "fr-unhandled-propagate",
         "Unhandled failures propagate verbatim: the residual "
         "computation keeps every declared type not handled.",
         ["failure/algebra/handle-removes-exact-match",
          "failure/algebra/unhandled-child-failure-preserved"]),
    rule("AF-FAILURE-0012", "FAILURE", FAILURE_SRC, "fr-handler-propagate",
         "Handler failures propagate: handlers may fail themselves and "
         "their rows union into the result ((child minus H) union "
         "handlers), never treated as automatically handled.",
         ["failure/algebra/handler-new-failure-propagated"]),
    rule("AF-FAILURE-0013", "FAILURE", FAILURE_SRC, "fr-timedout-excluded",
         "TimedOut is a Control Outcome (the frozen HandleMatch::"
         "timed_out), never a failure type; TIMEOUT composition adds no "
         "type to the child row.",
         ["failure/algebra/timeout-does-not-introduce-failure",
          "SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION"]),
    rule("AF-FAILURE-0014", "FAILURE", FAILURE_SRC, "fr-cancelled-excluded",
         "Cancelled is a Control Outcome: parent cancellation is never a "
         "normal failure, no cancelled matcher exists, and no "
         "cancellation failure type can be introduced.",
         ["SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION"]),
    rule("AF-FAILURE-0015", "FAILURE", FAILURE_SRC, "fr-recoveryblocked-excluded",
         "RecoveryBlocked is a Recovery State: not a Workflow failure, "
         "not catchable by an ordinary HANDLE, never mapped into a "
         "business failure type by retry or compensation logic.",
         ["SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION"]),
    rule("AF-FAILURE-0016", "FAILURE", FAILURE_SRC, "fr-incidents-excluded",
         "TaskAttemptLost is an Infrastructure Fact (never automatically "
         "Failure<WorkerLost>) and RuntimeIncident lives on the Incident "
         "Plane; operational conditions and platform degradation are "
         "likewise outside the Failure Row.",
         ["SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION"]),
    rule("AF-FAILURE-0017", "FAILURE", FAILURE_SRC, "fr-retry-preserves",
         "RETRY preserves the child's failure row verbatim: retry "
         "attempts are not a proof that business failure is impossible, "
         "and only a future Recovery/Replay Contract may narrow it.",
         ["failure/algebra/retry-preserves-possible-failure"]),
]
RULES.extend(FAILURE_RULES)

# ---------------------------------------------------------------------------
# Resource Requirement Row rules (Phase 4A.3, area RESOURCE)
# ---------------------------------------------------------------------------

RESOURCE_SRC = "spec/resource/resource-requirement.md"

RESOURCE_RULES = [
    rule("AF-RESOURCE-0001", "RESOURCE", RESOURCE_SRC, "rr-exact-pinning",
         "A ResourceTypeRef is exact-pinned: nominal identity is the full "
         "(SpecificationId, TypeName, pinned TypeSchemaDigest) triple; an "
         "unregistered schema or a wrong digest is rejected, the same "
         "short name is never accepted.",
         ["resource/golden/same-short-name-different-family",
          "resource/negative/wrong-resource-type-digest",
          "SUITE-RESOURCE-NOMINAL-EXACT-PIN"]),
    rule("AF-RESOURCE-0002", "RESOURCE", RESOURCE_SRC, "rr-universe",
         "A resource type must be explicitly recognized by the explicit, "
         "read-only ResourceTypeUniverse: a legal AFIDL schema alone is "
         "not a resource type, and recognition pins the type's closed "
         "attribute vocabulary.",
         ["resource/negative/schema-not-recognized-as-resource-type",
          "SUITE-RESOURCE-EXPLICIT-UNIVERSE"]),
    rule("AF-RESOURCE-0003", "RESOURCE", RESOURCE_SRC, "rr-vs-capability",
         "A resource requirement does not grant authority: requirement "
         "satisfaction is a static predicate, never an authorization, and "
         "no requirement-satisfied-implies-grant path exists.",
         ["SUITE-RESOURCE-DECLARATION-NOT-AUTHORITY"]),
    rule("AF-RESOURCE-0004", "RESOURCE", RESOURCE_SRC, "rr-port-positive",
         "A requirement port must be positive (port_no > 0); port 0 is "
         "rejected.",
         ["resource/negative/invalid-port-zero"]),
    rule("AF-RESOURCE-0005", "RESOURCE", RESOURCE_SRC, "rr-ports-unique",
         "Resource ports must be unique within one requirement row: one "
         "contract interface declares at most one requirement per port.",
         ["resource/negative/duplicate-port"]),
    rule("AF-RESOURCE-0006", "RESOURCE", RESOURCE_SRC, "rr-canonical-attributes",
         "Requirement attributes are canonical: NFC-normalized text, "
         "unsigned UTF-8 key order, and the same semantic requirements "
         "always produce the same canonical ACE1 bytes.",
         ["resource/golden/canonical-ordering",
          "resource/equivalence/attribute-ordering"]),
    rule("AF-RESOURCE-0007", "RESOURCE", RESOURCE_SRC, "rr-attributes-duplicates",
         "Duplicate normalized attribute keys are rejected: two different "
         "values for one key are never silently overwritten, and "
         "NFC-equivalent spellings are the same key, not a legal "
         "equivalence input.",
         ["resource/negative/duplicate-attribute-key",
          "resource/negative/nfc-duplicate-attribute-key"]),
    rule("AF-RESOURCE-0008", "RESOURCE", RESOURCE_SRC, "rr-closed-row",
         "The requirement row is a closed set with no open/residual/"
         "unknown wire state; canonical order is ascending port number.",
         ["resource/golden/empty-requirement-row",
          "resource/golden/multiple-resource-ports"]),
    rule("AF-RESOURCE-0009", "RESOURCE", RESOURCE_SRC, "rr-unknown-not-empty",
         "An unknown requirement upper bound is never the empty row: "
         "empty would falsely claim the computation needs nothing.",
         ["SUITE-RESOURCE-WIRE-FREEZE"]),
    rule("AF-RESOURCE-0010", "RESOURCE", RESOURCE_SRC, "rr-no-latest-lookup",
         "There is no implicit latest lookup: resolution is exact against "
         "the explicitly supplied universe, with no fallback and no "
         "implicit upcast.",
         ["resource/negative/unknown-resource-type",
          "SUITE-RESOURCE-EXPLICIT-UNIVERSE"]),
    rule("AF-RESOURCE-0011", "RESOURCE", RESOURCE_SRC, "rr-satisfaction-exact-type",
         "Static satisfaction requires the exact resource type: same "
         "short name, same record shape or similar provider is never "
         "accepted, and satisfaction proves nothing beyond static "
         "attribute matching.",
         ["resource/negative/candidate-wrong-type",
          "resource/golden/same-short-name-different-family"]),
    rule("AF-RESOURCE-0012", "RESOURCE", RESOURCE_SRC, "rr-non-strengthening",
         "A requirement row cannot be strengthened: within one port "
         "namespace the new row's accepted candidate set must contain the "
         "old row's (per retained port, new.required_attributes subset of "
         "old.required_attributes as full (key,value) pairs); a new port "
         "or a new/changed condition proves strengthening.",
         ["resource/compatibility/identical-rows",
          "resource/compatibility/new-removes-attribute",
          "resource/compatibility/new-adds-attribute",
          "resource/compatibility/new-adds-port"]),
    rule("AF-RESOURCE-0013", "RESOURCE", RESOURCE_SRC, "rr-no-subtyping",
         "No implicit ResourceType subtyping exists: a retained port with "
         "a changed nominal type (including the same short name under a "
         "different pin) is Unproven, never compatible.",
         ["resource/compatibility/changed-nominal-type-same-port",
          "resource/compatibility/same-short-name-different-digest"]),
    rule("AF-RESOURCE-0014", "RESOURCE", RESOURCE_SRC, "rr-port-namespace",
         "Resource ports are local to one contract interface: comparing "
         "rows of two different namespaces is not a static proof and "
         "fails closed; equal port numbers across components do not name "
         "the same resource.",
         ["resource/golden/same-type-distinct-ports",
          "resource/compatibility/cross-component-namespaces-not-merged"]),
    rule("AF-RESOURCE-0015", "RESOURCE", RESOURCE_SRC, "rr-no-blind-union",
         "No cross-component merge without an explicit port mapping: "
         "aggregating requirement rows by equal port numbers is forbidden "
         "and no merge operation exists in this layer.",
         ["SUITE-RESOURCE-NO-CROSS-COMPONENT-MERGE"]),
    rule("AF-RESOURCE-0016", "RESOURCE", RESOURCE_SRC, "rr-authority-distinction",
         "The ResourceHandle capability ceiling, EffectGrant and "
         "ResourceRequirement are distinct: declaring a requirement "
         "produces no runtime authority and no attenuation.",
         ["SUITE-RESOURCE-DECLARATION-NOT-AUTHORITY"]),
    rule("AF-RESOURCE-0017", "RESOURCE", RESOURCE_SRC, "rr-exact-attributes",
         "Attribute constraints form a closed exact-value conjunctive "
         "predicate language: an attribute key outside the type's "
         "descriptor vocabulary has no known constraint meaning and is "
         "rejected; no glob, regex, path prefix, CIDR, policy expression "
         "or predicate script exists.",
         ["resource/negative/unknown-attribute",
          "resource/compatibility/unknown-attribute-implication-unproven"]),
    rule("AF-RESOURCE-0018", "RESOURCE", RESOURCE_SRC, "rr-unproven",
         "Unproven is a fail-closed analysis result: it never enters the "
         "wire model, is never interpreted as the empty row and is never "
         "auto-judged compatible.",
         ["resource/compatibility/cross-component-namespaces-not-merged"]),
]
RULES.extend(RESOURCE_RULES)

# ---------------------------------------------------------------------------
# Information Flow rules (Phase 4A.4, area INFOFLOW)
# ---------------------------------------------------------------------------

INFOFLOW_SRC = "spec/information-flow/information-flow.md"

INFOFLOW_RULES = [
    rule("AF-INFOFLOW-0001", "INFOFLOW", INFOFLOW_SRC, "if-four-concepts",
         "Provenance, Trust, Taint and Information Flow stay four distinct "
         "concepts: no trust score, no single trusted flag, and no "
         "collapse of the influence dimensions into one value.",
         ["infoflow/golden/source-taint-summary",
          "SUITE-INFOFLOW-CONCEPTS-DISTINCT"]),
    rule("AF-INFOFLOW-0002", "INFOFLOW", INFOFLOW_SRC, "if-label-identity",
         "Label identity is nominal (catalog SpecificationId, label name) "
         "resolved by the explicit read-only universe: unknown catalog or "
         "label rejected, duplicates rejected, no auto-registration, no "
         "case-insensitive matching, and label order never enters "
         "semantic identity.",
         ["infoflow/negative/unknown-label",
          "infoflow/equivalence/label-ordering"]),
    rule("AF-INFOFLOW-0003", "INFOFLOW", INFOFLOW_SRC,
         "if-confidentiality-taint-separate",
         "Confidentiality restrictions and taints are structurally "
         "separate nominal types: where data may go and what may have "
         "influenced a value are never unified into one collection.",
         ["infoflow/golden/single-confidentiality-label",
          "infoflow/golden/source-taint-summary",
          "SUITE-INFOFLOW-CONCEPTS-DISTINCT"]),
    rule("AF-INFOFLOW-0004", "INFOFLOW", INFOFLOW_SRC,
         "if-no-label-hierarchy",
         "No confidentiality hierarchy is invented: containment is set "
         "membership plus explicit catalog rules; no total order over "
         "user-defined labels exists from which downgrade flows could be "
         "claimed.",
         ["SUITE-INFOFLOW-NO-INVENTED-LATTICE"]),
    rule("AF-INFOFLOW-0005", "INFOFLOW", INFOFLOW_SRC,
         "if-no-authority-elevation",
         "Ordinary data transformation MUST NOT elevate instruction "
         "authority: content claiming to be a system instruction stays "
         "external_data, source authentication is not instruction "
         "authorization, and a MAX authority is only a may-carry upper "
         "bound, never effective instruction authority or endorsement.",
         ["infoflow/negative/untrusted-promoted-to-instruction",
          "infoflow/propagation/authority-never-elevated",
          "infoflow/propagation/mixed-user-and-web-authority",
          "infoflow/propagation/mixed-platform-and-external-authority"]),
    rule("AF-INFOFLOW-0006", "INFOFLOW", INFOFLOW_SRC,
         "if-conservative-propagation",
         "Taint propagation is may-influence conservative: the output "
         "taint is the union over all inputs that MAY influence it; an "
         "incomplete influence relation yields the explicit unknown or "
         "the conservative union of all possible inputs, and no author "
         "can prove a real influence away with an incomplete list.",
         ["infoflow/propagation/pure-expression-inherits-taints",
          "infoflow/propagation/unknown-influence-remains-unknown",
          "SUITE-INFOFLOW-CONSERVATIVE-PROPAGATION"]),
    rule("AF-INFOFLOW-0007", "INFOFLOW", INFOFLOW_SRC, "if-model-mixer",
         "The LLM strong taint mixer: all model-visible inputs may "
         "influence all model outputs, so output taint is the union of "
         "every visible input taint plus model_generated, influence "
         "stays unknown, and authority is never elevated.",
         ["infoflow/propagation/model-output-inherits-every-input-taint",
          "infoflow/propagation/model-output-adds-model-generated",
          "SUITE-INFOFLOW-MODEL-MIXER"]),
    rule("AF-INFOFLOW-0008", "INFOFLOW", INFOFLOW_SRC,
         "if-model-no-endorsement",
         "A model output is never endorsed: model said verified does not "
         "become endorsed, and summarizing untrusted web does not become "
         "trusted knowledge.",
         ["infoflow/propagation/summarization-does-not-endorse",
          "infoflow/propagation/schema-validation-does-not-endorse"]),
    rule("AF-INFOFLOW-0009", "INFOFLOW", INFOFLOW_SRC, "if-flow-kinds",
         "Data egress and decision influence are separate flow kinds, "
         "checked separately and never compared across: no data leak "
         "does not imply no decision influence, and vice versa.",
         ["infoflow/golden/data-egress-sink",
          "infoflow/golden/decision-influence-sink",
          "SUITE-INFOFLOW-KIND-SEPARATION"]),
    rule("AF-INFOFLOW-0010", "INFOFLOW", INFOFLOW_SRC, "if-sinks",
         "External-effect sinks reuse the structured EffectOperationRef "
         "by exact cross-module identity and resolve fail-closed against "
         "the explicit catalog; classification comes only from the "
         "catalog, never from the operation name, and referencing an "
         "effect is not being granted it.",
         ["infoflow/negative/unknown-sink-operation",
          "SUITE-INFOFLOW-SINK-NOT-GRANT"]),
    rule("AF-INFOFLOW-0011", "INFOFLOW", INFOFLOW_SRC, "if-sink-unclassified",
         "An unclassifiable sink yields Unproven, never an assumption of "
         "safety; sink-port 0 is rejected.",
         ["infoflow/negative/invalid-sink-port"]),
    rule("AF-INFOFLOW-0012", "INFOFLOW", INFOFLOW_SRC,
         "if-ceiling-not-authorization",
         "A declared flow ceiling is not an observed flow and not a "
         "policy approval: every judgment is ProvenWithinDeclaredCeiling, "
         "RequiresTrustedTransformation (an obligation, not an event) or "
         "Unproven - never an authorization.",
         ["SUITE-INFOFLOW-DECLARATION-NOT-AUTHORIZATION"]),
    rule("AF-INFOFLOW-0013", "INFOFLOW", INFOFLOW_SRC,
         "if-declassification-endorsement",
         "Declassification and endorsement are separate privileged "
         "transformations; ordinary tasks cannot self-declare either, "
         "and this phase records at most the obligations "
         "requires_declassification / requires_endorsement.",
         ["infoflow/negative/missing-privileged-transformation"]),
    rule("AF-INFOFLOW-0014", "INFOFLOW", INFOFLOW_SRC,
         "if-no-automatic-privilege",
         "No static transformation without an authorization proof deletes "
         "a taint or lowers a confidentiality restriction: the wire model "
         "expresses no such operation and this phase executes no "
         "declassifier or endorser.",
         ["infoflow/propagation/memory-persistence-retains-taint",
          "SUITE-INFOFLOW-NO-AUTOMATIC-PRIVILEGE"]),
    rule("AF-INFOFLOW-0015", "INFOFLOW", INFOFLOW_SRC,
         "if-persistence-not-trust",
         "Persistence does not imply trust: store, load, serialize, "
         "deserialize and summarize round-trips return data with its "
         "original taints and provenance influence.",
         ["infoflow/propagation/memory-persistence-retains-taint",
          "infoflow/propagation/summarization-does-not-endorse"]),
    rule("AF-INFOFLOW-0016", "INFOFLOW", INFOFLOW_SRC,
         "if-unknown-handling",
         "The five unknowns are distinct and never collapsed to an empty "
         "list: unknown source is the explicit unknown_external category, "
         "unknown influence is the sticky influence_unknown Top, unknown "
         "labels and sinks are validation errors, and canonicalization "
         "never repairs unknown into a lower-risk state; an explicit "
         "unknown_external source is never sufficient for a positive flow "
         "proof even when the summary claims clean influence.",
         ["infoflow/golden/unknown-influence",
          "infoflow/negative/unknown-taint-treated-as-clean",
          "infoflow/golden/unknown-external-with-clean-claim",
          "infoflow/golden/unknown-external-high-impact-decision",
          "infoflow/golden/unknown-external-explicit-taint",
          "infoflow/negative/unknown-external-clean-claim-require-proven",
          "infoflow/propagation/model-output-with-external-source",
          "infoflow/propagation/stored-external-content-not-endorsed"]),
    rule("AF-INFOFLOW-0017", "INFOFLOW", INFOFLOW_SRC, "if-non-relaxation",
         "Flow non-relaxation: F_new must not relax F_old - per kind the "
         "new ceiling set must be a subset of the old, confidentiality "
         "restrictions a superset, authority not elevated, taint and source "
         "capacity not narrowed, unknown influence not repaired; "
         "undecidable containment is Unproven and never a substitution "
         "approval.",
         ["infoflow/compatibility/new-adds-permitted-sink",
          "infoflow/compatibility/new-drops-restriction",
          "infoflow/compatibility/unknown-relationship-unproven",
          "infoflow/compatibility/flow-non-relaxation-dropped-source",
          "infoflow/compatibility/flow-non-relaxation-dropped-taint",
          "infoflow/compatibility/dropped-unknown-source-unproven"]),
    rule("AF-INFOFLOW-0018", "INFOFLOW", INFOFLOW_SRC,
         "if-canonical-ordering",
         "Canonical flow form is deterministic: resolved nominal label "
         "identities, sets sorted by identity bytes (constraints by kind "
         "and sink coordinate), duplicates rejected, declaration order "
         "never reaching the canonical ACE1 bytes.",
         ["infoflow/equivalence/label-ordering",
          "infoflow/equivalence/constraint-ordering"]),
]
RULES.extend(INFOFLOW_RULES)

# ---------------------------------------------------------------------------
# Usage rules (Phase 4A.5A, area USAGE)
# ---------------------------------------------------------------------------

USAGE_SRC = "spec/usage/usage-signature.md"

USAGE_RULES = [
    rule("AF-USAGE-0001", "USAGE", USAGE_SRC, "usage-static-scope",
         "The Usage Signature is a static declaration of additive upper "
         "bounds only: it is not a measurement, estimate, evidence, billing "
         "record, ledger reservation or runtime authorization.",
         ["SUITE-USAGE-STATIC-ONLY"]),
    rule("AF-USAGE-0002", "USAGE", USAGE_SRC, "usage-meter-identity",
         "A usage meter is resolved by the exact (catalog SpecificationId, "
         "NFC name) identity in an explicit read-only UsageMeterUniverse; "
         "unknown identities fail closed and no discovery or latest fallback "
         "exists.",
         ["usage/golden/single-finite-meter", "usage/negative/unknown-meter"]),
    rule("AF-USAGE-0003", "USAGE", USAGE_SRC, "usage-additive-only",
         "This phase supports only non-negative integer additive meters with "
         "an explicit unit identity; unsupported aggregation is rejected and "
         "different units are not combined or treated as compatible.",
         ["usage/negative/unsupported-aggregation",
          "usage/compatibility/different-meter-unit"]),
    rule("AF-USAGE-0004", "USAGE", USAGE_SRC, "usage-phase-separation",
         "Forward and recovery are independent (phase, meter) coordinates; "
         "recovery usage never refunds, overwrites or substitutes for forward "
         "usage.",
         ["usage/golden/forward-recovery-separate",
          "usage/compatibility/forward-recovery-not-interchangeable",
          "usage/algebra/compensation-no-refund"]),
    rule("AF-USAGE-0005", "USAGE", USAGE_SRC, "usage-bound-states",
         "finite(0), finite(n), unbounded and missing are distinct states; "
         "missing is never interpreted as finite(0), and unbounded is never "
         "encoded as UINT64_MAX.",
         ["usage/golden/explicit-zero", "usage/golden/unbounded",
          "usage/golden/empty-not-zero", "usage/negative/meter-omitted-not-zero"]),
    rule("AF-USAGE-0006", "USAGE", USAGE_SRC, "usage-canonical",
         "Canonical Usage Signature form resolves meters, rejects duplicate "
         "(phase, meter) keys, orders by phase number and complete meter "
         "identity bytes, and is idempotent without deleting zero limits.",
         ["usage/negative/duplicate-phase-meter",
          "usage/equivalence/limit-ordering"]),
    rule("AF-USAGE-0007", "USAGE", USAGE_SRC, "usage-accumulate",
         "Additive accumulation is checked and conservative: finite bounds "
         "add, unbounded absorbs, and an unknown operand remains unknown; "
         "overflow never wraps or saturates.",
         ["usage/algebra/accumulation-associativity",
          "usage/algebra/accumulation-zero-identity",
          "usage/algebra/unknown-propagation",
          "usage/negative/arithmetic-overflow"]),
    rule("AF-USAGE-0008", "USAGE", USAGE_SRC, "usage-branch-join",
         "Mutually exclusive branch join takes the maximum possible upper "
         "bound and is commutative, associative and idempotent; absent or "
         "unknown coordinates are not zero branches.",
         ["usage/algebra/branch-join-commutative",
          "usage/algebra/branch-join-associative",
          "usage/algebra/branch-join-idempotent"]),
    rule("AF-USAGE-0009", "USAGE", USAGE_SRC, "usage-repeat",
         "Finite repetition uses checked multiplication; repeat(0, U) "
         "produces explicit zero only for coordinates represented by U, and "
         "an unproven count cannot produce a false finite bound.",
         ["usage/algebra/repeat-zero", "usage/algebra/repeat-one",
          "usage/algebra/repeat-n", "usage/algebra/foreach-max-inflight-not-count"]),
    rule("AF-USAGE-0010", "USAGE", USAGE_SRC, "usage-static-proof",
         "Static ceiling and compatibility judgments distinguish Proven, "
         "Unproven and Violation; inability to prove a bound is never "
         "reinterpreted as compatibility or zero usage.",
         ["usage/golden/empty-not-zero", "usage/negative/meter-omitted-not-zero",
          "usage/compatibility/missing-new-meter", "SUITE-USAGE-STATIC-ONLY"]),
    rule("AF-USAGE-0011", "USAGE", USAGE_SRC, "usage-control-propagation",
         "Static control propagation is conservative: SEQUENCE and PARALLEL "
         "accumulate, SELECT joins, SCOPE preserves usage, TIMEOUT does not "
         "erase already possible usage, and AWAIT does not invent zero.",
         ["SUITE-USAGE-CONTROL"]),
    rule("AF-USAGE-0012", "USAGE", USAGE_SRC, "usage-retry-race-iterate",
         "RETRY amplifies metered usage by maximum attempts, RACE retains "
         "loser usage, and ITERATE uses only a proven total repetition bound; "
         "max_inflight is never a total iteration count.",
         ["usage/algebra/retry-amplification",
          "usage/algebra/race-loser-retained",
          "usage/algebra/foreach-max-inflight-not-count"]),
    rule("AF-USAGE-0013", "USAGE", USAGE_SRC, "usage-non-increase",
         "Usage non-increase compares the same phase, meter identity and "
         "unit; lower or equal finite ceilings and finite under old unbounded "
         "are Proven, while higher or newly unbounded ceilings are Violation "
         "and missing or incomparable coordinates are Unproven.",
         ["usage/compatibility/same-limits",
          "usage/compatibility/new-lower-limit",
          "usage/compatibility/new-higher-limit",
          "usage/compatibility/different-meter-unit",
          "usage/compatibility/missing-new-meter"]),
    rule("AF-USAGE-0014", "USAGE", USAGE_SRC, "usage-cost-boundary",
         "Usage quantity is not monetary cost; this phase has no currency, "
         "price snapshot, conversion, rounding or provider-pricing ABI.",
         ["SUITE-USAGE-STATIC-ONLY"]),
    rule("AF-USAGE-0015", "USAGE", USAGE_SRC, "usage-budget-boundary",
         "A static ceiling proof is not a BudgetReservation, Reserve, Settle, "
         "Quota Ledger or cross-run quota decision; runtime ledgers remain "
         "separate authorities.",
         ["usage/negative/quota-not-semantic-budget",
          "SUITE-USAGE-STATIC-ONLY"]),
    rule("AF-USAGE-0016", "USAGE", USAGE_SRC, "usage-no-runtime",
         "The Usage wire model and crate contain no receipt, ledger, runtime "
         "measurement, scheduler, worker, pricing API, policy engine or "
         "UsageSignatureDigest, and no HashV1 domain is added.",
         ["SUITE-USAGE-WIRE-FREEZE"]),
]
RULES.extend(USAGE_RULES)

# ---------------------------------------------------------------------------
# Cost Projection rules (Phase 4A.5B, area USAGE)
# ---------------------------------------------------------------------------

COST_SRC = "spec/usage/cost-projection.md"

COST_RULES = [
    rule("AF-USAGE-0017", "USAGE", COST_SRC, "cost-currency-identity",
         "Currency identity and atomic unit are explicit universe-resolved "
         "values; currencies never imply precision or FX and incompatible "
         "currencies cannot be combined.",
         ["cost/golden/single-currency-meter", "cost/negative/missing-currency",
          "cost/negative/different-currency"]),
    rule("AF-USAGE-0018", "USAGE", COST_SRC, "cost-money-atomic",
         "MoneyAmount is a non-negative uint64 atomic-unit count; floating "
         "point money, negative refunds and implicit conversions are absent.",
         ["cost/golden/single-currency-meter", "SUITE-COST-STATIC"]),
    rule("AF-USAGE-0019", "USAGE", COST_SRC, "cost-price-snapshot",
         "PriceTableSnapshot is complete by value, immutable for this proof, "
         "and validates every meter and unit against explicit universes; no "
         "latest provider lookup exists.",
         ["cost/golden/canonical-price-order", "cost/negative/wrong-meter-unit",
          "SUITE-COST-STATIC"]),
    rule("AF-USAGE-0020", "USAGE", COST_SRC, "cost-price-rate",
         "Only fixed non-negative rational rates with denominator > 0 are "
         "supported; tiered and progressive pricing cannot be approximated as "
         "exact cost projection.",
         ["cost/pricing/exact-integer-rate", "cost/pricing/fractional-rate",
          "cost/negative/zero-denominator", "cost/negative/unsupported-tiered"]),
    rule("AF-USAGE-0021", "USAGE", COST_SRC, "cost-rounding",
         "The supported safe rounding rule is per-meter-unit integer ceil; "
         "aggregate-once rounding is rejected because it can underestimate "
         "per-request upward rounding.",
         ["cost/pricing/per-unit-conservative-rounding",
          "cost/negative/unsupported-batch-rounding",
          "SUITE-COST-ROUNDING"]),
    rule("AF-USAGE-0022", "USAGE", COST_SRC, "cost-meter-coverage",
         "Every explicit possible Usage coordinate requires a matching price "
         "entry; missing price coverage is Unproven and never free.",
         ["cost/negative/missing-price", "cost/negative/missing-usage-not-zero",
          "cost/pricing/explicit-free-meter"]),
    rule("AF-USAGE-0023", "USAGE", COST_SRC, "cost-projection-states",
         "Static Cost Projection preserves finite, unbounded and unproven "
         "states, forward/recovery coordinates and checked arithmetic; no "
         "missing or unknown usage becomes zero.",
         ["cost/golden/explicit-zero-rate", "cost/golden/explicit-zero-usage",
          "cost/pricing/unbounded-positive", "cost/pricing/unbounded-free",
          "cost/negative/overflow"]),
    rule("AF-USAGE-0024", "USAGE", COST_SRC, "cost-static-budget",
         "Forward and recovery static cost ceilings are checked independently; "
         "forward surplus cannot subsidize recovery overspend, and the result "
         "is not reservation or admission.",
         ["cost/budget/forward-within", "cost/budget/forward-exceeds",
          "cost/budget/recovery-within", "cost/budget/recovery-exceeds",
          "cost/budget/forward-surplus-no-subsidy"]),
    rule("AF-USAGE-0025", "USAGE", COST_SRC, "cost-non-increase",
         "Cost non-increase is independent from Usage non-increase and is "
         "comparable only under equal Currency and equal complete price "
         "snapshot values.",
         ["cost/budget/cost-non-increase",
          "cost/budget/different-price-separated"]),
    rule("AF-USAGE-0026", "USAGE", COST_SRC, "cost-control-propagation",
         "Cost consumes the Phase 4A.5A Usage Envelope; it does not recount "
         "RACE, RETRY, FOREACH, HANDLE, compensation or TIMEOUT independently "
         "and does not invent zero for unresolved INVOKE/CALL_SUBFLOW.",
         ["SUITE-COST-STATIC"]),
    rule("AF-USAGE-0027", "USAGE", COST_SRC, "cost-no-runtime",
         "Cost Projection adds no price snapshot digest, cost digest, budget "
         "digest or HashV1 domain and contains no billing, ledger, provider, "
         "FX, reservation, settlement, scheduler or runtime object.",
         ["SUITE-COST-WIRE-FREEZE"]),
]
RULES.extend(COST_RULES)

# ---------------------------------------------------------------------------
# Recovery rules (Phase 4A.6, area RECOVERY)
# ---------------------------------------------------------------------------

RECOVERY_SRC = "spec/recovery/recovery-contract.md"

RECOVERY_RULES = [
    rule("AF-RECOVERY-0001", "RECOVERY", RECOVERY_SRC, "recovery-static-boundary",
         "Recovery Contract is a static declaration and proof layer only: it "
         "does not create receipts, grants, keys, runtime state or execution.",
         ["SUITE-RECOVERY-STATIC"]),
    rule("AF-RECOVERY-0002", "RECOVERY", RECOVERY_SRC, "recovery-replay-modes",
         "ReplaySemantics is closed at recompute, recorded, reconcile and "
         "forbidden with distinct static obligations.",
         ["recovery/golden/pure-recorded", "recovery/golden/recompute",
          "recovery/golden/forbidden-replay"]),
    rule("AF-RECOVERY-0003", "RECOVERY", RECOVERY_SRC, "recovery-replay-not-authority",
         "Replay declaration is not authority, safe retry permission, runtime "
         "evidence or historical/fork equivalence.",
         ["recovery/replay/historical-no-dispatch", "recovery/replay/fork-fresh-admission",
          "SUITE-RECOVERY-STATIC"]),
    rule("AF-RECOVERY-0004", "RECOVERY", RECOVERY_SRC, "recovery-idempotency",
         "IdempotencySemantics is closed at natural, keyed, none and reconciled; "
         "each is a declaration rather than provider proof.",
         ["recovery/golden/natural-idempotency", "recovery/golden/keyed-idempotency",
          "recovery/golden/reconciled-idempotency"]),
    rule("AF-RECOVERY-0005", "RECOVERY", RECOVERY_SRC, "recovery-key-obligations",
         "KEYED recovery carries explicit key stability obligations and never "
         "generates an IdempotencyKey or durable Invocation/Attempt identity.",
         ["recovery/golden/keyed-idempotency", "recovery/negative/keyed-without-evidence"]),
    rule("AF-RECOVERY-0006", "RECOVERY", RECOVERY_SRC, "recovery-effect-identity",
         "Effect recovery clauses use the complete canonical EffectAtom identity "
         "including the full selector and qualifier family, member discriminant "
         "and semantic payload; family alone cannot match and duplicates reject.",
         ["recovery/golden/effect-specific-clause", "recovery/negative/duplicate-effect-clause",
          "recovery/equivalence/effect-clause-order",
          "recovery/golden/recovery-exact-qualifier-match",
          "recovery/golden/recovery-natural-none-distinct-clauses",
          "recovery/golden/recovery-clause-complete-atom-identity",
          "recovery/negative/recovery-natural-vs-none-mismatch",
          "recovery/negative/recovery-none-vs-natural-mismatch",
          "recovery/negative/recovery-different-resource-selector",
          "recovery/negative/recovery-different-scope-binding",
          "recovery/negative/recovery-proof-no-false-proven",
          "recovery/equivalence/qualifier-member-order"]),
    rule("AF-RECOVERY-0007", "RECOVERY", RECOVERY_SRC, "recovery-coverage",
         "Every known recoverable Effect requires an explicit matching clause; "
         "unknown/open effect coverage is not assumed safe.",
         ["recovery/negative/undeclared-effect-identity",
          "recovery/negative/unknown-effect-coverage"]),
    rule("AF-RECOVERY-0008", "RECOVERY", RECOVERY_SRC, "recovery-reconciliation",
         "Reconciliation requirement declares capability/evidence obligations; "
         "it never creates APPLIED, NOT_APPLIED, PARTIALLY_APPLIED or STILL_UNKNOWN "
         "runtime evidence.",
         ["recovery/golden/reconciled-idempotency", "recovery/negative/reconciled-without-result",
          "recovery/reconciliation/not-applied-evidence"]),
    rule("AF-RECOVERY-0009", "RECOVERY", RECOVERY_SRC, "recovery-unknown-effect",
         "Unknown effects may only block or require reconciliation; UNKNOWN is "
         "never blindly retried and PARTIALLY_APPLIED is never coerced to another result.",
         ["recovery/negative/none-unknown-blind-retry", "recovery/reconciliation/still-unknown-blocked",
          "recovery/reconciliation/partially-applied"]),
    rule("AF-RECOVERY-0010", "RECOVERY", RECOVERY_SRC, "recovery-compensation",
         "Compensation is a new explicit Effect operation obligation, not an "
         "execution grant, rollback API or hidden method.",
         ["recovery/golden/compensation-requirement", "recovery/negative/unknown-recovery-operation"]),
    rule("AF-RECOVERY-0011", "RECOVERY", RECOVERY_SRC, "recovery-forward-history",
         "Compensation preserves forward Effect history and never subtracts "
         "forward Usage/Cost or erases Failure/Cancelled/Timeout facts.",
         ["recovery/compensation/forward-history-preserved",
          "recovery/compensation/recovery-usage-not-refund"]),
    rule("AF-RECOVERY-0012", "RECOVERY", RECOVERY_SRC, "recovery-static-proof",
         "Static safety returns ProvenStaticConstraint, RequiresEvidence, "
         "Unproven or Violation; Proven is never runtime retry authorization.",
         ["recovery/golden/pure-recorded", "recovery/negative/keyed-without-evidence",
          "recovery/negative/reconciled-without-result", "SUITE-RECOVERY-PROOF"]),
    rule("AF-RECOVERY-0013", "RECOVERY", RECOVERY_SRC, "recovery-compatibility",
         "Recovery compatibility is conservative: exact-equivalent declarations "
         "are proven, removed capability is weakened and other mode changes are "
         "Unproven; enum order is not a strength order.",
         ["recovery/compatibility/identical", "recovery/compatibility/clause-removed",
          "recovery/compatibility/keyed-not-natural"]),
    rule("AF-RECOVERY-0014", "RECOVERY", RECOVERY_SRC, "recovery-cross-dimensions",
         "K-to-ε/Φ/U correspondence is explicit future integration; RecoveryBlocked "
         "is not a business FailureValue and no temporary ComponentContract exists.",
         ["SUITE-RECOVERY-STATIC"]),
    rule("AF-RECOVERY-0015", "RECOVERY", RECOVERY_SRC, "recovery-no-runtime",
         "Recovery adds no Receipt, Grant, Reconciliation execution, Compensation "
         "execution, journal, runtime, scheduler, worker or Recovery digest/domain.",
         ["SUITE-RECOVERY-WIRE-FREEZE"]),
]
RULES.extend(RECOVERY_RULES)

# ---------------------------------------------------------------------------
# Execution Requirements rules (Phase 4A.7, area EXECUTION)
# ---------------------------------------------------------------------------

EXECUTION_SRC = "spec/execution/execution-requirements.md"

EXECUTION_RULES = [
    rule("AF-EXECUTION-0001", "EXECUTION", EXECUTION_SRC,
         "execution-static-boundary",
         "Execution Requirements is a static declaration/proof layer only: it "
         "contains no worker identity, endpoint, lease, credential, heartbeat, "
         "live load, live price, reservation, scheduler state, placement result "
         "or HashV1 digest.",
         ["SUITE-EXECUTION-STATIC"]),
    rule("AF-EXECUTION-0002", "EXECUTION", EXECUTION_SRC,
         "execution-hard-soft-separation",
         "Hard constraints are conjunctive eligibility conditions; closed soft "
         "preferences never compensate for a hard rejection or unproven fact.",
         ["execution/golden/hard-and-soft",
          "execution/matching/soft-preferences-do-not-bypass-hard",
          "SUITE-EXECUTION-MATCHING"]),
    rule("AF-EXECUTION-0003", "EXECUTION", EXECUTION_SRC,
         "execution-capability-identity",
         "Capability matching uses the complete explicit catalog/name identity "
         "and an explicitly recognized exact version; unknown capability or "
         "version fails closed.",
         ["execution/golden/single-capability",
          "execution/golden/multiple-capabilities",
          "execution/negative/unknown-capability",
          "execution/negative/wrong-version"]),
    rule("AF-EXECUTION-0004", "EXECUTION", EXECUTION_SRC,
         "execution-capability-attributes",
         "Capability attributes are NFC-normalized, non-empty, sorted and "
         "checked against the explicit vocabulary; unknown attributes are not "
         "silently compatible.",
         ["execution/negative/invalid-capability-attribute",
          "execution/negative/unknown-capability-attribute"]),
    rule("AF-EXECUTION-0005", "EXECUTION", EXECUTION_SRC,
         "execution-quantity-units",
         "CPU millicores, memory bytes and GPU device count are independent "
         "unsigned units; quantity capacity is nominal, checked only within a "
         "unit, and missing evidence is unproven rather than zero.",
         ["execution/golden/quantities",
          "execution/golden/gpu-capability-quantity",
          "execution/matching/missing-quantity-unproven",
          "execution/negative/invalid-quantity-representation",
          "execution/negative/quantity-unit-mismatch"]),
    rule("AF-EXECUTION-0006", "EXECUTION", EXECUTION_SRC,
         "execution-sandbox-assurance",
         "Sandbox assurance is a closed basic/strong hard requirement; a weak "
         "descriptor rejects and an absent descriptor is unproven, while the "
         "level is not enforcement or attestation.",
          ["execution/golden/sandbox-trust-region-runtime",
          "execution/matching/sandbox-weak",
          "execution/negative/unknown-sandbox-profile"]),
    rule("AF-EXECUTION-0007", "EXECUTION", EXECUTION_SRC,
         "execution-trust-domain",
         "Trust domains are exact nominal identities resolved from an explicit "
         "universe; matching never infers provider, parent, identity, "
         "delegation or authority equivalence.",
         ["execution/golden/sandbox-trust-region-runtime",
          "execution/matching/trust-mismatch",
          "execution/negative/unknown-trust"]),
    rule("AF-EXECUTION-0008", "EXECUTION", EXECUTION_SRC,
         "execution-region-residency",
         "Execution region and data residency are separate hard constraints; "
         "nominal region identities require explicit resolution and never use "
         "prefix, geography or provider inference.",
         ["execution/golden/sandbox-trust-region-runtime",
          "execution/matching/region-exact",
          "execution/matching/residency-exact",
          "execution/matching/residency-violation",
          "execution/negative/unknown-region"]),
    rule("AF-EXECUTION-0009", "EXECUTION", EXECUTION_SRC,
         "execution-runtime-abi",
         "Runtime profile and implementation ABI requirements use exact explicit "
         "identity/feature/digest pins; missing or incompatible evidence is not "
         "implicitly compatible.",
         ["execution/golden/runtime-and-abi",
          "execution/matching/runtime-mismatch",
          "execution/matching/abi-unproven",
          "execution/negative/unknown-runtime"]),
    rule("AF-EXECUTION-0010", "EXECUTION", EXECUTION_SRC,
         "execution-canonicalization",
         "Canonicalization resolves explicit universes, rejects duplicate hard "
         "requirements, sorts unordered dimensions deterministically and is "
         "idempotent; no requirement is silently relaxed.",
         ["execution/equivalence/capability-order",
          "execution/equivalence/region-order",
          "execution/equivalence/preference-order"]),
    rule("AF-EXECUTION-0011", "EXECUTION", EXECUTION_SRC,
         "execution-profile-matching",
         "Static profile matching checks every hard constraint conjunctively and "
         "returns only descriptor proof, rejection or unproven evidence; it is "
         "not availability, admission or placement.",
         ["execution/matching/all-hard-satisfied",
          "execution/matching/hard-capability-rejected",
          "execution/matching/abi-unproven",
          "SUITE-EXECUTION-MATCHING"]),
    rule("AF-EXECUTION-0012", "EXECUTION", EXECUTION_SRC,
         "execution-feasibility-direction",
         "Feasibility compares old eligible profiles as a subset of new eligible "
         "profiles; added requirements or narrowed regions are not deployment "
         "compatible by default.",
         ["execution/compatibility/reduces-memory",
          "execution/compatibility/increases-memory",
          "execution/compatibility/tightens-residency",
          "execution/compatibility/adds-capability",
          "execution/compatibility/removes-capability"]),
    rule("AF-EXECUTION-0013", "EXECUTION", EXECUTION_SRC,
         "execution-security-preservation",
         "A feasibility expansion does not prove security preservation; sandbox, "
         "trust and residency relaxation is independently reported as a security "
         "violation or unproven relation.",
         ["execution/compatibility/weakens-sandbox",
          "execution/compatibility/expands-residency",
          "execution/compatibility/trust-change"]),
    rule("AF-EXECUTION-0014", "EXECUTION", EXECUTION_SRC,
         "execution-combined-compatibility",
         "Combined compatibility requires both independent feasibility and "
         "security verdicts and never hides their directions in one boolean.",
         ["execution/compatibility/identical",
          "execution/compatibility/strengthens-sandbox",
          "execution/compatibility/version-change"]),
    rule("AF-EXECUTION-0015", "EXECUTION", EXECUTION_SRC,
         "execution-no-authority-or-aggregation",
         "X does not create execution authority and does not blindly aggregate "
         "control-tree requirements into a fictitious worker or scheduler "
         "surface; cross-dimension composition is deferred.",
         ["SUITE-EXECUTION-STATIC"]),
]
RULES.extend(EXECUTION_RULES)

# Component Contract rules (Phase 4B.1, area CONTRACT)
COMPONENT_SRC = "spec/component/component-contract.md"
RELATION_SRC = "spec/component/relation-catalogs.md"
COMPONENT_RULES = [
    rule("AF-CONTRACT-0001", "CONTRACT", COMPONENT_SRC, "component-contract-closed-abi",
         "ComponentContract is a closed thirteen-field semantic body with exact imports and no arbitrary extension field.",
         ["SUITE-COMPONENT-CONTRACT-WIRE-FREEZE"]),
    rule("AF-CONTRACT-0002", "CONTRACT", COMPONENT_SRC, "component-contract-identity-digest",
         "Exact ComponentIdentity enters semantic identity while publication version and implementation metadata remain outside.",
         ["SUITE-COMPONENT-CONTRACT-CANONICAL"]),
    rule("AF-CONTRACT-0003", "CONTRACT", COMPONENT_SRC, "component-contract-canonical-digest",
         "Only a validator-created checked body may be hashed with the existing component-contract HashV1 domain.",
         ["SUITE-COMPONENT-CONTRACT-CANONICAL"]),
    rule("AF-CONTRACT-0004", "CONTRACT", COMPONENT_SRC, "component-contract-ports",
         "Public I/O ports use independent positive namespaces and exact nominal TypeSchemaRef pins.",
         ["SUITE-COMPONENT-CONTRACT-STATIC", "SUITE-COMPONENT-CONTRACT-COMPATIBILITY"]),
    rule("AF-CONTRACT-0005", "CONTRACT", COMPONENT_SRC, "component-contract-cross-dimension",
         "The Failure row remains a separate exact-pinned proof dimension in the assembled contract.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0006", "CONTRACT", COMPONENT_SRC, "component-contract-effect-resource",
         "Effect operations and exact resource types require explicit complete relation coverage.",
         ["SUITE-COMPONENT-RELATION-CATALOGS", "SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0007", "CONTRACT", COMPONENT_SRC, "component-contract-effect-selector-mapping",
         "Effect selectors bind only to declared resource ports and exact mapped resource attributes.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0008", "CONTRACT", COMPONENT_SRC, "component-contract-flow-ports",
         "Information-flow sources and sinks resolve in their exact input, resource and output namespaces.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0009", "CONTRACT", COMPONENT_SRC, "component-contract-recovery-usage",
         "Usage ceilings remain exact additive declarations and missing recovery coordinates are not zero.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0010", "CONTRACT", COMPONENT_SRC, "component-contract-recovery-bindings",
         "Recovery actions bind complete forward/action EffectAtom identities and form an acyclic dependency graph.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0011", "CONTRACT", COMPONENT_SRC, "component-contract-execution",
         "Execution constraints are resolved against explicit capability and profile universes.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0012", "CONTRACT", COMPONENT_SRC, "component-contract-flow-unknown",
         "Unknown provenance or influence never establishes an unjustified positive flow proof.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0013", "CONTRACT", COMPONENT_SRC, "component-contract-effect-resource-closure",
         "An effect selector requires a declared resource port, exact resource pin, full qualifier profile and closed operation support.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0014", "CONTRACT", COMPONENT_SRC, "component-contract-flow-partitions",
         "EffectFlowPartitions retain full atom identity and only narrow an existing operation-level flow ceiling.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0015", "CONTRACT", COMPONENT_SRC, "component-contract-recovery-action-coverage",
         "Every required K action has exactly one exact ε binding and no undeclared action is accepted.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0016", "CONTRACT", COMPONENT_SRC, "component-contract-recovery-usage-checked",
         "Recovery meter coverage is exhaustive and activation, attempt and usage bounds use checked arithmetic.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0017", "CONTRACT", COMPONENT_SRC, "component-contract-proof-composition",
         "Any non-Proven required dimension prevents a Proven aggregate contract verdict.",
         ["SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0018", "CONTRACT", RELATION_SRC, "flow-execution-catalog-exact-relations",
         "F/X relations match exact labels, taints, authority levels, flow kinds, sinks, regions and trust domains.",
         ["SUITE-COMPONENT-RELATION-CATALOGS", "SUITE-COMPONENT-CONTRACT-STATIC"]),
    rule("AF-CONTRACT-0019", "CONTRACT", RELATION_SRC, "component-relation-catalogs-snapshots",
         "Proof context pins canonical relation content and changed snapshots invalidate prior proof results.",
         ["SUITE-COMPONENT-RELATION-CATALOGS"]),
    rule("AF-CONTRACT-0020", "CONTRACT", COMPONENT_SRC, "component-contract-substitution",
         "Substitution uses exact I/O pins and existing directional dimension proofs; unsupported refinements remain Unproven.",
         ["SUITE-COMPONENT-CONTRACT-COMPATIBILITY"]),
    rule("AF-CONTRACT-0021", "CONTRACT", COMPONENT_SRC, "component-contract-static-authority-boundary",
         "Static compatibility and relation proofs never create runtime authority, worker attestation or execution authorization.",
         ["SUITE-COMPONENT-CONTRACT-WIRE-FREEZE"]),
    rule("AF-CONTRACT-0022", "CONTRACT", COMPONENT_SRC, "component-contract-recovery-action-guard",
         "Each K action possibility must be contained by the exact declared action epsilon guard under frozen containment; a Violation requires a validator-created type-valid reachable countermodel, otherwise the relation remains Unproven, and no guard proof creates receipt eligibility.",
         ["computation-root-recovery/audit/AUDIT-10", "computation-root-recovery/guard/G03", "computation-root-recovery/guard/G05", "computation-root-recovery/guard/G06", "computation-root-recovery/guard/G15", "computation-root-recovery/guard/G16", "SUITE-COMPONENT-CONTRACT-STATIC", "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE"]),
]
RULES.extend(COMPONENT_RULES)

PORTABLE_PROOF_SRC = "spec/portable/portable-proof-inputs.md"
PORTABLE_PROOF_RULES = [
    ("ppi-closed-schema", "The registered Bundle is a closed ten-field typed record; every member is required and IDs retain local namespaces."),
    ("ppi-exact-scalars", "Portable machine scalars use exact ACE1 widths and representations; Int64 and Decimal never pass through binary floating point."),
    ("ppi-presence", "Required fields, None, Some(empty), Complete(empty) and Unavailable retain their distinct states."),
    ("ppi-canonical-bytes", "Accepted bytes exactly equal schema-directed ACE1 re-encoding; decoders reject noncanonical input instead of repairing it."),
    ("ppi-set-order", "Declared sets use their registered typed order; duplicate values and conflicting identities fail at their specified stages."),
    ("ppi-scoped-ids", "Local IDs resolve in their own table namespace and are never renumbered during decoding."),
    ("ppi-assets-inline", "All required content is inline and each reference retains its digest domain and exact nominal schema pin."),
    ("ppi-reference-closure", "D2 resolves root-scoped references and rejects dangling, conflicting or unexplained transport rows."),
    ("ppi-finite-graphs", "Structural-type and assertion graphs are bounded, finite and acyclic before expansion."),
    ("ppi-associations", "Resolver source, target, export and dependency associations retain their complete scoped typed identities."),
    ("ppi-context-scopes", "Context and nested-universe member scopes are independent; D0-D4 never fills one scope from another."),
    ("ppi-schema-content", "Schema assets use the existing canonical TypeSchema grammar and type-schema HashV1 domain."),
    ("ppi-ir-contract-content", "IR and ComponentContract bytes use their existing exact canonical encodings and HashV1 domains; raw assets do not mint Checked wrappers."),
    ("ppi-checksum", "Registry, normative-source and other raw-file checksums are plain SHA-256 and are never confused with HashV1."),
    ("ppi-source-registry", "Compiler inputs bind complete inline registries, closed tables and all referenced normative source bytes."),
    ("ppi-current-profile", "D4 accepts only the exact implementation-owned current static semantic content tuple."),
    ("ppi-role-boundary", "All ten semantic roles are represented and Lowering/Runtime remain ModelMissing in the static profile."),
    ("ppi-attribution", "Only full-content Profile A is supported; its 84 applied rules do not assert complete dependency closure."),
    ("ppi-no-upcast", "Unknown or unsupported semantic content fails closed without implicit upcast or label substitution."),
    ("ppi-no-capability", "D0-D4 output is raw content validation and cannot mint checked contract, IR, resolver or RootExport capabilities."),
    ("ppi-resource-limits", "Decoder work and allocation limits yield ResourceLimit without truncation or partial positive results."),
    ("ppi-independent-conformance", "Production bytes and stage results agree with an independent Python ACE1/schema oracle that does not consume Rust Actual results."),
    ("ppi-additive-freeze", "Portable registrations are additive; existing ABI meanings, schemas, HashV1 domains and frozen digests retain their identities."),
    ("ppi-mutation-evidence", "Comparator faults and malformed-input cases are separately detected by production conformance.")
]
for number, (anchor, statement) in enumerate(PORTABLE_PROOF_RULES, 173):
    CASE_FOR_RULE = {
        "ppi-closed-schema": "positive/full-ten-field-d4",
        "ppi-exact-scalars": "d4-host-width-scalar-remains-uint64",
        "ppi-presence": "positive-none-and-empty-distinct",
        "ppi-canonical-bytes": "d1-unsorted-schema-set",
        "ppi-set-order": "d1-duplicate-set-member",
        "ppi-scoped-ids": "d2-dangling-context",
        "ppi-assets-inline": "d2-dangling-schema",
        "ppi-reference-closure": "d2-dangling-root",
        "ppi-finite-graphs": "d2-structural-cycle",
        "ppi-associations": "d2-assertion-cycle",
        "ppi-context-scopes": "positive/full-ten-field-d4",
        "ppi-schema-content": "d3-invalid-schema-blob",
        "ppi-ir-contract-content": "d3-corrupt-ir-under-old-pin",
        "ppi-checksum": "d3-source-plain-checksum",
        "ppi-source-registry": "d3-registry-checksum-mismatch",
        "ppi-current-profile": "d4-registry-bytes-conflict",
        "ppi-role-boundary": "d4-missing-runtime-model",
        "ppi-attribution": "d4-profile-b-unsupported",
        "ppi-no-upcast": "d4-version-label-change",
        "ppi-no-capability": "positive/profile-a-exact",
        "ppi-resource-limits": "d0-invalid-width-type",
        "ppi-independent-conformance": "positive/full-ten-field-d4",
        "ppi-additive-freeze": "positive/full-ten-field-d4",
        "ppi-mutation-evidence": "d1-duplicate-set-member",
    }[anchor]
    RULES.append(rule(
        f"AF-SEMANTIC-{number:04}", "SEMANTIC", PORTABLE_PROOF_SRC, anchor,
        statement, [
            f"portable-proof-codec/{CASE_FOR_RULE}",
            "portable-proof-codec/positive/full-ten-field-d4",
            "SUITE-PORTABLE-PROOF-CODEC",
        ]))




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
    ("agentflow.component.contract@1", "schema",
     "spec/idl/component-contract.afidl",
     "Closed thirteen-field ComponentContract ABI using exact pins to the existing semantic dimension schemas."),
    ("agentflow.component.contract.semantics@1", "schema_language",
     "spec/component/component-contract.md",
     "Component identity and digest scope, exact public port ABI, cross-dimension static proof, typed K-to-epsilon guard compatibility, and substitution limits."),
    ("agentflow.component.relation-catalogs.semantics@1", "schema_language",
     "spec/component/relation-catalogs.md",
     "Exact versioned relation catalog snapshots and content-bound static proof assumptions."),
    ("agentflow.effect.resource.compatibility@1", "schema",
     "spec/idl/effect-resource-compatibility.afidl",
     "Exact operation/resource-type compatibility relation, selector mappings and complete qualifier profiles."),
    ("agentflow.flow.execution.relation@1", "schema",
     "spec/idl/flow-execution-relation.afidl",
     "Exact information-flow and execution region, trust, sandbox and sink relation."),
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
    ("agentflow.effect.signature@1", "schema",
     "spec/idl/effect-signature.afidl",
     "Normative shape of the Effect Signature: effect atoms (operation "
     "identity, logical resource selector, closed qualifiers), explicit "
     "occurrence bounds, the closed guard language and the four capability "
     "row states (pure, closed, bounded-open, unknown). Reuses the semantic "
     "core's PrimitiveConstant by exact cross-module import."),
    ("agentflow.effect.signature.semantics@1", "schema_language",
     "spec/effect/effect-signature.md",
     "Effect Signature semantics: the declaration/authorization boundary, "
     "atom identity, fail-closed catalog resolution, occurrence bound and "
     "guard rules, the closed/open/top row distinction, the effect algebra "
     "(containment, accumulation, branch join, repetition) and static "
     "control-opcode propagation."),
    ("agentflow.failure.row@1", "schema",
     "spec/idl/failure-row.afidl",
     "Normative shape of the static Typed Failure Row: the nominal "
     "FailureTypeRef (an exact TypeSchemaRef pin reused from the semantic "
     "core) and the closed FailureRow set. Only typed business failures "
     "enter the row; control outcomes, recovery states, infrastructure "
     "facts and incidents are structurally outside it."),
    ("agentflow.failure.row.semantics@1", "schema_language",
     "spec/failure/failure-row.md",
     "Failure Row semantics: the six error planes, nominal identity and "
     "exact pinning, the closed-row and empty-not-success rules, "
     "canonical ordering, exact containment, the union algebra, HANDLE "
     "residual and handler propagation, control propagation, and the "
     "Unproven analysis result."),
    ("agentflow.information.flow@1", "schema",
     "spec/idl/information-flow.afidl",
     "Normative shape of the static Information Flow Contract: "
     "confidentiality and taint labels as separate nominal types, the "
     "closed instruction-authority scale, typed logical source "
     "categories, the conservative static influence summary, "
     "data-egress vs decision-influence flow kinds, workflow-output and "
     "structured effect-operation sinks, and the closed flow contract. "
     "The external effect sink reuses the effect signature's "
     "EffectOperationRef by exact cross-module import."),
    ("agentflow.information.flow.semantics@1", "schema_language",
     "spec/information-flow/information-flow.md",
     "Information Flow semantics: the four-concept separation "
     "(provenance/trust/taint/flow), the frozen security invariants, "
     "label and taint identity, instruction authority with no "
     "self-elevation, conservative may-influence propagation, the LLM "
     "strong taint mixer, the egress/influence split, "
     "declassification/endorsement obligations, persistence without "
     "trust upgrade, unknown handling, flow non-relaxation and the "
     "static-vs-runtime boundary."),
    ("agentflow.usage.signature@1", "schema",
     "spec/idl/usage-signature.afidl",
     "Normative shape of the static Usage Signature: exact additive meter "
     "identity, forward/recovery phases, finite/unbounded upper bounds and "
     "the closed Usage Signature declaration."),
    ("agentflow.usage.signature.semantics@1", "schema_language",
     "spec/usage/usage-signature.md",
     "Static Usage semantics: explicit meter universe and units, additive "
     "upper bounds, phase separation, canonicalization, conservative usage "
     "algebra and control propagation, static ceiling proof, non-increase "
     "compatibility and the runtime ledger boundary."),
    ("agentflow.usage.cost@1", "schema",
     "spec/idl/cost-projection.afidl",
     "Normative shape of the static Cost Projection declaration: explicit "
     "currency and atomic money, rational fixed price entries, immutable "
     "price table snapshot and forward/recovery static cost ceiling."),
    ("agentflow.usage.cost.semantics@1", "schema_language",
     "spec/usage/cost-projection.md",
     "Static Cost Projection semantics: currency identity, atomic units, "
     "fixed rational pricing, conservative per-meter-unit rounding, meter "
     "coverage, static cost and budget proofs, cost non-increase and the "
     "runtime billing boundary."),
    ("agentflow.recovery.contract@1", "schema",
     "spec/idl/recovery-contract.afidl",
     "Normative shape of the static Recovery Contract: replay and idempotency "
     "semantics, key obligations, exact EffectAtom recovery clauses, "
     "reconciliation requirements, unknown-effect handling and compensation "
     "obligations."),
    ("agentflow.recovery.usage.coverage@1", "schema",
     "spec/idl/recovery-usage-coverage.afidl",
     "Exhaustive exact recovery-action to usage-meter coverage catalog."),
    ("agentflow.recovery.contract.semantics@1", "schema_language",
     "spec/recovery/recovery-contract.md",
     "Static Recovery semantics: replay boundaries, idempotency declarations, "
     "unknown and partial effect safety, reconciliation/compensation obligations, "
     "forward-history preservation, static proofs and conservative compatibility."),
    ("agentflow.execution.requirements@1", "schema",
     "spec/idl/execution-requirements.afidl",
     "Normative shape of static Execution Requirements: exact capability and "
     "attribute requirements, independent nominal compute quantities, sandbox, "
     "trust, region/residency, runtime profile, implementation ABI obligations "
     "and closed soft preferences."),
    ("agentflow.execution.requirements.semantics@1", "schema_language",
     "spec/execution/execution-requirements.md",
     "Static X semantics: hard/soft separation, explicit capability/profile "
     "universes, canonicalization, descriptor matching, separate feasibility "
     "and security-preservation proofs, compatibility and the no-authority "
     "boundary."),
    ("agentflow.registry.core@1", "registry",
     "spec/registry/registry-spec.md",
     "Registry mechanics: Specification Registry, RuleId system, Stable ABI "
     "Number Registry, ConformanceCaseId and canonical registry form."),
    ("agentflow.registry.rule-areas@1", "closed_table",
     "spec/tables/rule-areas.json",
     "Closed table of RuleId areas."),
    ("agentflow.resource.requirement@1", "schema",
     "spec/idl/resource-requirement.afidl",
     "Normative shape of the static Resource Requirement Row: the nominal "
     "ResourceTypeRef (an exact TypeSchemaRef pin reused from the semantic "
     "core), the exact-value ResourceAttributeConstraint, the port-keyed "
     "ResourceRequirement and the closed ResourceRequirementRow. Only "
     "static need is declared here; binding, handles, adapters and "
     "authority are structurally outside."),
    ("agentflow.resource.requirement.semantics@1", "schema_language",
     "spec/resource/resource-requirement.md",
     "Resource Requirement semantics: nominal identity and exact pinning, "
     "the explicit resource-type universe, the requirement port namespace, "
     "exact-value attributes, the closed row, canonical ordering, static "
     "satisfaction, requirement non-strengthening, the Unproven result, "
     "the requirement/capability/grant distinction and the deferred "
     "binding/handle boundary."),
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
    ("agentflow.semantic.leaf-contract-resolution.semantics@1", "schema_language",
     "spec/computation/leaf-contract-resolution.md",
     "Exact, source-preserving static contract resolution for INVOKE, AWAIT "
     "and CALL_SUBFLOW computation leaves; no workflow control algebra or "
     "runtime authority."),
    ("agentflow.semantic.control-contract-algebra.semantics@1", "schema_language",
     "spec/computation/control-contract-algebra.md",
     "Conservative, provenance-preserving static composition for SEQUENCE, "
     "SELECT, PARALLEL and RACE without workflow ABI or runtime authority."),
    ("agentflow.semantic.advanced-control-contract-algebra.semantics@1", "schema_language",
     "spec/computation/advanced-control-contract-algebra.md",
     "Derived-only static propagation for existing ITERATE, RETRY, TIMEOUT and HANDLE opcodes without changing Semantic IR or ComponentContract ABI."),
    ("agentflow.semantic.scope-compensation-contract-algebra.semantics@1", "schema_language",
     "spec/computation/scope-compensation-contract-algebra.md",
     "Derived-only SCOPE and COMPENSATION_SCOPE algebra, checked full control-tree closure, and root-relative K action/Recovery-Usage proof with runtime authority, receipts, export assertions, reservations, and execution explicitly outside this static proof boundary."),
    ("agentflow.computation.root-export-contract-proof.semantics@1", "schema_language",
     "spec/computation/root-export-contract-proof.md",
     "Pure in-process RootExportAssertion proof: exact typed coordinates and nine-dimension static root/export compatibility, with body, publisher, authority, budget and runtime evidence kept separate."),
    ("agentflow.semantic.types@1", "schema_language",
     "spec/semantic/core-semantic-types.md",
     "Workflow semantic core semantics: machine vs workflow primitives, "
     "type DAG rules, exact pinning, SSA value slots, expression purity and "
     "type checking, resource slot scope."),
    ("agentflow.portable.proof.inputs@1", "schema",
     "spec/idl/portable-proof-inputs.afidl",
     "Closed ten-field portable static proof Bundle and its typed inline content, context, resolver, compiler and assertion records."),
    ("agentflow.portable.proof.inputs.semantics@1", "schema_language",
     "spec/portable/portable-proof-inputs.md",
     "ACE1 portable proof input semantics, closed D0-D4 validation, exact content identity, scoped references and current static semantic support."),
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
    "spec/idl/effect-signature.afidl",
    "spec/idl/failure-row.afidl",
    "spec/idl/resource-requirement.afidl",
    "spec/idl/information-flow.afidl",
    "spec/idl/usage-signature.afidl",
    "spec/idl/cost-projection.afidl",
    "spec/idl/recovery-contract.afidl",
    "spec/idl/execution-requirements.afidl",
    "spec/idl/component-contract.afidl",
    "spec/idl/effect-resource-compatibility.afidl",
    "spec/idl/recovery-usage-coverage.afidl",
    "spec/idl/flow-execution-relation.afidl",
    "spec/idl/portable-proof-inputs.afidl",
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
        {
            "suite_id": "SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION",
            "description": (
                "The five non-failure planes are structurally outside the "
                "Failure Row: no TimedOut/Cancelled/RecoveryBlocked/"
                "TaskAttemptLost/RuntimeIncident failure type exists "
                "anywhere in af-failure, the frozen HandleMatch carries "
                "timed_out as a payload-less unit case beside the typed "
                "failure matcher, and TIMEOUT/AWAIT compositions add no "
                "type. Covers the static absence audit and the read-only "
                "HandleMatch consistency check."
            ),
            "targets": [
                "af-failure static surface audit (no control-outcome failure types)",
                "spec/idl/semantic-ir.afidl HandleMatch (read-only model check)",
                "spec/conformance/failure-row/algebra/",
            ],
        },
        {
            "suite_id": "SUITE-FAILURE-DEFERRED-CONTRACT-RESOLUTION",
            "description": (
                "INVOKE and CALL_SUBFLOW failure propagation is deferred "
                "to the ComponentContract stage: the analysis-level "
                "Unproven envelope fails closed (failure_row_unproven) "
                "and a missing explicit FailureRow is never assumed "
                "empty. Covers the Unproven corpus case and the static "
                "surface."
            ),
            "targets": [
                "af-failure StaticFailureEnvelope::require_row",
                "spec/conformance/failure-row/negative/unproven-failure-containment",
            ],
        },
        {
            "suite_id": "SUITE-FAILURE-NOMINAL-NO-STRING-MATCHING",
            "description": (
                "Failure matching is exact nominal identity only: no "
                "error-message or error-code substring matching, no "
                "exception class-name reflection, no provider-specific "
                "mapping, and no same-short-name or same-shape "
                "containment inference. Covers the static audit over the "
                "crate source and the same-short-name corpus."
            ),
            "targets": [
                "af-failure static surface audit (no string/reflection matching)",
                "spec/conformance/failure-row/golden/same-short-name-different-family",
            ],
        },
        {
            "suite_id": "SUITE-FAILURE-NO-RUNTIME",
            "description": (
                "The failure layer is static only: no af-ir, af-effect "
                "or af-spec dependency, no I/O, clock, randomness, "
                "async runtime, worker transport or mapping adapter; "
                "propagation is a pure function over rows. Covers the "
                "crate dependency audit and the source surface."
            ),
            "targets": [
                "af-failure crate dependency set (af-core, af-idl, af-semantic)",
                "af-failure static surface audit (no runtime/journal/plan symbols)",
            ],
        },
        {
            "suite_id": "SUITE-FAILURE-WIRE-FREEZE",
            "description": (
                "Phase 4A.2 added exactly one AFIDL module and no HashV1 "
                "domain: no FailureRowDigest exists, semantic-ir.afidl "
                "stays byte-identical, the effect corpus stays at "
                "revision 2, and every pre-existing corpus digest is "
                "unchanged. Covers the static surface audits and the "
                "zero-drift corpus runs."
            ),
            "targets": [
                "af-failure static surface audit (no FailureRowDigest)",
                "af-ir wire-freeze audit (module/ABI/domain counts)",
                "spec/conformance/effect-signature/ (revision 2 unchanged)",
            ],
        },
        {
            "suite_id": "SUITE-RESOURCE-DECLARATION-NOT-AUTHORITY",
            "description": (
                "A resource requirement is a declaration of need, never a "
                "grant: satisfaction is a static predicate over explicit "
                "descriptors, no requirement-satisfied-implies-grant path "
                "exists anywhere in af-resource, and the capability "
                "ceiling / effect grant / requirement distinction is "
                "enforced by the static absence audit. Covers the "
                "authority-distinction rules and the crate source audit."
            ),
            "targets": [
                "af-resource static surface audit (no grant/capability symbols)",
                "spec/conformance/resource-requirement/compatibility/",
            ],
        },
        {
            "suite_id": "SUITE-RESOURCE-EXPLICIT-UNIVERSE",
            "description": (
                "Resource-type recognition is an explicit registry-level "
                "fact: the universe is explicit, read-only, in-memory "
                "input; a legal AFIDL schema alone is not a resource "
                "type; there is no latest-schema lookup, no fallback and "
                "no implicit upcast. Covers the universe corpus cases "
                "and the static surface."
            ),
            "targets": [
                "af-resource ResourceTypeUniverse (explicit descriptors)",
                "spec/conformance/resource-requirement/negative/",
            ],
        },
        {
            "suite_id": "SUITE-RESOURCE-NOMINAL-EXACT-PIN",
            "description": (
                "Resource-type matching is exact nominal identity only: "
                "the full (SpecificationId, TypeName, pinned "
                "TypeSchemaDigest) triple; the same short name under a "
                "different specification or with a different digest is a "
                "DIFFERENT resource type, in satisfaction and in "
                "compatibility alike. No implicit subtyping exists. "
                "Covers the same-short-name corpus cases and the "
                "changed-type compatibility cases."
            ),
            "targets": [
                "spec/conformance/resource-requirement/golden/same-short-name-different-family",
                "spec/conformance/resource-requirement/compatibility/",
            ],
        },
        {
            "suite_id": "SUITE-RESOURCE-NO-CROSS-COMPONENT-MERGE",
            "description": (
                "Requirement rows are never merged across components: "
                "port numbers are local to one contract interface, equal "
                "port numbers do not name the same resource, and no "
                "union/merge operation exists in af-resource. "
                "Aggregation requires an explicit port mapping at the "
                "later contract-analysis stage. Covers the "
                "cross-namespace corpus case and the static absence "
                "audit."
            ),
            "targets": [
                "af-resource static surface audit (no merge/union symbols)",
                "spec/conformance/resource-requirement/compatibility/cross-component-namespaces-not-merged",
            ],
        },
        {
            "suite_id": "SUITE-RESOURCE-NO-RUNTIME",
            "description": (
                "The resource layer is static only: no af-ir, af-effect, "
                "af-failure or af-spec dependency, no I/O, clock, "
                "randomness, async runtime, network, credential, adapter "
                "or binding; satisfaction and non-strengthening are pure "
                "functions over explicit values. Covers the crate "
                "dependency audit and the source surface."
            ),
            "targets": [
                "af-resource crate dependency set (af-core, af-idl, af-semantic)",
                "af-resource static surface audit (no runtime symbols)",
            ],
        },
        {
            "suite_id": "SUITE-RESOURCE-WIRE-FREEZE",
            "description": (
                "Phase 4A.3 added exactly one AFIDL module and no HashV1 "
                "domain: no resource-requirement digest exists, "
                "semantic-ir.afidl stays byte-identical, the effect "
                "corpus stays at revision 2, the failure corpus at "
                "revision 1, and every pre-existing corpus digest is "
                "unchanged. Covers the static surface audits and the "
                "zero-drift corpus runs."
            ),
            "targets": [
                "af-resource static surface audit (no digest domains)",
                "af-ir wire-freeze audit (module/ABI/domain counts)",
                "spec/conformance/effect-signature/ and failure-row/ (revisions unchanged)",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-CONCEPTS-DISTINCT",
            "description": (
                "Provenance, trust, taint and information flow stay four "
                "distinct concepts: no trust score, no single trusted "
                "flag; confidentiality restrictions and taints are "
                "separate nominal types; unknown provenance is never "
                "trusted and unknown taint is never the empty set. "
                "Covers the static absence audit and the label/summary "
                "corpus cases."
            ),
            "targets": [
                "af-flow static surface audit (no trust-score symbols)",
                "spec/conformance/information-flow/golden/",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-CONSERVATIVE-PROPAGATION",
            "description": (
                "Taint propagation is may-influence conservative: union "
                "algebra properties frozen, pure expressions inherit all "
                "input taints, unknown influence stays unknown, and no "
                "incomplete influence list can prove a real influence "
                "away. Covers the propagation corpus family and the "
                "static surface."
            ),
            "targets": [
                "af-flow propagation functions (pure, catalogs injected)",
                "spec/conformance/information-flow/propagation/",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-DECLARATION-NOT-AUTHORIZATION",
            "description": (
                "A declared flow ceiling is not an observed flow and not "
                "a policy approval: judgments are "
                "ProvenWithinDeclaredCeiling / "
                "RequiresTrustedTransformation / Unproven only, no "
                "PolicyDecision substitute exists anywhere in af-flow, "
                "and referencing an effect sink is not being granted it. "
                "Covers the static absence audit and the sink corpus "
                "cases."
            ),
            "targets": [
                "af-flow static surface audit (no grant/policy symbols)",
                "spec/conformance/information-flow/",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-KIND-SEPARATION",
            "description": (
                "Data egress and decision influence are separate flow "
                "kinds judged separately: the non-relaxation proof never "
                "compares ceilings across kinds, and the two golden sink "
                "cases freeze the distinction. Covers the flow-kind "
                "rules and the compatibility corpus."
            ),
            "targets": [
                "af-flow flow_not_relaxed (per-kind subset checks)",
                "spec/conformance/information-flow/golden/ (egress + decision sinks)",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-MODEL-MIXER",
            "description": (
                "The LLM strong taint mixer: all model-visible inputs may "
                "influence all model outputs; output taint is the union "
                "of every visible input taint plus model_generated, "
                "influence stays unknown, authority is never elevated, "
                "and a model output is never endorsed. Covers the mixer "
                "corpus cases and the static surface."
            ),
            "targets": [
                "af-flow mix_model_visible_inputs",
                "spec/conformance/information-flow/propagation/",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-NO-AUTOMATIC-PRIVILEGE",
            "description": (
                "No automatic declassification, no automatic endorsement, "
                "persistence does not imply trust: the wire model "
                "expresses no taint-deleting or restriction-lowering "
                "operation, ordinary transformations (store/load/"
                "serialize/summarize) never endorse, and this phase "
                "records at most requires_declassification / "
                "requires_endorsement obligations. Covers the "
                "persistence corpus cases and the static absence audit."
            ),
            "targets": [
                "af-flow static surface audit (no declassify/endorse execution)",
                "spec/conformance/information-flow/propagation/",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-NO-INVENTED-LATTICE",
            "description": (
                "No confidentiality hierarchy is invented: containment is "
                "set membership plus explicit catalog rules, there is no "
                "total order over user-defined labels from which "
                "downgrade flows could be claimed, and undecidable "
                "containment is Unproven rather than resolved by an "
                "invented subtyping. Covers the non-relaxation corpus "
                "and the static surface."
            ),
            "targets": [
                "af-flow flow_not_relaxed (explicit-subset proof only)",
                "spec/conformance/information-flow/compatibility/",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-SINK-NOT-GRANT",
            "description": (
                "External-effect sinks reuse the structured "
                "EffectOperationRef by exact cross-module identity, "
                "resolve fail-closed against the explicit catalog, are "
                "classified only by the catalog (never by operation "
                "name), and referencing an effect is not being granted "
                "it. Covers the sink corpus cases and the read-only "
                "consistency audit over the effect module."
            ),
            "targets": [
                "af-flow EffectSinkUniverse (explicit, immutable input)",
                "spec/conformance/information-flow/negative/unknown-sink-operation",
            ],
        },
        {
            "suite_id": "SUITE-INFOFLOW-WIRE-FREEZE",
            "description": (
                "Phase 4A.4 added exactly one AFIDL module and no HashV1 "
                "domain: no InformationFlowDigest exists, semantic-ir"
                ".afidl stays byte-identical, the effect corpus stays at "
                "revision 2, the failure corpus at revision 1, the "
                "resource corpus at revision 1, and every pre-existing "
                "corpus digest is unchanged. Covers the static surface "
                "audits and the zero-drift corpus runs."
            ),
            "targets": [
                "af-flow static surface audit (no flow digest)",
                "af-ir wire-freeze audit (module/ABI/domain counts)",
                "spec/conformance/effect-signature/, failure-row/, resource-requirement/ (revisions unchanged)",
            ],
        },
        {
            "suite_id": "SUITE-USAGE-STATIC-ONLY",
            "description": (
                "The Usage layer is declaration-only: explicit additive meter "
                "identity and units, finite/unbounded upper bounds, missing "
                "versus zero, static ceiling/non-increase proofs, and no "
                "measurement, billing, pricing, reservation, settlement, "
                "quota or runtime ledger. Covers Usage golden, negative and "
                "compatibility cases plus the static absence audit."
            ),
            "targets": [
                "af-usage static surface audit",
                "spec/conformance/usage-signature/golden/",
                "spec/conformance/usage-signature/negative/",
                "spec/conformance/usage-signature/compatibility/",
            ],
        },
        {
            "suite_id": "SUITE-USAGE-ALGEBRA",
            "description": (
                "Static Usage algebra is conservative and checked: additive "
                "accumulation, branch maximum join, repetition, explicit zero, "
                "unknown propagation and arithmetic overflow preserve sound "
                "upper bounds."
            ),
            "targets": [
                "af-usage algebra functions",
                "spec/conformance/usage-signature/algebra/",
            ],
        },
        {
            "suite_id": "SUITE-USAGE-CONTROL",
            "description": (
                "Control propagation accumulates all possible usage: retry "
                "amplifies by attempts, race retains loser consumption, "
                "compensation never refunds forward usage, and max_inflight "
                "is not a total iteration bound."
            ),
            "targets": [
                "af-usage control propagation functions",
                "spec/conformance/usage-signature/algebra/",
            ],
        },
        {
            "suite_id": "SUITE-USAGE-WIRE-FREEZE",
            "description": (
                "Phase 4A.5A adds exactly one Usage AFIDL module and additive "
                "ABI/corpus entries but no HashV1 domain; all pre-existing "
                "AFIDL TypeSchemaDigests, SemanticIrDigest, effect/failure/"
                "resource/information-flow corpora and ABI assignments remain "
                "unchanged."
            ),
            "targets": [
                "af-usage static surface audit (no digest domain)",
                "af-ir cumulative module/ABI/domain audit",
                "spec/conformance/usage-signature/",
            ],
        },
        {
            "suite_id": "SUITE-COST-STATIC",
            "description": (
                "Cost Projection is static only: explicit currency and atomic "
                "money, complete by-value fixed price snapshots, meter/unit "
                "coverage, forward/recovery cost bounds and static budget "
                "proofs. Missing price is Unproven, never free, and no billing "
                "or ledger authority exists."
            ),
            "targets": [
                "af-usage cost/currency/pricing/budget functions",
                "spec/conformance/cost-projection/golden/",
                "spec/conformance/cost-projection/negative/",
                "spec/conformance/cost-projection/budget/",
            ],
        },
        {
            "suite_id": "SUITE-COST-ROUNDING",
            "description": (
                "Fixed rational rates use checked integer arithmetic and the "
                "safe per-meter-unit ceiling rule; aggregate-once rounding is "
                "rejected because it can underestimate independently rounded "
                "requests."
            ),
            "targets": [
                "af-usage project_cost and checked ceil arithmetic",
                "spec/conformance/cost-projection/pricing/",
            ],
        },
        {
            "suite_id": "SUITE-COST-WIRE-FREEZE",
            "description": (
                "Phase 4A.5B adds one additive Cost Projection AFIDL module "
                "and its ABI/corpus entries but no HashV1 domain; all prior "
                "TypeSchemaDigests, SemanticIrDigest, Usage Signature and "
                "previous corpora remain unchanged."
            ),
            "targets": [
                "af-usage cost static surface audit (no digest/ledger/runtime)",
                "af-ir cumulative module/ABI/domain audit",
                "spec/conformance/cost-projection/",
            ],
        },
        {
            "suite_id": "SUITE-RECOVERY-STATIC",
            "description": (
                "Recovery Contract is static only: replay/idempotency declarations, "
                "exact EffectAtom clauses, reconciliation and compensation obligations "
                "are not receipts, grants, runtime decisions or business failures."
            ),
            "targets": [
                "af-recovery static surface audit",
                "spec/conformance/recovery-contract/golden/",
                "spec/conformance/recovery-contract/replay/",
            ],
        },
        {
            "suite_id": "SUITE-RECOVERY-PROOF",
            "description": (
                "Unknown and partially-applied effects fail closed: blind retry is "
                "forbidden, reconciliation evidence is required, and static proof "
                "distinguishes Proven, RequiresEvidence, Unproven and Violation."
            ),
            "targets": [
                "af-recovery analyze_recovery_safety",
                "spec/conformance/recovery-contract/reconciliation/",
                "spec/conformance/recovery-contract/negative/",
            ],
        },
        {
            "suite_id": "SUITE-RECOVERY-WIRE-FREEZE",
            "description": (
                "Phase 4A.6 adds one additive Recovery AFIDL module and ABI/corpus "
                "entries but no HashV1 domain, receipt, runtime or journal surface; "
                "all prior TypeSchemaDigests and corpora remain unchanged."
            ),
            "targets": [
                "af-recovery static surface audit",
                "af-ir cumulative module/ABI/domain audit",
                "spec/conformance/recovery-contract/",
            ],
        },
        {
            "suite_id": "SUITE-EXECUTION-STATIC",
            "description": (
                "Execution Requirements is static only: explicit capability, "
                "quantity, sandbox, trust, residency, runtime and ABI "
                "constraints are not workers, advertisements, admission, "
                "leases, reservations, scheduler state or authority."
            ),
            "targets": [
                "af-execution static surface audit",
                "spec/conformance/execution-requirements/golden/",
                "spec/conformance/execution-requirements/negative/",
            ],
        },
        {
            "suite_id": "SUITE-EXECUTION-MATCHING",
            "description": (
                "Static profile matching checks all hard constraints and keeps "
                "rejection, unproven evidence and descriptor proof distinct; "
                "soft preferences never compensate for a hard failure."
            ),
            "targets": [
                "af-execution match_execution_requirements",
                "spec/conformance/execution-requirements/matching/",
            ],
        },
        {
            "suite_id": "SUITE-EXECUTION-COMPATIBILITY",
            "description": (
                "Execution compatibility freezes separate placement-feasibility "
                "and security-preservation directions and combines them only "
                "after both verdicts are available."
            ),
            "targets": [
                "af-execution execution_compatibility",
                "spec/conformance/execution-requirements/compatibility/",
            ],
        },
        {
            "suite_id": "SUITE-EXECUTION-WIRE-FREEZE",
            "description": (
                "Phase 4A.7 adds one additive Execution Requirements AFIDL "
                "module and ABI/corpus entries but no HashV1 domain or X "
                "digest; all prior schema digests and static corpora remain "
                "unchanged."
            ),
            "targets": [
                "af-execution static surface audit (no digest/runtime symbols)",
                "af-ir cumulative module/ABI/domain audit",
                "spec/conformance/execution-requirements/",
            ],
        },
        {
            "suite_id": "SUITE-EFFECT-CATALOG-EXPLICIT-UNIVERSE",
            "description": (
                "Effect operation resolution happens against an explicit, "
                "immutable, in-memory catalog universe only: af-effect has "
                "no registry lookup, no latest-version fallback, no "
                "filesystem/network discovery and no default catalog, so "
                "an unresolved operation can only fail closed. Covers the "
                "crate dependency audit and the unknown-operation corpus."
            ),
            "targets": [
                "af-effect crate dependency set (af-core, af-idl, af-semantic)",
                "af_effect::EffectCatalogUniverse (explicit, immutable input)",
                "spec/conformance/effect-signature/negative/",
            ],
        },
        {
            "suite_id": "SUITE-EFFECT-DECLARATION-NOT-AUTHORIZATION",
            "description": (
                "An effect signature is never authority: the crate has no "
                "EffectGrant/EffectIntent/EffectReceipt/Effect Gateway/"
                "Credential Broker/Provider Dispatch symbol, no execution "
                "surface, and the unknown row carries no capability "
                "material. Covers the static absence audit over the crate "
                "source and the frozen top-row corpus."
            ),
            "targets": [
                "af-effect static surface audit (no grant/receipt/gateway symbols)",
                "spec/conformance/effect-signature/negative/top-is-not-authorization",
            ],
        },
        {
            "suite_id": "SUITE-EFFECT-NO-RUNTIME",
            "description": (
                "The effect layer is static only: no af-ir dependency, no "
                "I/O, clock, randomness, async runtime, policy engine, "
                "credential broker or provider; control propagation is a "
                "pure function over signatures. Covers the crate "
                "dependency audit and the source surface."
            ),
            "targets": [
                "af-effect crate dependency set",
                "af-effect static surface audit (no runtime/journal/plan symbols)",
            ],
        },
        {
            "suite_id": "SUITE-EFFECT-RETRY-BOUNDARY",
            "description": (
                "The retry envelope conservatively scales the logical "
                "bounds by the physical attempt ceiling and records the "
                "obligation that narrowing requires a recovery contract: "
                "no idempotency qualifier is exploited this phase. Covers "
                "the retry algebra corpus and the envelope shape."
            ),
            "targets": [
                "af_effect::control::retry_envelope",
                "spec/conformance/effect-signature/algebra/retry-amplification",
            ],
        },
        {
            "suite_id": "SUITE-EFFECT-WIRE-FREEZE",
            "description": (
                "Phase 4A.1 added exactly one AFIDL module and no HashV1 "
                "domain: the canonical effect signature has ACE1 bytes but "
                "no digest, the semantic-ir module stays byte-identical, "
                "and every pre-existing corpus digest is unchanged. Covers "
                "the static surface audits and the zero-drift corpus runs."
            ),
            "targets": [
                "af-effect static surface audit (no EffectSignatureDigest)",
                "af-ir wire-freeze audit (module/ABI/domain counts)",
            ],
        },
]

SUITE_LIST.extend([
    {
        "suite_id": "SUITE-PORTABLE-PROOF-CODEC",
        "description": (
            "Independent Python and Rust production conformance for the registered "
            "portable proof-input schema, canonical ACE1 codec, closed D0-D4 "
            "decoder, inline content identities, reference closure and exact "
            "static semantic-support selection. No context reconstruction or "
            "RootExport revalidation is asserted by this suite."
        ),
        "targets": [
            "tools/gen-portable-proof-codec-corpus.py",
            "spec/conformance/portable-proof-codec/manifest.json",
            "spec/conformance/portable-proof-codec/comparator-mutations.json",
            "crates/af-portable/src",
            "crates/af-portable/tests/conformance.rs",
            "crates/af-portable/tests/static_surface.rs",
        ],
    },
    {
        "suite_id": "SUITE-COMPONENT-CONTRACT-CANONICAL",
        "description": (
            "Component identity, all thirteen semantic fields and full Effect "
            "Atom identity define canonical contract bytes. Publication version, "
            "implementation identity, price and diagnostics are excluded. "
            "Expected bytes and HashV1 values are generated by the independent "
            "Python component-contract oracle."
        ),
        "targets": [
            "crates/af-contract/src/canonical.rs",
            "crates/af-contract/src/digest.rs",
            "spec/conformance/component-contract/canonical/",
            "spec/conformance/component-contract/golden/",
        ],
    },
    {
        "suite_id": "SUITE-COMPONENT-CONTRACT-COMPATIBILITY",
        "description": (
            "Component substitution preserves separate directional proofs. "
            "Exact I/O sets and nominal pins are the positive v1 subset; "
            "required-input/output breaks violate and unsupported relations "
            "remain Unproven."
        ),
        "targets": [
            "crates/af-contract/src/compatibility.rs",
            "spec/conformance/component-contract/compatibility/",
        ],
    },
    {
        "suite_id": "SUITE-COMPONENT-CONTRACT-STATIC",
        "description": (
            "Single-component validation composes exact I/O/R/F/epsilon/K/U/X "
            "checks with explicit relation snapshots. Open and unknown rows, "
            "missing catalog coverage and missing meter coordinates fail "
            "closed; non-Proven subproofs remain visible in the report."
        ),
        "targets": [
            "crates/af-contract/src/validate.rs",
            "spec/conformance/component-contract/cross-dimension/",
            "spec/conformance/component-contract/negative/",
        ],
    },
    {
        "suite_id": "SUITE-COMPONENT-CONTRACT-WIRE-FREEZE",
        "description": (
            "Phase 4B.1 adds exactly the ComponentContract and three accepted "
            "relation schemas, with additive ABI and no new HashV1 domain or "
            "Semantic IR change. Existing schemas and prior corpus fixtures "
            "remain byte-identical."
        ),
        "targets": [
            "spec/idl/component-contract.afidl",
            "spec/idl/effect-resource-compatibility.afidl",
            "spec/idl/recovery-usage-coverage.afidl",
            "spec/idl/flow-execution-relation.afidl",
            "spec/conformance/component-contract/schema-golden.json",
        ],
    },
    {
        "suite_id": "SUITE-COMPONENT-RELATION-CATALOGS",
        "description": (
            "Three standalone relation catalogs have closed exact schemas, "
            "canonical typed content and checksums over ACE1 bytes. A proof "
            "records schema, opaque revision and content checksum; changed "
            "catalog content cannot reuse an old result."
        ),
        "targets": [
            "crates/af-contract/src/universe.rs",
            "spec/component/relation-catalogs.md",
            "spec/conformance/component-contract/relation-catalogs/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-INVOKE",
        "description": (
            "Exact INVOKE target resolution, checked contract pinning, exact "
            "public input/output port typing, required coverage, Phase 3C "
            "producer/scope reuse, and distinct call-site identity."
        ),
        "targets": [
            "crates/af-computation/src/resolution.rs",
            "crates/af-computation/tests/leaf_resolution.rs",
            "spec/conformance/computation-leaf/invoke/",
            "spec/conformance/computation-leaf/negative/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-MAPPING",
        "description": (
            "Exact resource requirement-port to ResourceSlotId mapping, "
            "explicit alias groups, missing attribute evidence, and "
            "source-preserving scoped Effect and Flow mappings."
        ),
        "targets": [
            "crates/af-computation/src/mapping.rs",
            "crates/af-computation/src/summary.rs",
            "spec/conformance/computation-leaf/mapping/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-SCOPED-SUMMARIES",
        "description": (
            "Scoped leaf summaries retain full source contract dimensions, "
            "Effect atom identity and qualifiers, flow provenance, "
            "per-call U/K/X boundaries, and exact proof-context references."
        ),
        "targets": [
            "crates/af-computation/src/summary.rs",
            "spec/computation/leaf-contract-resolution.md",
            "spec/conformance/computation-leaf/mapping/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-SUBFLOW",
        "description": (
            "CALL_SUBFLOW reuses exact Phase 3D targets and interface checks, "
            "requires a complete checked export, separates declaration from "
            "body proof, and checks exact-digest dependency closure."
        ),
        "targets": [
            "crates/af-ir/src/subflow.rs",
            "crates/af-computation/src/resolution.rs",
            "spec/conformance/computation-leaf/subflow/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-AWAIT",
        "description": (
            "AWAIT keeps its exact signal pin and correlation, explicit "
            "unknown external provenance, missing adapter obligations, and "
            "the absence of implicit timeout or business failure."
        ),
        "targets": [
            "crates/af-computation/src/resolution.rs",
            "crates/af-computation/src/summary.rs",
            "spec/conformance/computation-leaf/await/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-CONTEXT",
        "description": (
            "ComponentContract semantic digest stability is separated from "
            "checked proof reuse, which is bound to the exact context object "
            "and relation content."
        ),
        "targets": [
            "crates/af-contract/src/digest.rs",
            "crates/af-contract/src/proof.rs",
            "crates/af-computation/tests/leaf_resolution.rs",
            "spec/conformance/computation-leaf/context/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-PYTHON-ORACLE",
        "description": (
            "A standalone Python implementation computes exact target, "
            "port/resource, scoped effect, subflow, context and verdict "
            "expectations without calling Rust or defining a wire ABI."
        ),
        "targets": [
            "tools/gen-computation-leaf-corpus.py",
            "spec/conformance/computation-leaf/manifest.json",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-PHASE-BOUNDARY",
        "description": (
            "The phase resolves only one leaf and cannot create runtime "
            "authority or close SEQUENCE, SELECT, PARALLEL, RACE, ITERATE, "
            "RETRY, TIMEOUT, HANDLE, SCOPE, or compensation contracts."
        ),
        "targets": [
            "crates/af-computation/Cargo.toml",
            "spec/computation/leaf-contract-resolution.md",
            "docs/implementation/phase-04b2a-proof-boundaries.md",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-LEAF-EQUIVALENCE",
        "description": (
            "Analysis is deterministic for one exact snapshot and source "
            "presentation-only metadata does not alter semantic identity."
        ),
        "targets": [
            "spec/conformance/computation-leaf/equivalence/",
            "tools/gen-computation-leaf-corpus.py",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-SEQUENCE",
        "description": (
            "SEQUENCE preserves child order, uses formal Effect/Usage "
            "accumulation, weakens conditional lower bounds, and retains "
            "Phase 3C cross-sibling dataflow evidence."
        ),
        "targets": [
            "crates/af-computation/src/control.rs",
            "crates/af-computation/tests/leaf_resolution.rs",
            "spec/conformance/computation-control/sequence/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-SELECT",
        "description": (
            "SELECT uses exclusive joins, full guard/qualifier identities, "
            "exact typed interfaces, and explicit unresolved selector "
            "coverage/decision influence."
        ),
        "targets": [
            "crates/af-computation/src/control.rs",
            "spec/conformance/computation-control/select/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-PARALLEL",
        "description": (
            "The frozen all/fail_fast modes retain sibling effects and usage, "
            "keep explicit branch-output and resource mappings, and do not "
            "aggregate X into one Worker."
        ),
        "targets": [
            "spec/semantic/semantic-ir.md sections 10.3 and 13.2",
            "spec/conformance/computation-control/parallel/",
            "crates/af-computation/tests/leaf_resolution.rs",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-RACE",
        "description": (
            "RACE is first-terminal and conservatively accumulates every "
            "contestant's effect, usage, failure and recovery facts without "
            "choosing a static winner."
        ),
        "targets": [
            "spec/conformance/computation-control/race/",
            "crates/af-computation/src/control.rs",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-DIMENSIONS",
        "description": (
            "Failure, Effect, Flow, Usage, Resource, Recovery and Execution "
            "use their separate frozen algebras or source-scoped summaries; "
            "unknown is never treated as empty."
        ),
        "targets": [
            "crates/af-computation/src/control.rs",
            "spec/conformance/computation-control/proof/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-OCCURRENCE",
        "description": (
            "Equal contract digests and AtomKeys do not merge logical call "
            "sites; control provenance retains source nodes and row ordinals."
        ),
        "targets": [
            "crates/af-computation/tests/leaf_resolution.rs",
            "spec/conformance/computation-control/sequence/",
            "spec/conformance/computation-control/parallel/",
            "spec/conformance/computation-control/race/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-PROOF",
        "description": (
            "Inherited Violation, Unproven and RequiresEvidence statuses are "
            "retained alongside any derived envelope; unsupported operators "
            "never default to empty."
        ),
        "targets": [
            "crates/af-computation/src/proof.rs",
            "spec/conformance/computation-control/proof/",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-PYTHON-ORACLE",
        "description": (
            "A standalone Python oracle computes the frozen control vectors "
            "without invoking Rust or introducing a persistence format."
        ),
        "targets": [
            "tools/gen-computation-control-corpus.py",
            "spec/conformance/computation-control/manifest.json",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-PHASE-BOUNDARY",
        "description": (
            "The derived summary is not a checked component contract and "
            "creates no control digest, runtime state, grant, handle, plan, "
            "journal, reducer or worker placement."
        ),
        "targets": [
            "crates/af-computation/Cargo.toml",
            "spec/computation/control-contract-algebra.md",
            "docs/implementation/phase-04b2b1-proof-boundaries.md",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-CONTROL-EQUIVALENCE",
        "description": (
            "Unordered summary inputs normalize deterministically while the "
            "frozen SEQUENCE child order remains semantic."
        ),
        "targets": [
            "spec/conformance/computation-control/equivalence/",
            "tools/gen-computation-control-corpus.py",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-ADVANCED-CONTROL-PYTHON-ORACLE",
        "description": (
            "A standalone Python oracle generates 72 vectors for the four "
            "existing advanced control opcodes, nested propagation, typed "
            "carry/output interfaces, the preserved Scope CaseId, and "
            "boundary failures. Scope is now supported by Phase 4B.2B.3; "
            "only its formerly unsupported Expected changed under revision 2. "
            "Every vector runs through checked Semantic IR, Phase 3C, leaf "
            "resolution and the production analyzer; output, static "
            "multiplicity, Flow dependencies, Recovery proof state and "
            "overflow are compared without runtime simulation."
        ),
        "targets": [
            "tools/gen-advanced-control-corpus.py",
            "spec/conformance/computation-advanced-control/manifest.json",
            "crates/af-computation/tests/advanced_control_conformance.rs",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE",
        "description": (
            "An independent Python oracle generates 55 Scope, exact K-derived "
            "conditional compensation, and nested control-tree vectors. Every "
            "case runs through checked IR, Phase 3C, production leaf resolution, "
            "the production algebra, and root closure; mutations cover resource "
            "proof, compensation bound, traversal order, and source accounting."
        ),
        "targets": [
            "tools/gen-computation-scope-closure-corpus.py",
            "spec/conformance/computation-scope-closure/manifest.json",
            "crates/af-computation/tests/scope_closure_conformance.rs",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE",
        "description": (
            "An independent Python oracle differentially checks typed K-to-"
            "epsilon guards, LocalActionBound/RootActionBound, exact source/"
            "action/profile multisets, operator-specific root folds, root "
            "epsilon/Recovery-Usage coverage, and stage-gated counterexamples "
            "without implementing RootExportAssertion."
        ),
        "targets": [
            "tools/gen-computation-root-recovery-corpus.py",
            "spec/conformance/computation-root-recovery/manifest.json",
            "crates/af-contract/src/recovery_guard.rs",
            "crates/af-computation/src/root_recovery.rs",
            "crates/af-computation/tests/root_recovery_conformance.rs",
        ],
    },
    {
        "suite_id": "SUITE-COMPUTATION-ROOT-EXPORT-PYTHON-ORACLE",
        "description": (
            "An independent Python oracle differentially checks the exact "
            "non-wire RootExportAssertion pins, typed I/O/R/E/F/K/X maps, "
            "nine static dimensions, proof-context invalidation and "
            "subflow/body separation. Every declared case executes checked "
            "Semantic IR, Phase 3C, leaf resolution, root derivation, "
            "RootRecoverySummary, and the production RootExport checker."
        ),
        "targets": [
            "tools/gen-computation-root-export-corpus.py",
            "spec/conformance/computation-root-export/manifest.json",
            "crates/af-computation/src/root_export.rs",
            "crates/af-computation/tests/root_export_conformance.rs",
        ],
    },
])

SUITE_LIST.sort(key=lambda suite: suite["suite_id"])
write_json("spec/conformance/suites.json", {
    "kind": "agentflow.conformance.suites",
    "suite_revision": 1,
    "suites": SUITE_LIST,
})
