#!/usr/bin/env python3
"""Generate an independent adversarial RootExport soundness corpus.

Expected results are derived only from the frozen per-entry containment rule
and explicit resolver-snapshot inputs. This script never invokes Rust and
never reads production output.
"""

from __future__ import annotations

import json
from legacy_edition1_inputs import assert_historical_output
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec/conformance/computation-root-export-soundness"
DIMENSIONS = ["I", "O", "Phi", "epsilon", "R", "F", "U", "K", "X"]


def atom(scope=(), qualifier="natural"):
    qualifiers = [] if qualifier == "plain" else [{"family": "idempotency", "member": qualifier}]
    return {
        "operation": {"catalog": "agentflow.effects@1", "name": "write"},
        "resource_selector": {
            "requirement_port_no": 1,
            "scope_bindings": [{"key": key, "value": value} for key, value in scope],
        },
        "qualifiers": qualifiers,
    }


def guard(value=None):
    return "unconditional" if value is None else {"input_equals": {"port": 1, "value": value}}


def effect(atom_value, upper, guard_value=None):
    return {"atom": atom_value, "lower": 0, "upper": upper, "guard": guard(guard_value)}


def atom_compatible(root, export):
    root_atom = root["atom"]
    export_atom = export["atom"]
    return (
        root_atom["operation"] == export_atom["operation"]
        and root_atom["qualifiers"] == export_atom["qualifiers"]
        and root_atom["resource_selector"]["requirement_port_no"]
        == export_atom["resource_selector"]["requirement_port_no"]
        and all(
            binding in root_atom["resource_selector"]["scope_bindings"]
            for binding in export_atom["resource_selector"]["scope_bindings"]
        )
    )


def guard_contained(root_guard, export_guard):
    if export_guard == "unconditional":
        return True
    if root_guard == "unconditional":
        return False
    return root_guard == export_guard


def typed_guard_counterexample(root_guard, export_guard):
    return root_guard != "unconditional" and export_guard != "unconditional" and root_guard != export_guard or (
        root_guard == "unconditional" and export_guard != "unconditional"
    )


def counterexample_value(root_guard, export_guard):
    if guard_contained(root_guard, export_guard):
        return None
    if root_guard == "unconditional":
        equals = export_guard.get("input_equals") if isinstance(export_guard, dict) else None
        if equals is None:
            return None
        value = equals["value"]
        return 1 if value != 1 else 0
    if root_guard.get("input_equals", {}).get("port") == export_guard.get("input_equals", {}).get("port"):
        return root_guard["input_equals"]["value"]
    return None


def containment_oracle(root_entries, export_entries):
    """Per-root-entry, same-row oracle with one-use export slots."""
    claimed = set()
    for root in root_entries:
        candidates = [
            (index, row) for index, row in enumerate(export_entries) if atom_compatible(root, row)
        ]
        if not candidates:
            return "violation"
        full = [
            (index, row)
            for index, row in candidates
            if row["lower"] <= root["lower"]
            and row["upper"] >= root["upper"]
            and guard_contained(root["guard"], row["guard"])
        ]
        available = next(((index, row) for index, row in full if index not in claimed), None)
        if available is not None:
            claimed.add(available[0])
            continue
        if full:
            return "unproven"
        if len(candidates) != 1:
            return "unproven"
        _, row = candidates[0]
        if guard_contained(root["guard"], row["guard"]) and row["upper"] < root["upper"]:
            return "violation"
        if row["upper"] >= root["upper"] and typed_guard_counterexample(root["guard"], row["guard"]):
            return "violation"
        return "unproven"
    return "proven"


def effect_case(case_id, root_entries, export_entries):
    keys = sorted(
        {
            binding["key"]
            for entry in [*root_entries, *export_entries]
            for binding in entry["atom"]["resource_selector"]["scope_bindings"]
        }
    )
    epsilon = containment_oracle(root_entries, export_entries)
    static = "proven" if epsilon == "proven" else epsilon
    row_analysis = []
    for root_ordinal, root_entry in enumerate(root_entries):
        rows = []
        for export_ordinal, export_entry in enumerate(export_entries):
            compatible = atom_compatible(root_entry, export_entry)
            bounds_covered = (
                export_entry["lower"] <= root_entry["lower"]
                and export_entry["upper"] >= root_entry["upper"]
            )
            guard_covered = guard_contained(root_entry["guard"], export_entry["guard"])
            counterexample = counterexample_value(root_entry["guard"], export_entry["guard"])
            rows.append(
                {
                    "export_ordinal": export_ordinal,
                    "root_atom": root_entry["atom"],
                    "export_atom": export_entry["atom"],
                    "operation_qualifier_selector_compatible": compatible,
                    "bounds_covered": bounds_covered,
                    "guard_contained": guard_covered,
                    "typed_countermodel_candidate": None
                    if counterexample is None
                    else {"port": 1, "primitive": "int64", "value": counterexample},
                }
            )
        row_analysis.append({"root_ordinal": root_ordinal, "attempted_export_rows": rows})
    return {
        "case_id": case_id,
        "kind": "effect_containment",
        "input": {
            "root_effects": root_entries,
            "export_effects": export_entries,
            "recognized_resource_attributes": keys,
            "effect_selector_rules": [
                {"selector_key": key, "resource_attribute_key": key, "required": False}
                for key in keys
            ],
        },
        "expected": {
            "stage": "root_export_checker",
            "epsilon": epsilon,
            "static_compatibility": static,
            "checked_capability": static == "proven",
            "preflight": "proven",
            "body_conformance": "requires_evidence",
            "root_effect_count": len(root_entries),
            "export_effect_count": len(export_entries),
            "input_type": {"port": 1, "primitive": "int64"},
            "row_analysis": row_analysis,
            "rule_ids": ["AF-SEMANTIC-0166"],
        },
    }


def snapshot_case(
    case_id,
    root_upper,
    check_upper,
    top_upper,
    *,
    extra_component=False,
    without_export=False,
    different_context=False,
):
    stale = root_upper != check_upper or extra_component or without_export or different_context
    epsilon = "unproven" if stale else ("violation" if root_upper > top_upper else "proven")
    static = epsilon
    return {
        "case_id": case_id,
        "kind": "resolver_snapshot",
        "input": {
            "root_snapshot_export_upper": root_upper,
            "checker_snapshot_export_upper": check_upper,
            "top_export_upper": top_upper,
            "snapshot_b_extra_component": extra_component,
            "snapshot_b_without_subflow_export": without_export,
            "checker_context_variant": different_context,
        },
        "expected": {
            "stage": "root_export_checker",
            "epsilon": epsilon,
            "static_compatibility": static,
            "checked_capability": static == "proven",
            "preflight": "unproven" if stale else "proven",
            "body_conformance": "requires_evidence",
            "root_effect_count": 1,
            "export_effect_count": 1,
            "rule_ids": ["AF-SEMANTIC-0160", "AF-SEMANTIC-0171", "AF-SEMANTIC-0166"],
        },
    }


def cases():
    result = []
    ab = [("a", "1"), ("b", "1")]
    root = atom(ab)

    result.append(
        effect_case(
            "root-export-soundness/f01/p30-split-guard-bound-selector",
            [effect(root, 10)],
            [
                effect(atom([("a", "1")]), 10, 0),
                effect(atom([("b", "1")]), 5),
            ],
        )
    )
    result.extend(
        [
            effect_case("root-export-soundness/f01/a-full-same-row", [effect(root, 10)], [effect(atom([("a", "1")]), 10)]),
            effect_case("root-export-soundness/f01/b-full-exact-selector", [effect(root, 10)], [effect(root, 10)]),
            effect_case("root-export-soundness/f01/c-guard-covered-bound-short", [effect(root, 10)], [effect(root, 5)]),
            effect_case("root-export-soundness/f01/d-bound-covered-guard-rejects", [effect(root, 10)], [effect(root, 10, 0)]),
            effect_case("root-export-soundness/f01/e-root-guard-narrowed-by-export", [effect(root, 10, 0)], [effect(root, 10)]),
            effect_case("root-export-soundness/f01/f-equal-typed-guard", [effect(root, 10, 0)], [effect(root, 10, 0)]),
            effect_case("root-export-soundness/f01/g-different-typed-guard", [effect(root, 10, 1)], [effect(root, 10, 0)]),
            effect_case("root-export-soundness/f01/h-qualifier-none", [effect(root, 10)], [effect(atom(ab, "none"), 10)]),
            effect_case("root-export-soundness/f01/i-qualifier-plain", [effect(root, 10)], [effect(atom(ab, "plain"), 10)]),
            effect_case("root-export-soundness/f01/j-selector-value-mismatch", [effect(atom([("a", "1")]), 10)], [effect(atom([("a", "2")]), 10)]),
            effect_case("root-export-soundness/f01/k-selector-extra-key", [effect(root, 10)], [effect(atom([("a", "1"), ("c", "1")]), 10)]),
            effect_case("root-export-soundness/f01/l-split-bounds-and-guard-rows", [effect(root, 10)], [effect(atom([("a", "1")]), 10, 0), effect(atom([("b", "1")]), 5)]),
            effect_case("root-export-soundness/f01/m-selector-and-guard-split", [effect(root, 10)], [effect(atom([("a", "1")]), 10, 0), effect(atom([("a", "1"), ("b", "1"), ("c", "1")]), 10)]),
            effect_case("root-export-soundness/f01/n-extra-decoy-does-not-hide-witness", [effect(root, 10)], [effect(root, 10), effect(atom([("a", "1"), ("c", "1")]), 20, 0)]),
            effect_case("root-export-soundness/f01/o-two-root-occurrences-one-slot", [effect(atom([("a", "1")]), 6), effect(atom([("b", "1")]), 6)], [effect(atom([]), 8)]),
            effect_case("root-export-soundness/f01/p-two-root-occurrences-two-slots", [effect(atom([("a", "1")]), 6), effect(atom([("b", "1")]), 6)], [effect(atom([("a", "1")]), 6), effect(atom([("b", "1")]), 6)]),
            effect_case("root-export-soundness/f01/q-three-occurrences-two-slots", [effect(atom([("a", "1")]), 4), effect(atom([("b", "1")]), 4), effect(atom([("c", "1")]), 4)], [effect(atom([("a", "1")]), 4), effect(atom([("b", "1")]), 4)]),
            effect_case("root-export-soundness/f01/r-three-occurrences-three-slots", [effect(atom([("a", "1")]), 4), effect(atom([("b", "1")]), 4), effect(atom([("c", "1")]), 4)], [effect(atom([("a", "1")]), 4), effect(atom([("b", "1")]), 4), effect(atom([("c", "1")]), 4)]),
        ]
    )

    for index in range(1, 13):
        key = f"scope_{index:02}"
        value = f"v{index:02}"
        one = atom([(key, value)])
        result.append(
            effect_case(
                f"root-export-soundness/f01/same-row-scope-bound-{index:02}",
                [effect(one, index + 2)],
                [effect(one, index + 2)],
            )
        )

    for index in range(1, 7):
        expected_value = index * 3
        one = atom([("guard_scope", str(index))])
        result.append(
            effect_case(
                f"root-export-soundness/f01/same-row-typed-guard-{index:02}",
                [effect(one, 20, expected_value)],
                [effect(one, 20, expected_value)],
            )
        )

    result.extend(
        [
            snapshot_case("root-export-soundness/f02/a-same-snapshot-positive", 1, 1, 1),
            snapshot_case("root-export-soundness/f02/b-same-context-changed-snapshot", 1, 10, 1),
            snapshot_case("root-export-soundness/f02/c-fresh-root-under-new-snapshot", 10, 10, 1),
            snapshot_case("root-export-soundness/f02/d-wider-top-export", 1, 1, 10),
            snapshot_case("root-export-soundness/f02/e-changed-snapshot-even-if-top-is-wide", 5, 7, 20),
            snapshot_case("root-export-soundness/f02/f-same-new-snapshot-positive", 10, 10, 10),
            snapshot_case("root-export-soundness/f02/g-component-pin-set-change", 1, 1, 1, extra_component=True),
            snapshot_case("root-export-soundness/f02/h-missing-target-export-association", 1, 1, 1, without_export=True),
            snapshot_case("root-export-soundness/f02/i-same-resolver-data-different-context-object", 1, 1, 1, different_context=True),
            snapshot_case("root-export-soundness/f02/j-new-snapshot-exact-positive", 2, 2, 2),
        ]
    )
    return result


def main():
    vectors = cases()
    ids = [case["case_id"] for case in vectors]
    assert len(ids) == len(set(ids))
    OUT.mkdir(parents=True, exist_ok=True)
    assert_historical_output(OUT / "cases.json", json.dumps(vectors, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "kind": "agentflow.conformance.computation-root-export-soundness.corpus",
        "revision": 1,
        "case_count": len(vectors),
        "case_ids": ids,
        "stage_distribution": {"root_export_checker": len(vectors)},
        "f01_case_count": sum(case["kind"] == "effect_containment" for case in vectors),
        "f02_case_count": sum(case["kind"] == "resolver_snapshot" for case in vectors),
        "expected_checked_capabilities": sum(case["expected"]["checked_capability"] for case in vectors),
        "oracle": "tools/gen-root-export-soundness-corpus.py::containment_oracle",
    }
    assert_historical_output(OUT / "manifest.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"RootExport soundness corpus: {len(vectors)} cases (F01={manifest['f01_case_count']}, F02={manifest['f02_case_count']})")


if __name__ == "__main__":
    main()
