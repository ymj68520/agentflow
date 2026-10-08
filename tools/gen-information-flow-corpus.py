#!/usr/bin/env python3
"""Freezes the Phase 4A.4 information-flow conformance corpus.

INDEPENDENT implementation of the Information Flow rules
(spec/information-flow/information-flow.md): label and taint identity,
instruction authority, conservative taint propagation, the LLM strong
taint mixer, flow-kind-separated ceilings, contract canonicalization and
flow non-relaxation. The Rust implementation in crates/af-flow must
agree with the frozen expectations byte-for-byte; expected values are
NEVER derived from the Rust code.

Plumbing reuse (allowed): the independent AFIDL1 implementation supplies
the module compiler, canonical-form builder and TypeSchemaDigest
computation, and the semantic-ir corpus module supplies the ACE1 value
encoder. The flow ORACLE logic below is implemented here from scratch and
shares nothing with crates/af-flow.

Run from the repository root:  python tools/gen-information-flow-corpus.py
"""

import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "information-flow"
CORPUS_REVISION = 1

# Plumbing: the semantic-ir corpus module carries the ACE1 encoder and the
# independent AFIDL1 compiler (which it loads as `afidl`).
_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
EFFECT_AFDL = ROOT / "spec" / "idl" / "effect-signature.afidl"
FLOW_AFDL = ROOT / "spec" / "idl" / "information-flow.afidl"

# Stable error kinds (spec/information-flow/information-flow.md section 17).
INVALID_INFORMATION_LABEL = "invalid_information_label"
UNKNOWN_INFORMATION_LABEL = "unknown_information_label"
DUPLICATE_INFORMATION_LABEL = "duplicate_information_label"
INVALID_FLOW_SOURCE = "invalid_flow_source"
INVALID_FLOW_SINK = "invalid_flow_sink"
UNKNOWN_FLOW_SINK = "unknown_flow_sink"
DUPLICATE_FLOW_CONSTRAINT = "duplicate_flow_constraint"
FLOW_NOT_PROVEN = "flow_not_proven"
FLOW_CONTRACT_RELAXED = "flow_contract_relaxed"
PRIVILEGED_TRANSFORMATION_REQUIRED = "privileged_transformation_required"


# Authority scale (index = strength).
AUTHORITY = ["none", "external_data", "delegated_agent", "user",
             "workflow", "platform"]

# ---------------------------------------------------------------------------
# Fixture catalogs (explicit, immutable, frozen)
# ---------------------------------------------------------------------------

LABEL_CATALOG = "agentflow.infoflow.labels@1"
TAINT_CATALOG = "agentflow.infoflow.taints@1"
EFFECT_CATALOG = "agentflow.test.effects@1"

CONFIDENTIALITY_LABELS = [
    (LABEL_CATALOG, "pii.email"),
    (LABEL_CATALOG, "pii.phone"),
    (LABEL_CATALOG, "internal.confidential"),
    (LABEL_CATALOG, "medical.record"),
]
TAINT_LABELS = [
    (TAINT_CATALOG, "web_untrusted"),
    (TAINT_CATALOG, "user_input"),
    (TAINT_CATALOG, "model_generated"),
    (TAINT_CATALOG, "file_system"),
]
# Decision-capable operations, classified EXPLICITLY by the catalog (never
# by name inspection): deploy carries decisions, issue_comment does not,
# and the name-based trap operations prove classification is catalog-only.
EFFECT_OPERATIONS = [
    (EFFECT_CATALOG, "production.deploy", True),
    (EFFECT_CATALOG, "payment.refund", True),
    (EFFECT_CATALOG, "github.issue_comment", False),
    (EFFECT_CATALOG, "email.send", False),
]

MODEL_GENERATED = (TAINT_CATALOG, "model_generated")

CONF_SET = set(CONFIDENTIALITY_LABELS)
TAINT_SET = set(TAINT_LABELS)
EFFECT_SET = {(c, n) for c, n, _ in EFFECT_OPERATIONS}
DECISION_SET = {(c, n) for c, n, d in EFFECT_OPERATIONS if d}


def fixture_catalogs():
    """The frozen catalog blocks embedded in every corpus case."""
    return {
        "labels": {
            "confidentiality": [{"catalog": c, "name": n}
                                for c, n in sorted(CONF_SET)],
            "taints": [{"catalog": c, "name": n} for c, n in sorted(TAINT_SET)],
        },
        "effects": [{"catalog": c, "name": n, "decision_capable": d}
                    for c, n, d in sorted(EFFECT_OPERATIONS)],
    }


# ---------------------------------------------------------------------------
# Shapes (local walker over the flow module set; ACE1 plumbing from sir)
# ---------------------------------------------------------------------------

def load_flow_types():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("effect-signature.afidl", EFFECT_AFDL.read_text(encoding="utf-8")),
        ("information-flow.afidl", FLOW_AFDL.read_text(encoding="utf-8")),
    ]
    _modules, types_by_module = afidl.compile_module_set(sources)
    out = {}
    for spec, types in types_by_module.items():
        for schema in types:
            out[f"{spec}/{schema['name']}"] = schema
    return out


FLOW_TYPES = load_flow_types()


def shape_of_identity(identity):
    kind, payload = FLOW_TYPES[identity]["body"]
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
    raise AssertionError(f"unknown body kind {kind!r}")


def expr_shape(texpr):
    kind, value = texpr
    if kind == "primitive":
        return sir.PRIM_SHAPES[value]
    if kind == "ref":
        return shape_of_identity(value)
    if kind == "list":
        return ("list", expr_shape(value))
    if kind == "map_string":
        return ("map", expr_shape(value))
    if kind == "option":
        return ("option", expr_shape(value))
    raise AssertionError(f"unknown type expr {kind!r}")


CONTRACT_SHAPE = shape_of_identity(
    "agentflow.information.flow@1/InformationFlowContract")


def to_value_tree(shape, value):
    """Converts serde-form wire values (unit variants as bare strings)
    into the ACE1 value-tree form the encoder consumes."""
    kind = shape[0]
    if kind == "variant":
        if isinstance(value, str):
            return {value: None}
        case, payload = next(iter(value.items()))
        case_shape = dict(shape[2])[case]
        return {case: to_value_tree(case_shape, payload)}
    if kind == "record":
        return {name: to_value_tree(field_shape, value[name])
                for name, field_shape in shape[2]}
    if kind == "list":
        return [to_value_tree(shape[1], item) for item in value]
    return value


def canonical_ace1(contract):
    """The canonical ACE1 text of a canonical flow contract."""
    tree = to_value_tree(CONTRACT_SHAPE, contract)
    return sir.encode_value(CONTRACT_SHAPE,
                            sir.json_to_value(CONTRACT_SHAPE, tree))


# ---------------------------------------------------------------------------
# The oracle: identity, validation, canonicalization
# ---------------------------------------------------------------------------

class FlowOracleError(Exception):
    def __init__(self, kind, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind
        self.detail = detail


NAMESPACE_CHARS = set("abcdefghijklmnopqrstuvwxyz0123456789-")


def is_specification_id(text):
    if "@" not in text:
        return False
    namespace, _, major = text.rpartition("@")
    if not namespace or not major:
        return False
    for segment in namespace.split("."):
        if not segment or segment[0] not in "abcdefghijklmnopqrstuvwxyz":
            return False
        if not all(c.isascii() and c in NAMESPACE_CHARS for c in segment):
            return False
    return major.isascii() and major.isdigit() and major[0] != "0"


def nfc(text):
    return unicodedata.normalize("NFC", text)


def check_label(catalog, name):
    if not is_specification_id(catalog):
        raise FlowOracleError(
            INVALID_INFORMATION_LABEL,
            f"label catalog {catalog!r} is not a SpecificationId")
    normalized = nfc(name)
    if not normalized or any(c.isspace() for c in normalized):
        raise FlowOracleError(
            INVALID_INFORMATION_LABEL,
            f"label name {name!r} is not a valid label path")


def resolve_confidentiality(label):
    check_label(label["catalog"], label["name"])
    key = (label["catalog"], nfc(label["name"]))
    if key not in CONF_SET:
        raise FlowOracleError(
            UNKNOWN_INFORMATION_LABEL,
            f"confidentiality label {key[0]}/{key[1]} is not registered "
            "(case-sensitive, no auto-registration)")


def resolve_taint(taint):
    check_label(taint["catalog"], taint["name"])
    key = (taint["catalog"], nfc(taint["name"]))
    if key not in TAINT_SET:
        raise FlowOracleError(
            UNKNOWN_INFORMATION_LABEL,
            f"taint {key[0]}/{key[1]} is not registered (case-sensitive, "
            "no auto-registration)")


def check_sink(sink):
    if "workflow_output" in sink:
        port = sink["workflow_output"]
        if port == 0:
            raise FlowOracleError(
                INVALID_FLOW_SINK, "output port 0 is invalid")
        return "out", port
    operation = sink["external_effect"]
    key = (operation["catalog"], operation["name"])
    if not operation["catalog"] or not operation["name"]:
        raise FlowOracleError(INVALID_FLOW_SINK, "effect sink identity empty")
    if key not in EFFECT_SET:
        raise FlowOracleError(
            UNKNOWN_FLOW_SINK,
            f"effect sink {key[0]}/{key[1]} does not resolve in the "
            "supplied catalog universe (no name-based classification)")
    return "effect", f"{key[0]}/{key[1]}"


def source_sort_key(source):
    """The frozen canonical source order: category case number first
    (workflow_input < resource_requirement < unknown_external), then the
    port number numerically."""
    category = source["category"]
    if isinstance(category, str):
        assert category == "unknown_external"
        return (3, 0, "")
    if "workflow_input" in category:
        return (1, category["workflow_input"], "")
    if "resource_requirement" in category:
        return (2, category["resource_requirement"], "")
    raise AssertionError(f"unknown source category {category!r}")


def check_source(source):
    category = source["category"]
    if isinstance(category, str):
        assert category == "unknown_external"
        return ("unknown", None)
    if "workflow_input" in category:
        if category["workflow_input"] == 0:
            raise FlowOracleError(
                INVALID_FLOW_SOURCE, "source port 0 is invalid")
        return ("wf", category["workflow_input"])
    if "resource_requirement" in category:
        if category["resource_requirement"] == 0:
            raise FlowOracleError(
                INVALID_FLOW_SOURCE, "source port 0 is invalid")
        return ("rr", category["resource_requirement"])
    return ("unknown", None)


def canonicalize(contract):
    """Structural canonicalization with full catalog resolution."""
    seen_labels = set()
    restrictions = []
    for label in contract["confidentiality_restrictions"]:
        resolve_confidentiality(label)
        key = (label["catalog"], nfc(label["name"]))
        if key in seen_labels:
            raise FlowOracleError(
                DUPLICATE_INFORMATION_LABEL,
                f"confidentiality label {key[0]}/{key[1]} is declared twice")
        seen_labels.add(key)
        restrictions.append(key)
    restrictions.sort(key=lambda k: (k[0].encode(), k[1].encode()))

    seen_sources = set()
    for source in contract["summary"]["sources"]:
        seen_sources.add(check_source(source))

    seen_taints = set()
    taints = []
    for taint in contract["summary"]["taints"]:
        resolve_taint(taint)
        key = (taint["catalog"], nfc(taint["name"]))
        if key in seen_taints:
            raise FlowOracleError(
                DUPLICATE_INFORMATION_LABEL,
                f"taint {key[0]}/{key[1]} is declared twice")
        seen_taints.add(key)
        taints.append(key)
    taints.sort(key=lambda k: (k[0].encode(), k[1].encode()))

    seen_constraints = set()
    constraints = []
    for constraint in contract["flow_constraints"]:
        kind_, sink_key = check_sink(constraint["sink"])
        pair = (constraint["kind"], kind_, sink_key)
        if pair in seen_constraints:
            raise FlowOracleError(
                DUPLICATE_FLOW_CONSTRAINT,
                "the same (flow kind, sink) ceiling is declared twice")
        seen_constraints.add(pair)
        constraints.append((constraint["kind"], constraint["sink"]))
    def constraint_sort_key(c):
        sink = c[1]
        if "workflow_output" in sink:
            return (c[0], 0, sink["workflow_output"], "")
        operation = sink["external_effect"]
        return (c[0], 1, 0, (operation["catalog"], operation["name"]))

    constraints.sort(key=constraint_sort_key)

    return {
        "confidentiality_restrictions": [
            {"catalog": c, "name": n} for c, n in restrictions],
        "summary": {
            "sources": sorted(contract["summary"]["sources"],
                              key=source_sort_key),
            "taints": [{"catalog": c, "name": n} for c, n in taints],
            "instruction_authority":
                contract["summary"]["instruction_authority"],
            "influence_unknown": contract["summary"]["influence_unknown"],
        },
        "flow_constraints": [
            {"kind": kind_, "sink": sink} for kind_, sink in constraints],
    }


def validate(contract):
    canonical = canonicalize(contract)
    # Rule duplicates inside the summary source list are structural, not
    # errors (the summary is a set view); label lists above are the
    # declared-set lists where duplicates reject.
    return canonical


# ---------------------------------------------------------------------------
# Propagation oracle
# ---------------------------------------------------------------------------

def taint_key(taint):
    return (taint["catalog"], nfc(taint["name"]))


def union_taints(sets):
    merged = {taint_key(t) for set_ in sets for t in set_}
    return [{"catalog": c, "name": n} for c, n in sorted(merged)]


def merge_summaries(summaries):
    seen = {}
    for summary in summaries:
        for source in summary["sources"]:
            seen[json.dumps(source, sort_keys=True)] = source
    authority = max(AUTHORITY.index(s["instruction_authority"])
                    for s in summaries)
    return {
        "sources": sorted(seen.values(), key=source_sort_key),
        "taints": union_taints([s["taints"] for s in summaries]),
        "instruction_authority": AUTHORITY[authority],
        "influence_unknown": any(s["influence_unknown"] for s in summaries),
    }


def propagate_pure_expression(inputs, output_authority):
    """The output authority is the MAXIMUM of the input authorities -
    inherited, never claimed. A claimed output authority below the merged
    maximum is rejected: an ordinary transformation cannot elevate
    instruction authority, and claiming less than what flows through
    would silently launder authority."""
    merged = merge_summaries(inputs)
    if AUTHORITY.index(output_authority) > AUTHORITY.index(
            merged["instruction_authority"]):
        raise FlowOracleError(
            INVALID_FLOW_SOURCE,
            "an ordinary data transformation cannot elevate instruction "
            f"authority: inputs carry at most "
            f"{merged['instruction_authority']} but the output claims "
            f"{output_authority}")
    return {**merged, "instruction_authority":
            merged["instruction_authority"]}


def propagate_unknown_transformation(possible_inputs):
    merged = merge_summaries(possible_inputs)
    return {**merged, "influence_unknown": True}


def mix_model_visible_inputs(model_visible_inputs):
    merged = merge_summaries(model_visible_inputs)
    marker = {"catalog": MODEL_GENERATED[0], "name": MODEL_GENERATED[1]}
    if taint_key(marker) not in TAINT_SET:
        raise FlowOracleError(
            UNKNOWN_INFORMATION_LABEL,
            "the model_generated taint is not registered")
    merged["taints"] = union_taints([merged["taints"], [marker]])
    merged["influence_unknown"] = True
    return merged


# ---------------------------------------------------------------------------
# Static flow judgment oracle
# ---------------------------------------------------------------------------

def sink_identity(sink):
    if "workflow_output" in sink:
        return ("out", sink["workflow_output"])
    operation = sink["external_effect"]
    return ("effect", operation["catalog"], operation["name"])


def check_flow(flow_contract, kind_, sink):
    """The static flow judgment: missing ceiling fails closed, unknown
    influence is Unproven, egress under declared restrictions carries a
    declassification obligation, tainted decision influence carries an
    endorsement obligation, and unclassified decision sinks are Unproven."""
    canonical = canonicalize(flow_contract)
    target = sink_identity(sink)
    if not any(c["kind"] == kind_ and sink_identity(c["sink"]) == target
               for c in canonical["flow_constraints"]):
        raise FlowOracleError(
            FLOW_NOT_PROVEN,
            "no declared ceiling covers this (flow kind, sink): a missing "
            "flow proof is never flow allowed")
    if canonical["summary"]["influence_unknown"]:
        return {"outcome": "unproven"}
    requires_dec = (kind_ == "data_egress"
                    and bool(canonical["confidentiality_restrictions"]))
    if kind_ == "decision_influence":
        if "external_effect" in sink:
            operation = sink["external_effect"]
            if ((operation["catalog"], operation["name"])
                    not in DECISION_SET):
                return {"outcome": "unproven"}
        if canonical["summary"]["taints"]:
            return {"outcome": "requires_trusted_transformation",
                    "requires_declassification": False,
                    "requires_endorsement": True}
    if requires_dec:
        return {"outcome": "requires_trusted_transformation",
                "requires_declassification": True,
                "requires_endorsement": False}
    return {"outcome": "proven_within_declared_ceiling"}


def require_proven(verdict):
    """Demands a proven flow, failing closed."""
    if verdict["outcome"] == "proven_within_declared_ceiling":
        return
    if verdict["outcome"] == "requires_trusted_transformation":
        raise FlowOracleError(
            PRIVILEGED_TRANSFORMATION_REQUIRED,
            "the declared flow needs a declassification or endorsement "
            "this static layer cannot supply")
    raise FlowOracleError(
        FLOW_NOT_PROVEN,
        "the static flow relationship could not be proven: Unproven is "
        "never judged allowed")


# ---------------------------------------------------------------------------
# Non-relaxation oracle
# ---------------------------------------------------------------------------

def canonical_key_set(contract):
    """The comparable identity sets of a canonicalized contract."""
    c = canonicalize(contract)
    restrictions = {(l["catalog"], l["name"])
                    for l in c["confidentiality_restrictions"]}
    per_kind = {"data_egress": set(), "decision_influence": set()}
    for constraint in c["flow_constraints"]:
        sink = constraint["sink"]
        if "workflow_output" in sink:
            key = ("out", sink["workflow_output"])
        else:
            key = ("effect", sink["external_effect"]["catalog"],
                   sink["external_effect"]["name"])
        per_kind[constraint["kind"]].add(key)
    return c, restrictions, per_kind


def flow_not_relaxed(new_contract, old_contract):
    """Returns ("proven", None) or ("unproven", reason); proven
    relaxation raises."""
    _, new_restrictions, new_per_kind = canonical_key_set(new_contract)
    _, old_restrictions, old_per_kind = canonical_key_set(old_contract)

    if not old_restrictions <= new_restrictions:
        raise FlowOracleError(
            FLOW_CONTRACT_RELAXED,
            "the new contract dropped a confidentiality restriction")
    for kind_ in ("data_egress", "decision_influence"):
        if not new_per_kind[kind_] <= old_per_kind[kind_]:
            raise FlowOracleError(
                FLOW_CONTRACT_RELAXED,
                f"the new contract declares a {kind_} sink the old "
                "contract did not permit")

    new_summary = canonicalize(new_contract)["summary"]
    old_summary = canonicalize(old_contract)["summary"]
    if AUTHORITY.index(new_summary["instruction_authority"]) > \
            AUTHORITY.index(old_summary["instruction_authority"]):
        raise FlowOracleError(
            FLOW_CONTRACT_RELAXED,
            "the new contract claims a higher instruction authority")

    if old_summary["influence_unknown"] and \
            not new_summary["influence_unknown"]:
        return ("unproven", "influence_relation_unknown")

    def source_keys(summary):
        keys = set()
        for source in summary["sources"]:
            category = source["category"]
            if isinstance(category, str):
                keys.add(("unknown",))
            elif "workflow_input" in category:
                keys.add(("wf", category["workflow_input"]))
            elif "resource_requirement" in category:
                keys.add(("rr", category["resource_requirement"]))
            else:
                keys.add(("unknown",))
        return keys

    new_sources = source_keys(new_summary)
    old_sources = source_keys(old_summary)
    if ("unknown",) in old_sources:
        old_is_top = True
    else:
        old_is_top = False
    if not old_is_top and ("unknown",) not in new_sources:
        if not old_sources <= new_sources:
            return ("unproven", "influence_relation_unknown")
    return ("proven", None)


# ---------------------------------------------------------------------------
# Fixture builders (serde-form dicts)
# ---------------------------------------------------------------------------

def conf(name, catalog=LABEL_CATALOG):
    return {"catalog": catalog, "name": name}


def taint(name, catalog=TAINT_CATALOG):
    return {"catalog": catalog, "name": name}


def source_workflow_input(port):
    return {"category": {"workflow_input": port}}


def source_resource(port):
    return {"category": {"resource_requirement": port}}


def source_unknown():
    return {"category": "unknown_external"}


def effect_sink(catalog, name):
    return {"external_effect": {"catalog": catalog, "name": name}}


def output_sink(port):
    return {"workflow_output": port}


def summary(sources=(), taints=(), authority="none", unknown=False):
    return {
        "sources": list(sources),
        "taints": list(taints),
        "instruction_authority": authority,
        "influence_unknown": unknown,
    }


def contract(restrictions=(), contract_summary=None, constraints=()):
    return {
        "confidentiality_restrictions": list(restrictions),
        "summary": contract_summary if contract_summary is not None
        else summary(unknown=True),
        "flow_constraints": list(constraints),
    }


def constraint(kind_, sink):
    return {"kind": kind_, "sink": sink}


# ---------------------------------------------------------------------------
# Corpus assembly
# ---------------------------------------------------------------------------

CASES = []


def write_case(subdir, payload):
    path = CONF / subdir / f"{payload['case_id'].replace('/', '-')}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(payload, indent=2, ensure_ascii=False, sort_keys=True)
    path.write_text(text + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {path.relative_to(ROOT)}")
    CASES.append(payload["case_id"])


def base_case(case_kind, case_id, note):
    return {
        "kind": "agentflow.conformance.information-flow.case",
        "case_kind": case_kind,
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "catalogs": fixture_catalogs(),
    }


def golden(case_id, note, flow_contract, verdicts=()):
    canonical = validate(flow_contract)
    frozen_verdicts = []
    for verdict in verdicts:
        outcome = check_flow(flow_contract, verdict["kind"], verdict["sink"])
        if outcome != verdict["expected"]:
            raise AssertionError(
                f"{case_id}: verdict {outcome} drifted from frozen "
                f"{verdict['expected']}")
        frozen_verdicts.append(verdict)
    payload = base_case("golden", case_id, note)
    payload.update({
        "contract": flow_contract,
        "canonical_ace1_utf8": canonical_ace1(canonical),
        "is_empty": not canonical["confidentiality_restrictions"]
        and not canonical["flow_constraints"],
        "restriction_count": len(canonical["confidentiality_restrictions"]),
        "constraint_count": len(canonical["flow_constraints"]),
        "verdicts": frozen_verdicts,
    })
    write_case("golden", payload)


def negative(case_id, note, payload):
    """A validation rejection: the oracle must raise exactly this kind."""
    expected = payload["expected_kind"]
    if payload.get("operation") == "check_flow":
        try:
            check_flow(payload["contract"], payload["kind"], payload["sink"])
        except FlowOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    elif payload.get("operation") == "require_proven":
        try:
            verdict = check_flow(payload["contract"], payload["kind"],
                                 payload["sink"])
            require_proven(verdict)
        except FlowOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    elif payload.get("operation") == "propagate_pure_expression":
        try:
            propagate_pure_expression(
                [summary(sources=i["sources"], taints=i["taints"],
                         authority=i["authority"],
                         unknown=i["influence_unknown"])
                 for i in payload["inputs"]],
                payload["output_authority"])
        except FlowOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    else:
        try:
            validate(payload["contract"])
        except FlowOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    payload = {**base_case("negative", case_id, note), **payload}
    write_case("negative", payload)


def propagation(case_id, note, payload):
    """A propagation/mixer case with a frozen outcome summary."""
    operation = payload["operation"]
    if operation == "union_taints":
        result = union_taints(payload["operand_sets"])
        frozen = {"taints": result}
        payload = {**base_case("propagation", case_id, note), **payload,
                   **frozen}
        write_case("propagation", payload)
        return

    inputs = payload["inputs"]
    def build(entry):
        return summary(sources=entry["sources"], taints=entry["taints"],
                       authority=entry["authority"],
                       unknown=entry["influence_unknown"])

    if operation == "propagate_pure_expression":
        merged = propagate_pure_expression([build(i) for i in inputs],
                                           payload["output_authority"])
        frozen = {"summary": merged}
    elif operation == "propagate_unknown_transformation":
        merged = propagate_unknown_transformation([build(i) for i in inputs])
        frozen = {"summary": merged}
    elif operation == "mix_model_visible_inputs":
        merged = mix_model_visible_inputs([build(i) for i in inputs])
        frozen = {"summary": merged}
    else:
        raise AssertionError(f"unknown propagation operation {operation!r}")
    # Cross-check the frozen result against the oracle expectation stated
    # in the payload (kept declarative so the file documents itself).
    payload = {**base_case("propagation", case_id, note), **payload,
               **frozen}
    write_case("propagation", payload)


def compatibility(case_id, note, payload):
    """A non-relaxation or flow-proof judgment; the frozen expectation is
    checked against the oracle before freezing."""
    expected = payload["expected"]
    if payload.get("operation") == "flow_not_relaxed":
        try:
            verdict = flow_not_relaxed(payload["new_contract"],
                                       payload["old_contract"])
        except FlowOracleError as error:
            # The only expected raise here is the fail-closed rejection of
            # an unresolvable label inside one of the two contracts.
            if (expected["outcome"] == "unknown_label_in_new_catalog"
                    and error.kind == UNKNOWN_INFORMATION_LABEL):
                pass
            else:
                raise AssertionError(
                    f"{case_id}: unexpected oracle rejection "
                    f"{error.kind}") from error
        else:
            if verdict != (expected["outcome"], expected.get("reason")):
                raise AssertionError(
                    f"{case_id}: oracle verdict {verdict}, "
                    f"expected {expected}")
    payload = {**base_case("compatibility", case_id, note), **payload}
    write_case("compatibility", payload)


def compatibility_relaxed(case_id, note, payload):
    """A compatibility case whose correct outcome is the proven
    `flow_contract_relaxed` rejection."""
    try:
        verdict = flow_not_relaxed(payload["new_contract"],
                                   payload["old_contract"])
    except FlowOracleError as error:
        if error.kind != FLOW_CONTRACT_RELAXED:
            raise AssertionError(
                f"{case_id}: oracle raised {error.kind}, expected "
                f"{FLOW_CONTRACT_RELAXED}") from error
    else:
        raise AssertionError(
            f"{case_id}: expected relaxation rejection, oracle returned "
            f"{verdict}")
    frozen = {**base_case("compatibility", case_id, note), **payload}
    frozen["expected"] = {"outcome": "relaxed"}
    write_case("compatibility", frozen)


def equivalence(case_id, note, members, frozen_bytes):
    for member in members:
        canonical = validate(member)
        if canonical_ace1(canonical) != frozen_bytes:
            raise AssertionError(
                f"{case_id}: equivalence member drifted off the frozen "
                "canonical bytes")
    payload = base_case("equivalence", case_id, note)
    payload.update({
        "members": members,
        "canonical_ace1_utf8": frozen_bytes,
    })
    write_case("equivalence", payload)


def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()

    pii_email = lambda: conf("pii.email")
    internal = lambda: conf("internal.confidential")
    web = lambda: taint("web_untrusted")
    user_input = lambda: taint("user_input")

    # --- Golden ----------------------------------------------------------

    golden("infoflow/golden/empty-flow-contract",
           "the empty contract declares no restriction and no ceiling; "
           "its summary is the conservative unknown, NOT a trusted state",
           contract())

    golden("infoflow/golden/single-confidentiality-label",
           "one confidentiality restriction, nominally pinned",
           contract(restrictions=[pii_email()]))

    golden("infoflow/golden/multiple-labels",
           "several confidentiality restrictions",
           contract(restrictions=[pii_email(), conf("pii.phone"),
                                  internal()]))

    golden("infoflow/golden/source-taint-summary",
           "a conservative influence summary: typed sources, taints "
           "distinct from confidentiality labels, explicit authority",
           contract(
               contract_summary=summary(
                   sources=[source_workflow_input(1), source_resource(2),
                            source_unknown()],
                   taints=[web(), user_input()],
                   authority="external_data",
                   unknown=True),
               restrictions=[internal()]))

    golden("infoflow/golden/instruction-authority-label",
           "the closed instruction-authority scale in a summary",
           contract(
               contract_summary=summary(
                   sources=[source_workflow_input(1)],
                   authority="user",
                   unknown=False)))

    golden("infoflow/golden/data-egress-sink",
           "a data-egress ceiling to a structured external effect: with a "
           "known summary and a declared restriction the flow carries a "
           "recorded declassification obligation",
           contract(
               restrictions=[pii_email()],
               contract_summary=summary(
                   sources=[source_workflow_input(1)],
                   authority="user",
                   unknown=False),
               constraints=[constraint("data_egress",
                                       effect_sink(EFFECT_CATALOG,
                                                   "email.send"))]),
           verdicts=[{"kind": "data_egress",
                      "sink": effect_sink(EFFECT_CATALOG, "email.send"),
                      "expected": {
                          "outcome": "requires_trusted_transformation",
                          "requires_declassification": True,
                          "requires_endorsement": False}}])

    golden("infoflow/golden/decision-influence-sink",
           "a decision-influence ceiling to a decision-capable effect: "
           "tainted influence into the decision carries an endorsement "
           "obligation",
           contract(
               contract_summary=summary(sources=[source_unknown()],
                                        taints=[web()],
                                        authority="external_data",
                                        unknown=False),
               constraints=[constraint("decision_influence",
                                       effect_sink(EFFECT_CATALOG,
                                                   "production.deploy"))]),
           verdicts=[{"kind": "decision_influence",
                      "sink": effect_sink(EFFECT_CATALOG,
                                          "production.deploy"),
                      "expected": {
                          "outcome": "requires_trusted_transformation",
                          "requires_declassification": False,
                          "requires_endorsement": True}}])

    golden("infoflow/golden/conservative-source-influence",
           "the summary may list several sources including the explicit "
           "unknown external origin",
           contract(
               contract_summary=summary(
                   sources=[source_workflow_input(1),
                            source_workflow_input(2), source_unknown()],
                   taints=[user_input()],
                   unknown=True)))

    golden("infoflow/golden/unknown-influence",
           "influence_unknown = true is an explicit Top value, preserved "
           "by canonicalization; its flow judgment is Unproven even under "
           "a declared ceiling",
           contract(
               contract_summary=summary(taints=[web()], unknown=True),
               constraints=[constraint("data_egress", output_sink(1))]),
           verdicts=[{"kind": "data_egress", "sink": output_sink(1),
                      "expected": {"outcome": "unproven"}}])

    # --- Negative --------------------------------------------------------

    negative("infoflow/negative/unknown-label",
             "an unregistered confidentiality label is rejected: no "
             "auto-registration, no case-insensitive matching",
             {"contract": contract(restrictions=[conf("pii.Email")]),
              "expected_kind": UNKNOWN_INFORMATION_LABEL})

    negative("infoflow/negative/duplicate-normalized-label",
             "the same nominal label declared twice is rejected: a "
             "declared list is a set",
             {"contract": contract(restrictions=[pii_email(), pii_email()]),
              "expected_kind": DUPLICATE_INFORMATION_LABEL})

    negative("infoflow/negative/invalid-source-port",
             "source port 0 is invalid: ports are positive",
             {"contract": contract(
                  contract_summary=summary(sources=[source_workflow_input(0)],
                                           unknown=True)),
              "expected_kind": INVALID_FLOW_SOURCE})

    negative("infoflow/negative/invalid-sink-port",
             "output sink port 0 is invalid: ports are positive",
             {"contract": contract(
                  constraints=[constraint("data_egress",
                                          output_sink(0))]),
              "expected_kind": INVALID_FLOW_SINK})

    negative("infoflow/negative/unknown-sink-operation",
             "an unresolvable effect sink is rejected fail-closed: no "
             "latest fallback, no name-based classification",
             {"contract": contract(
                  constraints=[constraint("data_egress",
                                          effect_sink(EFFECT_CATALOG,
                                                      "database.drop"))]),
              "expected_kind": UNKNOWN_FLOW_SINK})

    negative("infoflow/negative/label-silently-removed",
             "a taint referencing an unregistered identity is rejected "
             "rather than silently dropped by canonicalization",
             {"contract": contract(
                  contract_summary=summary(taints=[taint("ghost_taint")],
                                           unknown=True)),
              "expected_kind": UNKNOWN_INFORMATION_LABEL})

    negative("infoflow/negative/unknown-taint-treated-as-clean",
             "an unknown influence set may not be judged clean: under a "
             "declared ceiling the verdict is Unproven, and demanding a "
             "proven flow fails closed",
             {"operation": "require_proven",
              "contract": contract(
                  contract_summary=summary(taints=[web()], unknown=True),
                  constraints=[constraint("data_egress", output_sink(1))]),
              "kind": "data_egress",
              "sink": output_sink(1),
              "expected_kind": FLOW_NOT_PROVEN})

    negative("infoflow/negative/untrusted-promoted-to-instruction",
             "an ordinary transformation cannot elevate instruction "
             "authority: web-untrusted inputs may not flow into an "
             "output claiming workflow authority",
             {"operation": "propagate_pure_expression",
              "inputs": [
                  {"sources": [source_unknown()],
                   "taints": [web()], "authority": "external_data",
                   "influence_unknown": True},
              ],
              "output_authority": "workflow",
              "expected_kind": INVALID_FLOW_SOURCE})

    negative("infoflow/negative/flow-asserted-allowed-without-proof",
             "a flow with no declared ceiling fails closed: missing flow "
             "proof is never flow allowed",
             {"operation": "check_flow",
              "contract": contract(restrictions=[pii_email()]),
              "kind": "data_egress",
              "sink": output_sink(1),
              "expected_kind": FLOW_NOT_PROVEN})

    negative("infoflow/negative/missing-privileged-transformation",
             "egress under a declared confidentiality restriction needs a "
             "declassification this layer cannot supply: the obligation is "
             "recorded and demanding a proven flow names the missing "
             "privileged transformation",
             {"operation": "require_proven",
              "contract": contract(
                  restrictions=[conf("medical.record")],
                  contract_summary=summary(
                      sources=[source_workflow_input(1)],
                      authority="workflow",
                      unknown=False),
                  constraints=[constraint("data_egress", output_sink(1))]),
              "kind": "data_egress",
              "sink": output_sink(1),
              "expected_kind": PRIVILEGED_TRANSFORMATION_REQUIRED})

    # --- Propagation -----------------------------------------------------

    propagation("infoflow/propagation/taint-union-identity",
                "the empty taint set is the union identity",
                {"operation": "union_taints",
                 "operand_sets": [[], [web()]],
                 "taints": [taint("web_untrusted")]})

    propagation("infoflow/propagation/taint-union-commutative",
                "union is commutative",
                {"operation": "union_taints",
                 "operand_sets": [[user_input()], [web()]],
                 "taints": [taint("web_untrusted"),
                            taint("user_input")]})

    propagation("infoflow/propagation/taint-union-associative",
                "union is associative (frozen over the flat result)",
                {"operation": "union_taints",
                 "operand_sets": [[web()], [user_input()],
                                  [taint("file_system")]],
                 "taints": [taint("web_untrusted"),
                            taint("user_input"),
                            taint("file_system")]})

    propagation("infoflow/propagation/taint-union-idempotent",
                "union is idempotent",
                {"operation": "union_taints",
                 "operand_sets": [[web(), user_input()],
                                  [web(), user_input()]],
                 "taints": [taint("web_untrusted"),
                            taint("user_input")]})

    propagation("infoflow/propagation/pure-expression-inherits-taints",
                "a pure expression inherits the union of ALL input taints "
                "and the maximum authority; unknown stays unknown",
                {"operation": "propagate_pure_expression",
                 "inputs": [
                     {"sources": [source_workflow_input(1)],
                      "taints": [web()], "authority": "external_data",
                      "influence_unknown": False},
                     {"sources": [source_workflow_input(2)],
                      "taints": [user_input()], "authority": "user",
                      "influence_unknown": True},
                 ],
                 "output_authority": "user",
                 "summary": {
                     "sources": [source_workflow_input(1),
                                 source_workflow_input(2)],
                     "taints": [user_input(), web()],
                     "instruction_authority": "user",
                     "influence_unknown": True}})

    propagation("infoflow/propagation/unknown-influence-remains-unknown",
                "an unknown transformation conservatively unions every "
                "possible input and stays unknown",
                {"operation": "propagate_unknown_transformation",
                 "inputs": [
                     {"sources": [source_workflow_input(1)],
                      "taints": [web()], "authority": "external_data",
                      "influence_unknown": False},
                 ],
                 "summary": {
                     "sources": [source_workflow_input(1)],
                     "taints": [web()],
                     "instruction_authority": "external_data",
                     "influence_unknown": True}})

    propagation("infoflow/propagation/model-output-inherits-every-input-taint",
                "the strong mixer: model output taint = union of every "
                "model-visible input taint",
                {"operation": "mix_model_visible_inputs",
                 "inputs": [
                     {"sources": [source_workflow_input(1)],
                      "taints": [web()], "authority": "external_data",
                      "influence_unknown": False},
                     {"sources": [source_resource(2)],
                      "taints": [taint("file_system")],
                      "authority": "delegated_agent",
                      "influence_unknown": False},
                 ],
                 "summary": {
                     "sources": [source_resource(2),
                                 source_workflow_input(1)],
                     "taints": [taint("file_system"), web()],
                     "instruction_authority": "delegated_agent",
                     "influence_unknown": True}})

    propagation("infoflow/propagation/model-output-adds-model-generated",
                "the mixer adds the registered model_generated taint",
                {"operation": "mix_model_visible_inputs",
                 "inputs": [
                     {"sources": [source_workflow_input(1)],
                      "taints": [], "authority": "none",
                      "influence_unknown": False},
                 ],
                 "summary": {
                     "sources": [source_workflow_input(1)],
                     "taints": [taint("model_generated",
                                     catalog=TAINT_CATALOG)],
                     "instruction_authority": "none",
                     "influence_unknown": True}})

    propagation("infoflow/propagation/memory-persistence-retains-taint",
                "write memory / read memory round trip retains taint: "
                "persistence is an ordinary transformation",
                {"operation": "propagate_pure_expression",
                 "inputs": [
                     {"sources": [source_unknown()],
                      "taints": [web()], "authority": "external_data",
                      "influence_unknown": True},
                 ],
                 "output_authority": "external_data",
                 "summary": {
                     "sources": [source_unknown()],
                     "taints": [web()],
                     "instruction_authority": "external_data",
                     "influence_unknown": True}})

    propagation("infoflow/propagation/summarization-does-not-endorse",
                "summarizing untrusted input does not endorse: taint "
                "survives, authority unchanged, unknown preserved",
                {"operation": "propagate_pure_expression",
                 "inputs": [
                     {"sources": [source_unknown()],
                      "taints": [web()], "authority": "external_data",
                      "influence_unknown": True},
                 ],
                 "output_authority": "external_data",
                 "summary": {
                     "sources": [source_unknown()],
                     "taints": [web()],
                     "instruction_authority": "external_data",
                     "influence_unknown": True}})

    propagation("infoflow/propagation/schema-validation-does-not-endorse",
                "schema validation is an ordinary transformation: it "
                "cannot clean taints or raise authority",
                {"operation": "propagate_pure_expression",
                 "inputs": [
                     {"sources": [source_workflow_input(3)],
                      "taints": [user_input()],
                      "authority": "external_data",
                      "influence_unknown": False},
                 ],
                 "output_authority": "external_data",
                 "summary": {
                     "sources": [source_workflow_input(3)],
                     "taints": [user_input()],
                     "instruction_authority": "external_data",
                     "influence_unknown": False}})

    propagation("infoflow/propagation/authority-never-elevated",
                "the propagation helper refuses an output authority above "
                "every input's: elevation is unrepresentable",
                {"operation": "propagate_pure_expression",
                 "inputs": [
                     {"sources": [source_unknown()],
                      "taints": [web()], "authority": "external_data",
                      "influence_unknown": True},
                 ],
                 "output_authority": "external_data",
                 "summary": {
                     "sources": [source_unknown()],
                     "taints": [web()],
                     "instruction_authority": "external_data",
                     "influence_unknown": True}})

    # --- Compatibility ---------------------------------------------------

    compatibility("infoflow/compatibility/same-flow-contract",
                  "identical contracts are trivially non-relaxing",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       restrictions=[pii_email()],
                       constraints=[constraint("data_egress",
                                               output_sink(1))]),
                   "old_contract": contract(
                       restrictions=[pii_email()],
                       constraints=[constraint("data_egress",
                                               output_sink(1))]),
                   "expected": {"outcome": "proven"}})

    compatibility("infoflow/compatibility/new-removes-permitted-sink",
                  "removing a permitted sink never relaxes",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       restrictions=[pii_email()],
                       constraints=[constraint("data_egress",
                                               output_sink(1))]),
                   "old_contract": contract(
                       restrictions=[pii_email()],
                       constraints=[constraint("data_egress",
                                               output_sink(1)),
                                    constraint("data_egress",
                                               output_sink(2))]),
                   "expected": {"outcome": "proven"}})

    compatibility_relaxed("infoflow/compatibility/new-adds-permitted-sink",
                          "a newly permitted sink is relaxation",
                          {"operation": "flow_not_relaxed",
                           "new_contract": contract(
                               restrictions=[pii_email()],
                               constraints=[constraint("data_egress",
                                                       output_sink(1)),
                                            constraint("data_egress",
                                                       output_sink(2))]),
                           "old_contract": contract(
                               restrictions=[pii_email()],
                               constraints=[constraint("data_egress",
                                                       output_sink(1))])})

    compatibility_relaxed("infoflow/compatibility/new-drops-restriction",
                          "dropping a confidentiality restriction is "
                          "relaxation even with identical ceilings",
                          {"operation": "flow_not_relaxed",
                           "new_contract": contract(
                               restrictions=[pii_email()],
                               constraints=[constraint("data_egress",
                                                       output_sink(1))]),
                           "old_contract": contract(
                               restrictions=[pii_email(), internal()],
                               constraints=[constraint("data_egress",
                                                       output_sink(1))])})

    compatibility("infoflow/compatibility/new-removes-taint-unproven",
                  "narrowing the summary's source capacity without proof "
                  "is Unproven: no incomplete influence list proves an "
                  "influence away",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       contract_summary=summary(
                           sources=[source_workflow_input(1)],
                           taints=[web()], unknown=False)),
                   "old_contract": contract(
                       contract_summary=summary(
                           sources=[source_workflow_input(1),
                                    source_workflow_input(2)],
                           taints=[web(), user_input()], unknown=False)),
                   "expected": {"outcome": "unproven",
                                "reason": "influence_relation_unknown"}})

    compatibility("infoflow/compatibility/same-labels-different-catalog",
                  "the same label name under a different catalog is a "
                  "DIFFERENT identity: unknown in this universe, the "
                  "comparison fails closed at canonicalization",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       restrictions=[conf("pii.email",
                                          catalog="agentflow.other.labels@1")]),
                   "old_contract": contract(restrictions=[pii_email()]),
                   "expected": {"outcome":
                                    "unknown_label_in_new_catalog"}})

    compatibility("infoflow/compatibility/unknown-relationship-unproven",
                  "repairing an old influence_unknown into a known set is "
                  "Unproven, never proven compatible",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       contract_summary=summary(
                           sources=[source_unknown()], taints=[web()],
                           unknown=False)),
                   "old_contract": contract(
                       contract_summary=summary(
                           sources=[source_unknown()], taints=[web()],
                           unknown=True)),
                   "expected": {"outcome": "unproven",
                                "reason": "influence_relation_unknown"}})

    compatibility("infoflow/compatibility/authority-not-elevated-ok",
                  "same or lower authority in the new summary is fine",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       contract_summary=summary(
                           sources=[source_workflow_input(1)],
                           authority="external_data", unknown=True)),
                   "old_contract": contract(
                       contract_summary=summary(
                           sources=[source_workflow_input(1)],
                           authority="user", unknown=True)),
                   "expected": {"outcome": "proven"}})

    compatibility_relaxed("infoflow/compatibility/authority-elevated-relaxed",
                          "a higher authority claim in the new contract is "
                          "relaxation: authority is never elevated by a "
                          "contract rewrite",
                          {"operation": "flow_not_relaxed",
                           "new_contract": contract(
                               contract_summary=summary(
                                   sources=[source_workflow_input(1)],
                                   authority="workflow", unknown=True)),
                           "old_contract": contract(
                               contract_summary=summary(
                                   sources=[source_workflow_input(1)],
                                   authority="user", unknown=True))})

    compatibility("infoflow/compatibility/kinds-never-compared-across",
                  "the per-kind check keeps kinds separate: removing a "
                  "data_egress ceiling does not touch the "
                  "decision_influence judgment (and vice versa)",
                  {"operation": "flow_not_relaxed",
                   "new_contract": contract(
                       constraints=[constraint("decision_influence",
                                               output_sink(1))]),
                   "old_contract": contract(
                       constraints=[constraint("data_egress",
                                               output_sink(1)),
                                    constraint("decision_influence",
                                               output_sink(1))]),
                   "expected": {"outcome": "proven"}})

    compatibility_relaxed("infoflow/compatibility/new-kind-same-sink-relaxed",
                          "a decision_influence ceiling on a sink the old "
                          "contract permitted only for data_egress is a "
                          "newly permitted flow: kinds are separate "
                          "permissions, never implied by each other",
                          {"operation": "flow_not_relaxed",
                           "new_contract": contract(
                               constraints=[constraint("decision_influence",
                                                       output_sink(1))]),
                           "old_contract": contract(
                               constraints=[constraint("data_egress",
                                                       output_sink(1))])})

    # --- Equivalence -----------------------------------------------------

    equivalence("infoflow/equivalence/label-ordering",
                "label declaration order is not identity",
                [contract(restrictions=[pii_email(), internal()]),
                 contract(restrictions=[internal(), pii_email()])],
                canonical_ace1(validate(
                    contract(restrictions=[pii_email(), internal()]))))

    equivalence("infoflow/equivalence/taint-ordering",
                "taint declaration order is not identity",
                [contract(contract_summary=summary(
                    sources=[source_workflow_input(1)],
                    taints=[web(), user_input()], unknown=True)),
                 contract(contract_summary=summary(
                     sources=[source_workflow_input(1)],
                     taints=[user_input(), web()], unknown=True))],
                canonical_ace1(validate(
                    contract(contract_summary=summary(
                        sources=[source_workflow_input(1)],
                        taints=[web(), user_input()], unknown=True)))))

    equivalence("infoflow/equivalence/constraint-ordering",
                "sink-rule declaration order is not identity",
                [contract(constraints=[
                    constraint("decision_influence",
                               effect_sink(EFFECT_CATALOG,
                                           "production.deploy")),
                    constraint("data_egress", output_sink(1))]),
                 contract(constraints=[
                     constraint("data_egress", output_sink(1)),
                     constraint("decision_influence",
                                effect_sink(EFFECT_CATALOG,
                                            "production.deploy"))])],
                canonical_ace1(validate(
                    contract(constraints=[
                        constraint("data_egress", output_sink(1)),
                        constraint("decision_influence",
                                   effect_sink(EFFECT_CATALOG,
                                               "production.deploy"))]))))

    counts = {}
    for case in CASES:
        kind_ = case.split("/")[1]
        counts[kind_] = counts.get(kind_, 0) + 1
    print(f"frozen {len(CASES)} cases: {counts}")


if __name__ == "__main__":
    main()
