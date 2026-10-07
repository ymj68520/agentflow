#!/usr/bin/env python3
"""Freezes the spec-registry conformance corpus.

Generates spec/conformance/spec-registry/: a minimal, self-contained registry
(valid/base) plus single-mutation copies that each trigger exactly one
cross-reference validation failure class. Deterministic and idempotent: the
same run always produces the same bytes (content digests are recomputed from
the generated source.md, so reruns are no-ops).
"""

import hashlib
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "spec-registry"
# Revision 2 (Phase 2C): adds the import-aware cases - a valid multi-module set
# whose second module imports the first by exact SpecificationId, a duplicate
# schema family, and a cross-module type cycle. Revision 1 was the Phase 2B
# single-module corpus; existing cases are unchanged.
CORPUS_REVISION = 2

FAMILY = "agentflow.test.mini@1"


def write_json(path: Path, payload: dict) -> None:
    text = json.dumps(payload, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


SOURCE_MD = """# Mini specification source

Normative source for the spec-registry conformance corpus (Phase 2B).
Everything here exists only so that rule anchors and content digests have a
real file to point at.

<a id="mini-rule-one"></a>

Mini rule one: the fixture schema keeps its declared numbering.

<a id="mini-rule-two"></a>

Mini rule two: fixture rules may exist without runnable conformance cases.
"""

REGISTRY_AFDL = """afidl 1;

schema agentflow.test.mini @1;

record Widget {
    1 id: string;
    2 count: uint32;
}

enum Color {
    1 red;
    2 green;
}
"""

REGISTRY_AFDL_RETIRED = """afidl 1;

schema agentflow.test.mini @1;

record Widget {
    2 count: uint32;
}

enum Color {
    1 red;
    2 green;
}
"""

REGISTRY_AFDL_EXTRA = """afidl 1;

schema agentflow.test.mini @1;

record Widget {
    1 id: string;
    2 count: uint32;
    3 extra: string;
}

enum Color {
    1 red;
    2 green;
}
"""

REGISTRY_AFDL_MOVED = """afidl 1;

schema agentflow.test.mini @1;

record Widget {
    2 count: uint32;
    3 id: string;
}

enum Color {
    1 red;
    2 green;
}
"""

# A second module of the same fixture registry, importing the first by its
# exact SpecificationId. Used by the import-aware corpus cases.
IMPORTER_AFDL = """afidl 1;

schema agentflow.test.consumer @1;

import agentflow.test.mini@1 as mini;

record Holder {
    1 color: mini.Color;
}
"""

# Identical schema family declared by two modules: the module set is not a
# function of specification identity.
DUPLICATE_FAMILY_AFDL = """afidl 1;

schema agentflow.test.mini @1;

record Other {
    1 value: string;
}
"""

# Two modules that depend on each other's types: mutual imports with a real
# type cycle, which the global graph must reject.
CYCLE_A_AFDL = """afidl 1;

schema agentflow.test.cycle-a @1;

import agentflow.test.cycle-b@1 as b;

record A {
    1 other: b.B;
}
"""

CYCLE_B_AFDL = """afidl 1;

schema agentflow.test.cycle-b @1;

import agentflow.test.cycle-a@1 as a;

record B {
    1 other: a.A;
}
"""


def base_files() -> dict:
    source_digest = hashlib.sha256(SOURCE_MD.encode("utf-8")).hexdigest()
    specifications = {
        "registry_revision": 1,
        "specifications": [
            {
                "specification_id": FAMILY,
                "kind": "schema",
                "status": "frozen",
                "normative_source": {
                    "logical_path": "source.md",
                    "content_digest": {"algorithm": "sha256", "hex": source_digest},
                },
                "description": "Mini fixture schema for the spec-registry corpus.",
            }
        ],
    }
    rules = {
        "registry_revision": 1,
        "rules": [
            {
                "rule_id": "AF-MINI-0001",
                "area": "MINI",
                "normative_source": "source.md",
                "source_anchor": "mini-rule-one",
                "requirement_level": "must",
                "short_statement": "The mini schema keeps its declared numbering.",
                "conformance_refs": ["CONF-MINI-0001"],
            },
            {
                "rule_id": "AF-MINI-0002",
                "area": "MINI",
                "normative_source": "source.md",
                "source_anchor": "mini-rule-two",
                "requirement_level": "should",
                "short_statement": "Fixture rules may exist without runnable cases.",
                "conformance_refs": [],
                "intentionally_uncovered_reason": (
                    "Corpus fixture rule; it exercises the intentionally-"
                    "uncovered classification, not a real requirement."
                ),
            },
        ],
    }
    rule_areas = {
        "kind": "agentflow.rule-areas",
        "version": "v1",
        "note": "Mini closed area table for the spec-registry corpus.",
        "areas": [
            {"area": "MINI", "description": "Mini fixture area"},
            {"area": "WIDGET", "description": "Second fixture area, intentionally unused"},
        ],
    }
    abi = {
        "registry_revision": 1,
        "assignments": [
            # Canonical order: (schema_family, type_name, number_kind, number)
            # with number_kind sorted as its JSON string, so Color rows come
            # before Widget rows.
            assignment("Color", "enum_value", 1, "red", "active"),
            assignment("Color", "enum_value", 2, "green", "active"),
            assignment("Widget", "field", 1, "id", "active"),
            assignment("Widget", "field", 2, "count", "active"),
        ],
    }
    conformance = {
        "kind": "agentflow.conformance.mini-cases",
        "note": "Mini fixture case universe.",
        "cases": [
            {
                "case_id": "CONF-MINI-0001",
                "description": "The only resolvable conformance case in the mini corpus.",
            }
        ],
    }
    return {
        "specifications.json": specifications,
        "rules.json": rules,
        "rule-areas.json": rule_areas,
        "abi-numbers.json": abi,
        "registry.afidl": REGISTRY_AFDL,
        "source.md": SOURCE_MD,
        "conformance/cases.json": conformance,
    }


def assignment(type_name, kind, number, name, status, family=None):
    return {
        "schema_family": family or FAMILY,
        "type_name": type_name,
        "number_kind": kind,
        "number": number,
        "symbolic_name": name,
        "status": status,
    }


def widget_field_1_index(rows):
    for index, entry in enumerate(rows):
        if (entry["type_name"], entry["number_kind"], entry["number"]) == ("Widget", "field", 1):
            return index
    raise AssertionError("Widget field 1 row not found")


def mutate(base, which):
    """Returns a copy of the base file dict with one mutation applied."""
    files = json.loads(json.dumps(base))  # deep copy
    specs = files["specifications.json"]["specifications"]
    rules = files["rules.json"]["rules"]
    abi_rows = files["abi-numbers.json"]["assignments"]

    if which == "duplicate-specification-id":
        clone = json.loads(json.dumps(specs[0]))
        clone["description"] = "Deliberate duplicate SpecificationId."
        specs.append(clone)
    elif which == "bad-rule-id":
        rules[0]["rule_id"] = "AF-MINI-1"
    elif which == "duplicate-rule-id":
        clone = json.loads(json.dumps(rules[0]))
        clone["source_anchor"] = "mini-rule-two"
        rules.append(clone)
    elif which == "missing-source":
        specs[0]["normative_source"]["logical_path"] = "missing.md"
    elif which == "missing-anchor":
        rules[0]["source_anchor"] = "mini-rule-missing"
    elif which == "unknown-rule-area":
        rules[0]["area"] = "JOURNAL"
    elif which == "unknown-conformance-case":
        rules[0]["conformance_refs"] = ["CONF-MISSING-0001"]
    elif which == "duplicate-abi-assignment":
        # Insert right after the original row so only the duplicate-tuple
        # check fires (the array stays canonically ordered).
        abi_rows.insert(widget_field_1_index(abi_rows) + 1,
                        assignment("Widget", "field", 1, "id", "active"))
    elif which == "retired-number-reuse":
        abi_rows[widget_field_1_index(abi_rows)] = assignment(
            "Widget", "field", 1, "legacy_id", "retired")
    elif which == "retired-number-in-use":
        abi_rows[widget_field_1_index(abi_rows)] = assignment(
            "Widget", "field", 1, "id", "retired")
    elif which == "name-rebound-to-new-number":
        abi_rows[widget_field_1_index(abi_rows)] = assignment(
            "Widget", "field", 1, "id", "retired")
        files["registry.afidl"] = REGISTRY_AFDL_MOVED
    elif which == "unregistered-abi-assignment":
        files["registry.afidl"] = REGISTRY_AFDL_EXTRA
    elif which == "active-assignment-missing":
        abi_rows.append(assignment("Widget", "field", 3, "extra", "active"))
        abi_rows.sort(key=lambda a: (
            a["schema_family"], a["type_name"], a["number_kind"], a["number"]))
    elif which == "canonical-order-violation":
        abi_rows[-1], abi_rows[-2] = abi_rows[-2], abi_rows[-1]
    elif which == "frozen-source-provisional":
        files["source.md"] = (
            SOURCE_MD + "\nStatus: provisional -- do not rely on this file.\n"
        )
        # The digest must still match so only the provisional check fires.
        digest = hashlib.sha256(files["source.md"].encode("utf-8")).hexdigest()
        specs[0]["normative_source"]["content_digest"]["hex"] = digest
    elif which == "content-digest-mismatch":
        specs[0]["normative_source"]["content_digest"]["hex"] = "0" * 63 + "1"
    elif which == "module-set-import":
        # Valid case: a second module imports the first by exact identity, and
        # the ABI registry covers both families.
        files["importer.afidl"] = IMPORTER_AFDL
        abi_rows.extend([
            assignment("Holder", "field", 1, "color", "active",
                       family="agentflow.test.consumer@1"),
        ])
        abi_rows.sort(key=lambda a: (
            a["schema_family"], a["type_name"], a["number_kind"], a["number"]))
    elif which == "duplicate-module-family":
        files["other.afidl"] = DUPLICATE_FAMILY_AFDL
    elif which == "cross-module-recursion":
        files["cycle-a.afidl"] = CYCLE_A_AFDL
        files["cycle-b.afidl"] = CYCLE_B_AFDL
    elif which == "retired-number":
        # Valid case: a genuinely retired number that the schema no longer
        # uses, with everything else consistent.
        abi_rows[widget_field_1_index(abi_rows)] = assignment(
            "Widget", "field", 1, "id", "retired")
        files["registry.afidl"] = REGISTRY_AFDL_RETIRED
    else:
        raise AssertionError(f"unknown mutation {which}")
    return files


CASES = [
    # (case_id, dir, outcome, expected error kind or None)
    ("spec-registry/valid-base", "valid/base", "valid", None),
    ("spec-registry/valid-retired-number", "valid/retired-number", "valid", None),
    ("spec-registry/valid-module-set-import", "valid/module-set-import",
     "valid", None),
    ("spec-registry/duplicate-specification-id", "invalid/duplicate-specification-id",
     "invalid", "duplicate_specification_id"),
    ("spec-registry/bad-rule-id", "invalid/bad-rule-id", "invalid", "invalid_rule_id"),
    ("spec-registry/duplicate-rule-id", "invalid/duplicate-rule-id",
     "invalid", "duplicate_rule_id"),
    ("spec-registry/missing-source", "invalid/missing-source",
     "invalid", "source_not_found"),
    ("spec-registry/missing-anchor", "invalid/missing-anchor",
     "invalid", "anchor_not_found"),
    ("spec-registry/unknown-rule-area", "invalid/unknown-rule-area",
     "invalid", "unknown_rule_area"),
    ("spec-registry/unknown-conformance-case", "invalid/unknown-conformance-case",
     "invalid", "unknown_conformance_case"),
    ("spec-registry/duplicate-abi-assignment", "invalid/duplicate-abi-assignment",
     "invalid", "duplicate_abi_assignment"),
    ("spec-registry/retired-number-reuse", "invalid/retired-number-reuse",
     "invalid", "abi_number_reassigned"),
    ("spec-registry/retired-number-in-use", "invalid/retired-number-in-use",
     "invalid", "abi_retired_number_in_use"),
    ("spec-registry/name-rebound-to-new-number", "invalid/name-rebound-to-new-number",
     "invalid", "abi_name_rebound"),
    ("spec-registry/unregistered-abi-assignment", "invalid/unregistered-abi-assignment",
     "invalid", "abi_unregistered_assignment"),
    ("spec-registry/active-assignment-missing", "invalid/active-assignment-missing",
     "invalid", "abi_active_assignment_missing"),
    ("spec-registry/canonical-order-violation", "invalid/canonical-order-violation",
     "invalid", "canonical_form"),
    ("spec-registry/frozen-source-provisional", "invalid/frozen-source-provisional",
     "invalid", "provisional_frozen_source"),
    ("spec-registry/content-digest-mismatch", "invalid/content-digest-mismatch",
     "invalid", "content_digest_mismatch"),
    ("spec-registry/duplicate-module-family", "invalid/duplicate-module-family",
     "invalid", "duplicate_module_family"),
    ("spec-registry/cross-module-recursion", "invalid/cross-module-recursion",
     "invalid", "module_set_compile"),
]


def main() -> None:
    if CONF.exists():
        shutil.rmtree(CONF)
    base = base_files()
    for case_id, directory, outcome, error_kind in CASES:
        files = base if directory == "valid/base" else mutate(base, directory.rsplit("/", 1)[1])
        target = CONF / directory
        for name, payload in files.items():
            if name.endswith(".json"):
                write_json(target / name, payload)
            else:
                (target / name).parent.mkdir(parents=True, exist_ok=True)
                (target / name).write_text(payload, encoding="utf-8", newline="\n")
        print(f"wrote {directory} -> {outcome}")

    manifest = {
        "kind": "agentflow.conformance.spec-registry",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Frozen spec-registry conformance corpus. Each case is a "
            "self-contained mini registry (Layout::corpus_case): valid cases "
            "must validate cleanly; invalid cases must produce at least one "
            "error whose kind matches error_kind. Generated by "
            "tools/gen-spec-registry-corpus.py; do not edit cases by hand."
        ),
        "cases": [
            {
                "case_id": case_id,
                "dir": directory,
                "outcome": outcome,
                **({"error_kind": error_kind} if error_kind else {}),
            }
            for case_id, directory, outcome, error_kind in CASES
        ],
    }
    write_json(CONF / "manifest.json", manifest)
    print(f"wrote manifest.json ({len(CASES)} cases)")


if __name__ == "__main__":
    main()
