#!/usr/bin/env python3
"""Read-only Phase 4B.0 inventory and frozen-surface audit.

This script verifies repository facts and reports known design findings. It
does not prove cross-dimension semantics, generate expectations, modify files,
or turn a source-symbol search into a semantic proof.
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BASELINE = "5dc75eb"

EXPECTED_MODULES = {
    "spec/idl/core/machine-primitives.afidl",
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
}

EXPECTED_IDENTITIES = {
    "agentflow.core.machine-primitives@1",
    "agentflow.spec.registry@1",
    "agentflow.semantic.core@1",
    "agentflow.semantic.ir@1",
    "agentflow.effect.signature@1",
    "agentflow.failure.row@1",
    "agentflow.resource.requirement@1",
    "agentflow.information.flow@1",
    "agentflow.usage.signature@1",
    "agentflow.usage.cost@1",
    "agentflow.recovery.contract@1",
    "agentflow.execution.requirements@1",
}

EXPECTED_IMPORTS = {
    "spec/idl/semantic-ir.afidl": {"agentflow.semantic.core@1 as core"},
    "spec/idl/effect-signature.afidl": {"agentflow.semantic.core@1 as core"},
    "spec/idl/failure-row.afidl": {"agentflow.semantic.core@1 as core"},
    "spec/idl/resource-requirement.afidl": {"agentflow.semantic.core@1 as core"},
    "spec/idl/information-flow.afidl": {
        "agentflow.semantic.core@1 as core",
        "agentflow.effect.signature@1 as effect",
    },
    "spec/idl/cost-projection.afidl": {"agentflow.usage.signature@1 as usage"},
    "spec/idl/recovery-contract.afidl": {"agentflow.effect.signature@1 as effect"},
    "spec/idl/execution-requirements.afidl": {"agentflow.semantic.core@1 as core"},
}

FROZEN_PATHS = [
    "Cargo.toml",
    "Cargo.lock",
    "spec/idl",
    "spec/tables/hash-domains.json",
    "spec/tables/rule-areas.json",
    "spec/registry/abi-numbers.json",
    "spec/registry/coverage-summary.json",
    "spec/conformance/suites.json",
    "spec/conformance/afidl",
    "spec/conformance/encoding",
    "spec/conformance/digest",
    "spec/conformance/spec-registry",
    "spec/conformance/semantic-core",
    "spec/conformance/semantic-ir",
    "spec/conformance/semantic-dataflow",
    "spec/conformance/subflow-interface",
    "spec/conformance/failure-row",
    "spec/conformance/resource-requirement",
    "spec/conformance/information-flow",
    "spec/conformance/usage-signature",
    "spec/conformance/cost-projection",
    "spec/conformance/execution-requirements",
]


def read_json(relative: str):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def git_differs(baseline: str, paths: list[str]) -> bool:
    result = subprocess.run(
        ["git", "diff", "--quiet", baseline, "--", *paths],
        cwd=ROOT,
        check=False,
    )
    if result.returncode not in (0, 1):
        raise RuntimeError(f"git diff failed with status {result.returncode}")
    return result.returncode == 1


def report_check(name: str, passed: bool, detail: str) -> bool:
    state = "PASS" if passed else "FAIL"
    print(f"{state} {name}: {detail}")
    return passed


def corpus_revision(relative: str) -> int:
    value = read_json(relative)
    return int(value["corpus_revision"])


def imports(relative: str) -> set[str]:
    source = (ROOT / relative).read_text(encoding="utf-8")
    found = set()
    pattern = r"^\s*import\s+([a-z0-9.-]+)\s*@\s*(\d+)\s+as\s+([a-z][a-z0-9_]*)\s*;"
    for namespace, major, alias in re.findall(pattern, source, flags=re.MULTILINE):
        found.add(f"{namespace}@{major} as {alias}")
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--baseline",
        default=DEFAULT_BASELINE,
        help=f"Phase 4B.0 baseline commit (default: {DEFAULT_BASELINE})",
    )
    args = parser.parse_args()

    failed = False
    specifications = read_json("spec/registry/specifications.json")["specifications"]
    rules = read_json("spec/registry/rules.json")["rules"]
    abi_rows = read_json("spec/registry/abi-numbers.json")["assignments"]
    active_rows = [row for row in abi_rows if row["status"] == "active"]
    modules = {
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "spec/idl").rglob("*.afidl")
    }
    domains = read_json("spec/tables/hash-domains.json")["domains"]
    suites = read_json("spec/conformance/suites.json")["suites"]

    for name, condition, detail in [
        ("Specifications", len(specifications) == 30, f"{len(specifications)} (expected 30)"),
        ("Normative Rules", len(rules) == 296, f"{len(rules)} (expected 296)"),
        ("Active ABI assignments", len(active_rows) == 400, f"{len(active_rows)} (expected 400)"),
        ("AFIDL modules", modules == EXPECTED_MODULES, f"{len(modules)}; exact 12-module family set={modules == EXPECTED_MODULES}"),
        ("HashV1 domains", len(domains) == 14, f"{len(domains)} (expected 14)"),
        ("Conformance suites", len(suites) == 67, f"{len(suites)} (expected 67)"),
        ("No ComponentContract AFIDL", not (ROOT / "spec/idl/component-contract.afidl").exists(), "component-contract.afidl absent"),
        (
            "No ComponentContract SpecificationId",
            all(spec["specification_id"] != "agentflow.component.contract@1" for spec in specifications),
            "no registered ComponentContract SpecificationId",
        ),
        (
            "No production ComponentContract type",
            not any(
                re.search(
                    r"\b(?:struct|enum|type)\s+ComponentContract\b",
                    "\n".join(
                        line.split("//", 1)[0]
                        for line in path.read_text(encoding="utf-8").splitlines()
                    ),
                )
                for path in (ROOT / "crates").glob("af-*/src/**/*.rs")
            ),
            "no struct/enum/type ComponentContract declaration in crates",
        ),
    ]:
        failed |= not report_check(name, condition, detail)

    actual_identities = set()
    for relative in sorted(EXPECTED_MODULES):
        source = (ROOT / relative).read_text(encoding="utf-8")
        match = re.search(r"^\s*schema\s+([a-z0-9.-]+)\s*@\s*(\d+)\s*;", source, flags=re.MULTILINE)
        if match:
            actual_identities.add(f"{match.group(1)}@{match.group(2)}")
    failed |= not report_check(
        "AFIDL SpecificationId set",
        actual_identities == EXPECTED_IDENTITIES,
        f"{len(actual_identities)} identities; exact set={actual_identities == EXPECTED_IDENTITIES}",
    )

    for path, expected_imports in EXPECTED_IMPORTS.items():
        actual_imports = imports(path)
        failed |= not report_check(
            f"{Path(path).name} exact imports",
            actual_imports == expected_imports,
            f"{sorted(actual_imports)}",
        )

    revisions = {
        "AFIDL": ("spec/conformance/afidl/schema-golden.json", 13),
        "Effect": ("spec/conformance/effect-signature/golden/effect-golden-pure-empty-row.json", 3),
        "Failure": ("spec/conformance/failure-row/golden/failure-golden-empty-failure-row.json", 1),
        "Resource": ("spec/conformance/resource-requirement/golden/resource-golden-empty-requirement-row.json", 1),
        "InformationFlow": ("spec/conformance/information-flow/golden/infoflow-golden-empty-flow-contract.json", 2),
        "Usage": ("spec/conformance/usage-signature/golden/usage-golden-empty-not-zero.json", 1),
        "Cost": ("spec/conformance/cost-projection/golden/cost-golden-explicit-zero-usage.json", 1),
        "Recovery": ("spec/conformance/recovery-contract/golden/recovery-golden-pure-recorded.json", 2),
        "Execution": ("spec/conformance/execution-requirements/golden/execution-golden-empty.json", 1),
    }
    for name, (path, expected) in revisions.items():
        actual = corpus_revision(path)
        failed |= not report_check(
            f"{name} corpus revision",
            actual == expected,
            f"{actual} (expected {expected})",
        )

    failed |= not report_check(
        "Frozen schemas, ABI, hash domains and unrelated corpora unchanged",
        not git_differs(args.baseline, FROZEN_PATHS),
        f"compared with {args.baseline}; Effect/Recovery revisions and their source-digest registries are the explicit Phase 4B.0A changes",
    )

    # Presence checks are inventories, not behavioral proofs.
    required_models = {
        "af-failure": ["FailureRow", "FailureTypeRef"],
        "af-effect": ["EffectSignature", "EffectAtom", "ResourceSelector"],
        "af-resource": ["ResourceRequirementRow", "ResourceTypeUniverse"],
        "af-flow": ["InformationFlowContract", "EffectSinkUniverse"],
        "af-usage": ["UsageSignature", "StaticUsageEnvelope"],
        "af-recovery": ["RecoveryContract", "RecoveryProof"],
        "af-execution": ["ExecutionRequirements", "StaticExecutorProfile"],
    }
    for crate, symbols in required_models.items():
        source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (ROOT / "crates" / crate / "src").glob("*.rs")
        )
        present = all(symbol in source for symbol in symbols)
        failed |= not report_check(
            f"{crate} typed model inventory",
            present,
            ", ".join(symbols),
        )

    for name, relative, symbol in [
        ("Effect containment API", "crates/af-effect/src/algebra.rs", "pub fn contains"),
        ("Failure containment API", "crates/af-failure/src/algebra.rs", "pub fn contains"),
        ("Resource compatibility API", "crates/af-resource/src/compatibility.rs", "pub fn requirement_not_stronger"),
        ("Flow compatibility API", "crates/af-flow/src/compatibility.rs", "pub fn flow_not_relaxed"),
        ("Usage compatibility API", "crates/af-usage/src/compatibility.rs", "pub fn usage_not_increased"),
        ("Cost compatibility API", "crates/af-usage/src/cost.rs", "pub fn cost_not_increased"),
        ("Recovery compatibility API", "crates/af-recovery/src/compatibility.rs", "pub fn recovery_not_weakened"),
        ("Execution compatibility API", "crates/af-execution/src/compatibility.rs", "pub fn execution_compatibility"),
        ("Subflow interface API", "crates/af-ir/src/subflow.rs", "pub fn derive_subflow_interface"),
    ]:
        present = symbol in (ROOT / relative).read_text(encoding="utf-8")
        failed |= not report_check(name, present, f"{relative}: {symbol}")

    atom_key = (ROOT / "crates/af-effect/src/model.rs").read_text(encoding="utf-8")
    flow_schema = (ROOT / "spec/idl/information-flow.afidl").read_text(encoding="utf-8")
    recovery_schema = (ROOT / "spec/idl/recovery-contract.afidl").read_text(encoding="utf-8")
    full_qualifier_key = (
        "qualifiers: Vec<EffectQualifierIdentity>" in atom_key
        and "qualifier.member_no()" in atom_key
        and "qualifiers.iter().map(|q| q.family_no())" not in atom_key
    )
    failed |= not report_check(
        "Effect AtomKey preserves full qualifier identity",
        full_qualifier_key,
        "typed qualifier identity includes member discriminant and typed value",
    )
    if full_qualifier_key:
        print("CLOSED P0: Effect AtomKey includes qualifier family, member and typed value.")
    else:
        print("FINDING P0: AtomKey may omit qualifier member identity; audit required.")
    print("FINDING P1: Effect and Resource universes expose no operation/resource compatibility relation.")
    print("FINDING P1: FlowSink.external_effect carries EffectOperationRef, not resource-selector identity.")
    print("FINDING P1: Recovery reconciliation has no operation field; compensation carries EffectOperationRef only.")
    print("FINDING P1: No RecoveryAction-to-Usage profile links K recovery work to U recovery meters.")
    print("FINDING P1: F label/taint identities have no explicit relation to X region/trust/sandbox profiles.")
    print("FINDING P1: Component I/O subtype and alias-versus-target comparison rules are not materialized.")
    print("FINDING P1: No ComponentContract identity/version/canonical-body policy is registered.")
    contract_digest_call = False
    for path in (ROOT / "crates").glob("af-*/src/**/*.rs"):
        source = path.read_text(encoding="utf-8")
        if re.search(r"hash_v1\s*\([^;]*DigestDomain::ComponentContract", source, flags=re.DOTALL):
            contract_digest_call = True
            break
    failed |= not report_check(
        "No production ComponentContract digest call",
        not contract_digest_call,
        "no hash_v1 call using DigestDomain::ComponentContract",
    )
    if not full_qualifier_key:
        print("NOTE: the known P0 source pattern changed; re-audit behavior before relying on this inventory.")
    if "external_effect: effect.EffectOperationRef" not in flow_schema:
        print("NOTE: the known FlowSink shape changed; re-audit selector granularity.")
    if "compensation: CompensationRequirement" not in recovery_schema or "reconciliation: ReconciliationRequirement" not in recovery_schema:
        print("NOTE: the known Recovery shapes changed; re-audit full action identity.")

    print("LIMIT: this script checks static repository facts only; it does not prove cross-dimension semantics.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
