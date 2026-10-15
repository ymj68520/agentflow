#!/usr/bin/env python3
"""Read-only T3 admission evidence, NOT a fact producer or a conformance oracle.

Source census is lexical evidence at a pinned baseline. The semantic finding is
reviewed separately in the amendment request; absence of a spelling alone does
not prove that no equivalent formal binding could ever be approved. Exit 2 means
the currently reviewed admission is blocked, not that Rust regression failed.
"""

import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = "b121ab716bf4e87e33e7433b2b9dc3407699d327"
MODEL = "spec/tables/proof-report-model-v1.md"


def read(path):
    return (ROOT / path).read_text(encoding="utf-8")


def positions(path, pattern):
    text = read(path).split("#[cfg(test)]", 1)[0]
    return [text.count("\n", 0, match.start()) + 1 for match in re.finditer(pattern, text)]


def main():
    model = json.loads(re.search(r"```json\s*(.*?)\s*```", read(MODEL), re.S).group(1))
    specs = json.loads(read("spec/registry/specifications.json"))["specifications"]
    rules = json.loads(read("spec/registry/rules.json"))["rules"]
    normative = next(row for row in specs if row["specification_id"] == model["model_id"])
    digest = hashlib.sha256((ROOT / MODEL).read_bytes()).hexdigest()
    assert normative["normative_source"]["content_digest"]["hex"] == digest
    assert (len(specs), len(rules)) == (46, 436)
    result = subprocess.run(
        ["git", "diff", "--quiet", BASELINE, "--", "crates", "spec", "Cargo.toml", "Cargo.lock"],
        cwd=ROOT, check=False,
    )
    if result.returncode != 0:
        raise RuntimeError("Production/frozen tree differs from the reviewed baseline")

    control = "crates/af-computation/src/control.rs"
    leaf = "crates/af-computation/src/resolution.rs"
    constructor_sites = {}
    for prefix, path, pattern, expected in [
        ("C", control, r"ControlDimensionProof::new\(", 65),
        ("L", leaf, r"LeafDimensionProof::new\(", 27),
    ]:
        lines = positions(path, pattern)
        assert len(lines) == expected, (prefix, lines)
        constructor_sites[prefix] = [
            {"producer_site_id": f"{prefix}{index:02}", "source_path": path, "line": line,
             "capture_status": "NOT_IMPLEMENTED", "capture_case_ids": []}
            for index, line in enumerate(lines, 1)
        ]
    direct = positions("crates/af-computation/src/root_export.rs", r"unresolved_obligations\.push\(")
    inherited = positions(control, r"(?<!struct )(?<!impl )InheritedControlObligation\s*\{")
    assert len(direct) == 6 and len(inherited) == 3, (direct, inherited)
    adaptation = read("docs/implementation/phase-04c1b4c2-producer-adaptation-map.md")
    reviewed_arms = re.findall(r"^\| P(\d+) /\d+ \|", adaptation, re.M)
    assert sorted(map(int, reviewed_arms)) == list(range(1, 12))
    compatibility = read("crates/af-contract/src/compatibility.rs")
    literal_dimensions = re.search(
        r"fn dimensions\(.*?\{\s*\[(.*?)\]\s*\.into_iter", compatibility, re.S
    ).group(1)
    substitution_rows = re.findall(r'"([a-z-]+)"', literal_dimensions)
    assert len(substitution_rows) == 12 and "identity" in substitution_rows
    assert "if new.component_identity == previous.component_identity" in compatibility

    family_constraint = next(
        text for text in model["semantic_constraints"] if text.startswith("PredicateFamily =")
    )
    constraint_first_line = family_constraint.splitlines()[0]
    assert "substitution" not in constraint_first_line
    aliases = {
        family: [case for case in cases if "substitut" in case or "lineage" in case]
        for family, cases in model["closed_predicates"].items()
    }
    # This validates the specific reviewed absence, not semantic equivalence by
    # string matching. A new binding needs normative review, not a Python rename.
    assert not any(aliases.values())
    output = {
        "kind": "T3 admission/source census; NOT Typed Fact Seed Conformance",
        "baseline": BASELINE,
        "decision": "FORMAL_AMENDMENT_REQUIRED",
        "t3_status": "SOURCE_COVERAGE_BLOCKED; no production capture implemented",
        "model_id": model["model_id"], "model_sha256": digest,
        "registry_counts": {"specifications": len(specs), "rules": len(rules)},
        "production_and_frozen_git_surface_unchanged": True,
        "direct_root_sites": direct, "inherited_sites": inherited,
        "prerequisite_arms_reviewed": 11, "prerequisite_literal_occurrences_reviewed": 20,
        "constructor_sites": constructor_sites,
        "substitution_dimension_rows": substitution_rows,
        "predicate_family_declaration": constraint_first_line,
        "closed_predicate_roster": model["closed_predicates"],
        "substitution_or_lineage_named_cases": aliases,
        "finding": {
            "id": "T3-REG-01",
            "source": "crates/af-contract/src/compatibility.rs",
            "predicate": "checked replacement.component_identity == old.component_identity",
            "rule": "AF-CONTRACT-0020",
            "scope": "standalone contract substitution; no caller IR/root export required",
            "required_resolution": "approved exact predicate/kind/clause/role/source binding, or explicit scope amendment",
            "not_a_finding": "No claim of false-Proven or unavailable original data",
        },
        "new_t3_capture_cases": 0, "new_t3_comparator_mutations": 0,
        "scope_identity": "PENDING_G02", "normalization": "NOT_IMPLEMENTED",
        "full_d10_oracle": "NOT_IMPLEMENTED", "positive_capability": "NOT_AUTHORIZED",
    }
    print(json.dumps(output, ensure_ascii=True, indent=2))
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
