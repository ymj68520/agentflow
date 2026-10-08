#!/usr/bin/env python3
"""Independent Phase 4A.7 Execution Requirements oracle."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "execution-requirements"
CORPUS_REVISION = 1
spec = importlib.util.spec_from_file_location("sir", ROOT / "tools" / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sir)
afidl = sir.afidl
CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
EXEC = ROOT / "spec" / "idl" / "execution-requirements.afidl"
CAP_CATALOG = "agentflow.test.execution.capabilities@1"
TRUST_CATALOG = "agentflow.test.execution.trust@1"
REGION_CATALOG = "agentflow.test.execution.regions@1"
RUNTIME_CATALOG = "agentflow.test.execution.runtime@1"
CAPS = [("browser.automation", "1"), ("browser.automation", "2"),
        ("gpu.compute", "1"), ("model.inference", "2")]
TRUST = ["trusted.batch", "isolated.batch"]
REGIONS = ["us-east-1", "eu-west-1", "eu-central-1"]
RUNTIMES = [("python", "py311"), ("native", "abi2")]
ABI_SCHEMA = {
    "specification_id": "agentflow.test.failures@1",
    "type_name": "PaymentDeclined",
    "schema_digest": {
        "algorithm": "sha256",
        "domain": "type-schema",
        "hex": "a" * 64,
    },
}

def catalogs():
    return {
        "capabilities": [
            {"catalog": CAP_CATALOG, "name": name, "version": version,
             "attributes": [{"key": "api", "value": "v1"}]}
            for name, version in CAPS
        ],
        "trust_domains": [{"catalog": TRUST_CATALOG, "name": name} for name in TRUST],
        "regions": [{"catalog": REGION_CATALOG, "name": name} for name in REGIONS],
        "runtime_profiles": [
            {"catalog": RUNTIME_CATALOG, "name": name, "family": family,
             "features": ["sandboxed", "abi"]}
            for family, name in RUNTIMES
        ],
    }

def load_types():
    sources = [
        ("semantic-core.afidl", CORE.read_text(encoding="utf-8")),
        ("execution-requirements.afidl", EXEC.read_text(encoding="utf-8")),
    ]
    _, by_module = afidl.compile_module_set(sources)
    return {f"{spec}/{schema['name']}": schema
            for spec, schemas in by_module.items() for schema in schemas}

TYPES = load_types()

def shape(identity):
    kind, payload = TYPES[identity]["body"]
    if kind == "alias":
        return expr_shape(payload)
    if kind == "enum":
        return ("enum", identity, {v["name"] for v in payload["values"]})
    if kind == "record":
        return ("record", identity,
                [(f["name"], expr_shape(f["type"])) for f in payload["fields"]])
    if kind == "variant":
        return ("variant", identity,
                [(c["name"], expr_shape(c["type"])) for c in payload["cases"]])
    raise AssertionError(kind)

def expr_shape(expr):
    kind, value = expr
    if kind == "primitive":
        return sir.PRIM_SHAPES[value]
    if kind == "ref":
        return shape(value)
    if kind == "list":
        return ("list", expr_shape(value))
    if kind == "option":
        return ("option", expr_shape(value))
    return ("map", expr_shape(value))

REQ_SHAPE = shape("agentflow.execution.requirements@1/ExecutionRequirements")

def value_tree(shape_value, value):
    kind = shape_value[0]
    if kind == "enum":
        return value
    if kind == "record":
        return {name: value_tree(field_shape, value[name])
                for name, field_shape in shape_value[2]}
    if kind == "list":
        return [value_tree(shape_value[1], item) for item in value]
    if kind == "option":
        return None if value is None else value_tree(shape_value[1], value)
    if kind == "variant":
        if isinstance(value, str):
            return {value: None}
        name, payload = next(iter(value.items()))
        return {name: value_tree(dict(shape_value[2])[name], payload)}
    return value

def ace1(requirements):
    tree = value_tree(REQ_SHAPE, requirements)
    return sir.encode_value(REQ_SHAPE, sir.json_to_value(REQ_SHAPE, tree))

class XError(Exception):
    def __init__(self, kind):
        self.kind = kind

def cap_ref(name, catalog=CAP_CATALOG):
    return {"catalog": catalog, "name": name}

def attr(key="api", value="v1"):
    return {"key": key, "value": value}

def requirement(caps=(), quantities=(), sandbox=None, trust=None,
                regions=None, runtime=None, abis=(), prefs=()):
    return {
        "capabilities": [
            {"capability": cap_ref(name), "exact_version": version,
             "required_attributes": [attr(*item) if isinstance(item, tuple) else item
                                     for item in attributes]}
            for name, version, attributes in caps
        ],
        "quantities": [{"kind": kind, "minimum": minimum}
                       for kind, minimum in quantities],
        "sandbox": sandbox,
        "trust_domain": trust,
        "region_residency": regions,
        "runtime_profile": runtime,
        "implementation_abis": list(abis),
        "soft_preferences": list(prefs),
    }

def normalize(requirement_value):
    seen = set()
    capabilities = []
    known = {}
    for entry in catalogs()["capabilities"]:
        known.setdefault((entry["catalog"], entry["name"]), set()).add(entry["version"])
    for item in requirement_value["capabilities"]:
        key = (item["capability"]["catalog"], item["capability"]["name"])
        versions = known.get(key)
        if versions is None:
            raise XError("unknown_execution_capability")
        if item["exact_version"] not in versions:
            raise XError("unsupported_capability_version")
        attributes = sorted({(sir.nfc(x["key"]), sir.nfc(x["value"]))
                             for x in item["required_attributes"]})
        if any(not key or not value for key, value in attributes):
            raise XError("invalid_execution_capability")
        vocabulary = {
            (attribute["key"], attribute["value"])
            for entry in catalogs()["capabilities"]
            if entry["catalog"] == key[0] and entry["name"] == key[1]
            for attribute in entry["attributes"]
        }
        if not set(attributes).issubset(vocabulary):
            raise XError("capability_requirement_unproven")
        identity = (key, item["exact_version"])
        if identity in seen:
            raise XError("duplicate_execution_requirement")
        seen.add(identity)
        capabilities.append({
            "capability": item["capability"],
            "exact_version": item["exact_version"],
            "required_attributes": [{"key": key, "value": value}
                                    for key, value in attributes],
        })
    capabilities.sort(key=lambda item: (
        item["capability"]["catalog"],
        item["capability"]["name"],
        item["exact_version"],
    ))
    quantities = sorted(requirement_value["quantities"],
                        key=lambda item: {"cpu_millicores": 1,
                                           "memory_bytes": 2,
                                           "gpu_count": 3}[item["kind"]])
    seen_quantity = set()
    for item in quantities:
        if item["kind"] in seen_quantity:
            raise XError("duplicate_execution_requirement")
        seen_quantity.add(item["kind"])
        if not isinstance(item["minimum"], int) or item["minimum"] < 0:
            raise XError("invalid_compute_quantity")
    trust = requirement_value["trust_domain"]
    if trust is not None:
        if trust not in [{"catalog": TRUST_CATALOG, "name": name} for name in TRUST]:
            raise XError("unknown_trust_domain")
    regions = requirement_value["region_residency"]
    if regions is not None:
        for key in ("execution_regions", "residency_regions"):
            values = regions[key]
            if any(value not in [{"catalog": REGION_CATALOG, "name": name}
                                 for name in REGIONS] for value in values):
                raise XError("unknown_region")
            regions[key] = sorted({(value["catalog"], value["name"])
                                   for value in values})
            regions[key] = [{"catalog": catalog, "name": name}
                            for catalog, name in regions[key]]
    runtime = requirement_value["runtime_profile"]
    if runtime is not None:
        profile = next((entry for entry in catalogs()["runtime_profiles"]
                        if entry["catalog"] == runtime["profile"]["catalog"]
                        and entry["name"] == runtime["profile"]["name"]), None)
        if profile is None or profile["family"] != runtime["family"]:
            raise XError("unknown_runtime_profile")
        runtime["required_features"] = sorted(set(runtime["required_features"]))
        if not set(runtime["required_features"]).issubset(profile["features"]):
            raise XError("unknown_runtime_profile")
    for abi in requirement_value["implementation_abis"]:
        schema = abi["schema"]
        digest = schema["schema_digest"]
        if (not schema["specification_id"] or not schema["type_name"]
                or digest != {"algorithm": "sha256", "domain": "type-schema",
                              "hex": digest["hex"]}
                or len(digest["hex"]) != 64
                or digest["hex"] != digest["hex"].lower()
                or any(ch not in "0123456789abcdef" for ch in digest["hex"])):
            raise XError("runtime_abi_incompatible")
    requirement_value["implementation_abis"] = sorted(
        requirement_value["implementation_abis"],
        key=lambda item: (item["schema"]["specification_id"],
                          item["schema"]["type_name"],
                          item["schema"]["schema_digest"]["hex"]))
    preferences = sorted(set(requirement_value["soft_preferences"]),
                         key=lambda item: {
                             "artifact_locality": 1,
                             "warm_model_cache": 2,
                             "lower_load": 3,
                             "lower_cost": 4,
                         }[item])
    return {**requirement_value, "capabilities": capabilities,
            "quantities": quantities, "soft_preferences": preferences}

def executor_profile(**kwargs):
    return {
        "capabilities": kwargs.get("capabilities", []),
        "quantities": kwargs.get("quantities", []),
        "sandbox": kwargs.get("sandbox"),
        "trust_domain": kwargs.get("trust_domain"),
        "execution_region": kwargs.get("execution_region"),
        "residency_regions": kwargs.get("residency_regions", []),
        "runtime_profile": kwargs.get("runtime_profile"),
        "implementation_abis": kwargs.get("implementation_abis", []),
    }

def match(requirement_value, profile):
    normalized = normalize(requirement_value)
    for needed in normalized["capabilities"]:
        candidate = next(
            (item for item in profile["capabilities"]
             if item["capability"] == needed["capability"]), None)
        if candidate is None:
            return {"outcome": "rejected", "reason": "capability_missing"}
        if candidate["version"] != needed["exact_version"]:
            return {"outcome": "rejected", "reason": "capability_version"}
        if not all(item in candidate.get("attributes", [])
                   for item in needed["required_attributes"]):
            return {"outcome": "rejected", "reason": "capability_missing"}
    for needed in normalized["quantities"]:
        candidate = next((item for item in profile["quantities"]
                          if item["kind"] == needed["kind"]), None)
        if candidate is None:
            return {"outcome": "unproven", "reason": "quantity_profile"}
        if candidate["amount"] < needed["minimum"]:
            return {"outcome": "rejected", "reason": "quantity_insufficient"}
    if normalized["sandbox"] is not None:
        if profile["sandbox"] is None:
            return {"outcome": "unproven", "reason": "sandbox_profile"}
        levels = {"basic": 1, "strong": 2}
        if levels[profile["sandbox"]] < levels[normalized["sandbox"]["minimum"]]:
            return {"outcome": "rejected", "reason": "sandbox_too_weak"}
    if normalized["trust_domain"] is not None:
        if profile["trust_domain"] is None:
            return {"outcome": "unproven", "reason": "trust_domain"}
        if profile["trust_domain"] != normalized["trust_domain"]:
            return {"outcome": "rejected", "reason": "trust_domain_mismatch"}
    if normalized["region_residency"] is not None:
        regions = normalized["region_residency"]
        if profile["execution_region"] is None:
            return {"outcome": "unproven", "reason": "region_residency"}
        if (regions["execution_regions"]
                and profile["execution_region"] not in regions["execution_regions"]):
            return {"outcome": "rejected", "reason": "region_mismatch"}
        if (regions["residency_regions"]
                and not any(item in regions["residency_regions"]
                            for item in profile["residency_regions"])):
            return {"outcome": "rejected", "reason": "residency_mismatch"}
    if normalized["runtime_profile"] is not None:
        if profile["runtime_profile"] != normalized["runtime_profile"]["profile"]:
            return {"outcome": "rejected", "reason": "runtime_profile_mismatch"}
    for needed in normalized["implementation_abis"]:
        if needed["schema"] not in profile["implementation_abis"]:
            return {"outcome": "unproven", "reason": "abi_profile"}
    return {"outcome": "proven"}

def base(case_kind, case_id, note):
    return {"kind": "agentflow.conformance.execution-requirements.case",
            "case_kind": case_kind, "corpus_revision": CORPUS_REVISION,
            "case_id": case_id, "note": note, "catalogs": catalogs()}

CASES = []

def write_case(subdir, payload):
    path = CONF / subdir / f"{payload['case_id'].replace('/', '-')}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n",
                    encoding="utf-8", newline="\n")
    CASES.append(payload["case_id"])

def golden(case_id, note, requirements, profile):
    normalized = normalize(requirements)
    write_case("golden", {**base("golden", case_id, note),
                          "requirements": requirements, "profile": profile,
                          "canonical_ace1_utf8": ace1(normalized),
                          "expected": match(requirements, profile)})

def negative(case_id, note, requirements, expected):
    try:
        normalize(requirements)
    except XError as error:
        if error.kind != expected:
            raise AssertionError((case_id, error.kind, expected))
    else:
        raise AssertionError(case_id)
    write_case("negative", {**base("negative", case_id, note),
                            "requirements": requirements,
                            "expected_kind": expected})

def malformed_negative(case_id, note, requirements, expected):
    write_case("negative", {**base("negative", case_id, note),
                            "requirements": requirements,
                            "expected_kind": expected,
                            "malformed": True})

def matching(case_id, note, requirements, profile, expected):
    actual = match(requirements, profile)
    if actual != expected:
        raise AssertionError((case_id, actual, expected))
    write_case("matching", {**base("matching", case_id, note),
                            "requirements": requirements, "profile": profile,
                            "expected": expected})

def compatibility(case_id, note, new, old, expected):
    new = normalize(new)
    old = normalize(old)
    feasibility = "proven"
    security = "proven"
    old_caps = {(item["capability"]["catalog"], item["capability"]["name"]): item
                for item in old["capabilities"]}
    new_caps = {(item["capability"]["catalog"], item["capability"]["name"]): item
                for item in new["capabilities"]}
    for key, needed in new_caps.items():
        previous = old_caps.get(key)
        if previous is None:
            feasibility = "violation"
        elif needed["exact_version"] != previous["exact_version"]:
            feasibility = "unproven" if feasibility == "proven" else feasibility
        elif not set((item["key"], item["value"])
                     for item in needed["required_attributes"]).issubset(
                         {(item["key"], item["value"])
                          for item in previous["required_attributes"]}):
            feasibility = "violation"
    old_quantities = {item["kind"]: item["minimum"] for item in old["quantities"]}
    new_quantities = {item["kind"]: item["minimum"] for item in new["quantities"]}
    for kind in set(old_quantities) | set(new_quantities):
        if kind not in old_quantities or (
                kind in new_quantities and new_quantities[kind] > old_quantities[kind]):
            feasibility = "violation"

    old_sandbox = None if old["sandbox"] is None else {"basic": 1, "strong": 2}[old["sandbox"]["minimum"]]
    new_sandbox = None if new["sandbox"] is None else {"basic": 1, "strong": 2}[new["sandbox"]["minimum"]]
    if old_sandbox is None and new_sandbox is not None:
        feasibility = "violation"
    elif old_sandbox is not None and new_sandbox is None:
        security = "violation"
    elif old_sandbox is not None and new_sandbox is not None:
        if new_sandbox > old_sandbox:
            feasibility = "violation"
        elif new_sandbox < old_sandbox:
            security = "violation"
    if new["trust_domain"] != old["trust_domain"]:
        security = "violation" if new["trust_domain"] is None else "unproven"
    for key in ("execution_regions", "residency_regions"):
        new_regions = set() if new["region_residency"] is None else {
            (item["catalog"], item["name"])
            for item in new["region_residency"][key]}
        old_regions = set() if old["region_residency"] is None else {
            (item["catalog"], item["name"])
            for item in old["region_residency"][key]}
        if new_regions == old_regions:
            continue
        if not new_regions:
            security = "violation"
        elif not old_regions:
            feasibility = "violation"
        elif new_regions.issubset(old_regions):
            feasibility = "violation"
        elif old_regions.issubset(new_regions):
            security = "violation"
        else:
            feasibility = "unproven"
            security = "unproven"
    if new["runtime_profile"] != old["runtime_profile"] or new["implementation_abis"] != old["implementation_abis"]:
        feasibility = "unproven"
    combined = ("unproven" if "unproven" in (feasibility, security)
                else "violation" if "violation" in (feasibility, security)
                else "proven")
    actual = {"feasibility": feasibility, "security": security, "combined": combined}
    if actual != expected:
        raise AssertionError((case_id, actual, expected))
    write_case("compatibility", {**base("compatibility", case_id, note),
                                 "new_requirements": new, "old_requirements": old,
                                 "expected": expected})

def equivalence(case_id, note, members):
    frozen = ace1(normalize(members[0]))
    if any(ace1(normalize(member)) != frozen for member in members[1:]):
        raise AssertionError(case_id)
    write_case("equivalence", {**base("equivalence", case_id, note),
                               "members": members,
                               "canonical_ace1_utf8": frozen})

def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()
    profile = executor_profile(
        capabilities=[{"capability": cap_ref("browser.automation"),
                       "version": "1", "attributes": [attr()]}],
        quantities=[{"kind": "cpu_millicores", "amount": 4000},
                    {"kind": "memory_bytes", "amount": 2**34}],
        sandbox="strong",
        trust_domain={"catalog": TRUST_CATALOG, "name": "trusted.batch"},
        execution_region={"catalog": REGION_CATALOG, "name": "eu-west-1"},
        residency_regions=[{"catalog": REGION_CATALOG, "name": "eu-west-1"}],
        runtime_profile={"catalog": RUNTIME_CATALOG, "name": "py311"},
        implementation_abis=[ABI_SCHEMA],
    )
    golden("execution/golden/empty", "empty requirements", requirement(), profile)
    golden("execution/golden/single-capability", "single exact capability",
           requirement(caps=[("browser.automation", "1", [("api", "v1")])]),
           profile)
    golden("execution/golden/versioned-capability", "exact capability version",
           requirement(caps=[("model.inference", "2", [])]), profile)
    golden("execution/golden/multiple-capabilities", "multiple nominal capabilities",
           requirement(caps=[("model.inference", "2", []), ("browser.automation", "1", [])]),
           profile)
    golden("execution/golden/quantities", "CPU and memory quantities",
           requirement(quantities=[("cpu_millicores", 2000), ("memory_bytes", 2**33)]),
           profile)
    golden("execution/golden/gpu-capability-quantity", "GPU capability and quantity",
           requirement(caps=[("gpu.compute", "1", [])],
                       quantities=[("gpu_count", 1)]), profile)
    golden("execution/golden/sandbox-trust-region-runtime", "hard placement constraints",
           requirement(sandbox={"minimum": "strong"},
                       trust={"catalog": TRUST_CATALOG, "name": "trusted.batch"},
                       regions={"execution_regions": [{"catalog": REGION_CATALOG, "name": "eu-west-1"}],
                                "residency_regions": [{"catalog": REGION_CATALOG, "name": "eu-west-1"}]},
                       runtime={"family": "python", "profile": {"catalog": RUNTIME_CATALOG, "name": "py311"},
                                "required_features": ["sandboxed"]}),
           profile)
    golden("execution/golden/hard-and-soft", "soft preferences remain separate",
           requirement(caps=[("browser.automation", "1", [])],
                       prefs=["lower_cost", "artifact_locality"]), profile)
    golden("execution/golden/runtime-and-abi", "runtime and exact implementation ABI",
           requirement(runtime={"family": "python", "profile": {"catalog": RUNTIME_CATALOG, "name": "py311"},
                              "required_features": ["abi"]}, abis=[{"schema": ABI_SCHEMA}]), profile)
    # Negative -------------------------------------------------------------
    negative("execution/negative/unknown-capability", "unknown capability fails closed",
             requirement(caps=[("unknown", "1", [])]), "unknown_execution_capability")
    negative("execution/negative/wrong-version", "unknown exact version is not latest compatible",
             requirement(caps=[("browser.automation", "9", [])]), "unsupported_capability_version")
    negative("execution/negative/duplicate-capability", "duplicate hard requirement",
             requirement(caps=[("browser.automation", "1", []), ("browser.automation", "1", [])]),
             "duplicate_execution_requirement")
    negative("execution/negative/duplicate-quantity", "duplicate quantity",
             requirement(quantities=[("cpu_millicores", 1), ("cpu_millicores", 2)]),
             "duplicate_execution_requirement")
    negative("execution/negative/unknown-trust", "unknown trust domain",
             requirement(trust={"catalog": TRUST_CATALOG, "name": "unknown"}), "unknown_trust_domain")
    negative("execution/negative/unknown-region", "unknown region",
             requirement(regions={"execution_regions": [{"catalog": REGION_CATALOG, "name": "mars"}],
                                  "residency_regions": []}), "unknown_region")
    negative("execution/negative/unknown-runtime", "unknown runtime profile",
             requirement(runtime={"family": "python", "profile": {"catalog": RUNTIME_CATALOG, "name": "old"},
                                  "required_features": []}), "unknown_runtime_profile")
    negative("execution/negative/invalid-capability-attribute", "empty capability attribute",
             requirement(caps=[("browser.automation", "1", [("", "v")])]), "invalid_execution_capability")
    negative("execution/negative/unknown-capability-attribute", "attribute outside explicit vocabulary",
             requirement(caps=[("browser.automation", "1", [("tier", "gpu")])]),
             "capability_requirement_unproven")
    malformed_negative("execution/negative/invalid-quantity-representation", "negative quantity is not representable",
                       {**requirement(), "quantities": [{"kind": "cpu_millicores", "minimum": -1}]},
                       "invalid_compute_quantity")
    malformed_negative("execution/negative/quantity-unit-mismatch", "unknown quantity unit fails closed",
                       {**requirement(), "quantities": [{"kind": "cores", "minimum": 1}]},
                       "quantity_unit_mismatch")
    malformed_negative("execution/negative/unknown-sandbox-profile", "unknown sandbox level fails closed",
                       {**requirement(), "sandbox": {"minimum": "maximum"}},
                       "invalid_sandbox_requirement")
    # Matching -------------------------------------------------------------
    matching("execution/matching/all-hard-satisfied", "all hard requirements satisfied",
             requirement(caps=[("browser.automation", "1", [])],
                         quantities=[("cpu_millicores", 2000)]), profile, {"outcome": "proven"})
    matching("execution/matching/hard-capability-rejected", "missing hard capability rejected",
             requirement(caps=[("gpu.compute", "1", [])]), profile, {"outcome": "rejected", "reason": "capability_missing"})
    matching("execution/matching/quantity-insufficient", "quantity is not capability",
             requirement(quantities=[("memory_bytes", 2**35)]), profile, {"outcome": "rejected", "reason": "quantity_insufficient"})
    matching("execution/matching/missing-quantity-unproven", "missing quantity is not zero",
             requirement(quantities=[("gpu_count", 1)]), profile, {"outcome": "unproven", "reason": "quantity_profile"})
    matching("execution/matching/sandbox-weak", "weak sandbox rejected", requirement(sandbox={"minimum":"strong"}), executor_profile(sandbox="basic"), {"outcome":"rejected","reason":"sandbox_too_weak"})
    matching("execution/matching/trust-exact", "trust domain exact match", requirement(trust={"catalog":TRUST_CATALOG,"name":"trusted.batch"}), profile, {"outcome":"proven"})
    matching("execution/matching/trust-mismatch", "trust domain mismatch", requirement(trust={"catalog":TRUST_CATALOG,"name":"isolated.batch"}), profile, {"outcome":"rejected","reason":"trust_domain_mismatch"})
    matching("execution/matching/region-exact", "execution region exact match", requirement(regions={"execution_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"}],"residency_regions":[]}), profile, {"outcome":"proven"})
    matching("execution/matching/residency-exact", "residency region exact match", requirement(regions={"execution_regions":[],"residency_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"}]}), profile, {"outcome":"proven"})
    matching("execution/matching/residency-violation", "residency is a hard constraint", requirement(regions={"execution_regions":[],"residency_regions":[{"catalog":REGION_CATALOG,"name":"us-east-1"}]}), profile, {"outcome":"rejected","reason":"residency_mismatch"})
    matching("execution/matching/runtime-mismatch", "runtime profile mismatch", requirement(runtime={"family":"native","profile":{"catalog":RUNTIME_CATALOG,"name":"abi2"},"required_features":[]}), profile, {"outcome":"rejected","reason":"runtime_profile_mismatch"})
    matching("execution/matching/abi-exact", "exact ABI pin is present", requirement(abis=[{"schema": ABI_SCHEMA}]), profile, {"outcome":"proven"})
    matching("execution/matching/abi-unproven", "missing ABI evidence is unproven", requirement(abis=[{"schema": ABI_SCHEMA}]), executor_profile(), {"outcome":"unproven","reason":"abi_profile"})
    matching("execution/matching/soft-preferences-do-not-bypass-hard", "soft preference cannot bypass hard rejection", requirement(caps=[("gpu.compute","1",[])], prefs=["lower_cost"]), profile, {"outcome":"rejected","reason":"capability_missing"})
    # Compatibility --------------------------------------------------------
    compatibility("execution/compatibility/identical", "identical requirements", requirement(quantities=[("memory_bytes",2**33)]), requirement(quantities=[("memory_bytes",2**33)]), {"feasibility":"proven","security":"proven","combined":"proven"})
    compatibility("execution/compatibility/reduces-memory", "lower memory expands feasibility", requirement(quantities=[("memory_bytes",2**32)]), requirement(quantities=[("memory_bytes",2**33)]), {"feasibility":"proven","security":"proven","combined":"proven"})
    compatibility("execution/compatibility/increases-memory", "higher memory strengthens requirement", requirement(quantities=[("memory_bytes",2**35)]), requirement(quantities=[("memory_bytes",2**33)]), {"feasibility":"violation","security":"proven","combined":"violation"})
    compatibility("execution/compatibility/adds-capability", "new capability strengthens feasibility", requirement(caps=[("gpu.compute","1",[])]), requirement(), {"feasibility":"violation","security":"proven","combined":"violation"})
    compatibility("execution/compatibility/removes-capability", "removed capability expands feasibility", requirement(), requirement(caps=[("gpu.compute","1",[])]), {"feasibility":"proven","security":"proven","combined":"proven"})
    compatibility("execution/compatibility/weakens-sandbox", "weaker sandbox is security relaxation", requirement(sandbox={"minimum":"basic"}), requirement(sandbox={"minimum":"strong"}), {"feasibility":"proven","security":"violation","combined":"violation"})
    compatibility("execution/compatibility/strengthens-sandbox", "stronger sandbox can reduce feasibility", requirement(sandbox={"minimum":"strong"}), requirement(sandbox={"minimum":"basic"}), {"feasibility":"violation","security":"proven","combined":"violation"})
    compatibility("execution/compatibility/expands-residency", "expanded residency is a security relaxation", requirement(regions={"execution_regions":[],"residency_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"},{"catalog":REGION_CATALOG,"name":"us-east-1"}]}), requirement(regions={"execution_regions":[],"residency_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"}]}), {"feasibility":"proven","security":"violation","combined":"violation"})
    compatibility("execution/compatibility/tightens-residency", "tightened residency narrows feasibility", requirement(regions={"execution_regions":[],"residency_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"}]}), requirement(regions={"execution_regions":[],"residency_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"},{"catalog":REGION_CATALOG,"name":"us-east-1"}]}), {"feasibility":"violation","security":"proven","combined":"violation"})
    compatibility("execution/compatibility/trust-change", "trust domain change is unproven", requirement(trust={"catalog":TRUST_CATALOG,"name":"isolated.batch"}), requirement(trust={"catalog":TRUST_CATALOG,"name":"trusted.batch"}), {"feasibility":"proven","security":"unproven","combined":"unproven"})
    compatibility("execution/compatibility/version-change", "capability version has no implicit ordering", requirement(caps=[("browser.automation","2",[])]), requirement(caps=[("browser.automation","1",[])]), {"feasibility":"unproven","security":"proven","combined":"unproven"})
    # Equivalence -----------------------------------------------------------
    equivalence("execution/equivalence/capability-order", "capability source order is canonical", [requirement(caps=[("browser.automation","1",[]),("model.inference","2",[])]), requirement(caps=[("model.inference","2",[]),("browser.automation","1",[])])])
    equivalence("execution/equivalence/region-order", "region source order is canonical", [requirement(regions={"execution_regions":[{"catalog":REGION_CATALOG,"name":"eu-west-1"},{"catalog":REGION_CATALOG,"name":"us-east-1"}],"residency_regions":[]}), requirement(regions={"execution_regions":[{"catalog":REGION_CATALOG,"name":"us-east-1"},{"catalog":REGION_CATALOG,"name":"eu-west-1"}],"residency_regions":[]})])
    equivalence("execution/equivalence/preference-order", "preference source order is canonical", [requirement(prefs=["lower_cost","artifact_locality"]), requirement(prefs=["artifact_locality","lower_cost"])])
    print(f"frozen {len(CASES)} cases")


if __name__ == "__main__":
    main()

