#!/usr/bin/env python3
"""Freezes the Phase 4A.3 resource-requirement conformance corpus.

INDEPENDENT implementation of the Resource Requirement Row rules
(spec/resource/resource-requirement.md): nominal resource type identity,
exact schema pinning, the explicit resource-type universe, NFC attribute
normalization, canonical ordering, duplicate rejection, static candidate
satisfaction and the requirement non-strengthening proof. The Rust
implementation in crates/af-resource must agree with the frozen
expectations byte-for-byte; expected values are NEVER derived from the
Rust code.

Plumbing reuse (allowed): the independent AFIDL1 implementation supplies
the module compiler, canonical-form builder and TypeSchemaDigest
computation, and the semantic-ir corpus module supplies the ACE1 value
encoder. The resource ORACLE logic below is implemented here from scratch
and shares nothing with crates/af-resource.

Run from the repository root:  python tools/gen-resource-requirement-corpus.py
"""

import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "resource-requirement"
CORPUS_REVISION = 1

# Plumbing: the semantic-ir corpus module carries the ACE1 encoder and the
# independent AFIDL1 compiler (which it loads as `afidl`).
_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
RESOURCE_AFDL = ROOT / "spec" / "idl" / "resource-requirement.afidl"

# Stable error kinds (spec/resource/resource-requirement.md section 14).
INVALID_RESOURCE_TYPE = "invalid_resource_type"
UNKNOWN_RESOURCE_TYPE = "unknown_resource_type"
RESOURCE_TYPE_DIGEST_MISMATCH = "resource_type_digest_mismatch"
INVALID_RESOURCE_PORT = "invalid_resource_port"
DUPLICATE_RESOURCE_PORT = "duplicate_resource_port"
INVALID_RESOURCE_ATTRIBUTE = "invalid_resource_attribute"
DUPLICATE_RESOURCE_ATTRIBUTE = "duplicate_resource_attribute"
UNKNOWN_RESOURCE_ATTRIBUTE = "unknown_resource_attribute"
RESOURCE_REQUIREMENT_UNSATISFIED = "resource_requirement_unsatisfied"
RESOURCE_REQUIREMENT_UNPROVEN = "resource_requirement_unproven"
RESOURCE_REQUIREMENT_STRENGTHENED = "resource_requirement_strengthened"

# ---------------------------------------------------------------------------
# Fixture schemas (explicit, immutable, frozen). The production resource
# type registry is a later stage; these are TEST schemas with explicit
# descriptor vocabularies, never a masqueraded production registry.
# ---------------------------------------------------------------------------

RESOURCES_SPEC = "agentflow.test.resources@1"
RESOURCES_SOURCE = (
    "afidl 1;\n"
    "schema agentflow.test.resources@1;\n"
    "record Repository { 1 provider_hint: string; }\n"
    "record StorageBucket { 1 region_hint: string; }\n"
    "record QueueEndpoint { 1 protocol_hint: string; }\n"
)
ALT_SPEC = "agentflow.test.resources-alt@1"
ALT_SOURCE = (
    "afidl 1;\n"
    "schema agentflow.test.resources-alt@1;\n"
    "record Repository { 1 code: int64; }\n"
)

# The explicit recognition descriptors: which fixture schemas are
# resource types, pinned to which digest, with which closed attribute
# vocabulary. Frozen into every corpus case; the Rust runner rebuilds its
# universe from these very descriptors.
RESOURCE_DESCRIPTORS = {
    (RESOURCES_SPEC, "Repository"): {"provider", "visibility", "default_branch"},
    (RESOURCES_SPEC, "StorageBucket"): {"region", "storage_class", "durability"},
    (RESOURCES_SPEC, "QueueEndpoint"): {"protocol", "region"},
    (ALT_SPEC, "Repository"): {"tier"},
}


def compile_fixture(source):
    """Compiles one fixture module and returns {type_name: digest_hex}."""
    module = afidl.parse_afidl(source)
    family = f"{module['spec']['namespace']}@{module['spec']['major']}"
    out = {}
    for declaration in module["declarations"]:
        schema = afidl.canonical_type(family, declaration,
                                      afidl.import_table(module))
        _payload, digest = afidl.type_schema_digest(schema)
        out[schema["name"]] = digest
    return out


FIXTURE_DIGESTS = {
    RESOURCES_SPEC: compile_fixture(RESOURCES_SOURCE),
    ALT_SPEC: compile_fixture(ALT_SOURCE),
}


def fixture_schemas():
    return [
        {
            "specification_id": RESOURCES_SPEC,
            "source": RESOURCES_SOURCE,
            "types": [{"name": name, "digest_hex": digest}
                      for name, digest in sorted(FIXTURE_DIGESTS[RESOURCES_SPEC].items())],
        },
        {
            "specification_id": ALT_SPEC,
            "source": ALT_SOURCE,
            "types": [{"name": name, "digest_hex": digest}
                      for name, digest in sorted(FIXTURE_DIGESTS[ALT_SPEC].items())],
        },
    ]


def descriptor_table():
    """The frozen recognition descriptors, in deterministic order."""
    return [
        {
            "specification_id": spec,
            "type_name": name,
            "digest_hex": FIXTURE_DIGESTS[spec][name],
            "recognized_attributes": sorted(RESOURCE_DESCRIPTORS[(spec, name)]),
        }
        for spec, name in sorted(RESOURCE_DESCRIPTORS)
    ]


def descriptors_by_identity():
    """{(spec, name): descriptor} in the frozen table shape."""
    return {
        (entry["specification_id"], entry["type_name"]): entry
        for entry in descriptor_table()
    }


DESCRIPTORS = descriptors_by_identity()


# ---------------------------------------------------------------------------
# Shapes (local walker over the resource module set; ACE1 plumbing from
# sir)
# ---------------------------------------------------------------------------

def load_resource_types():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("resource-requirement.afidl", RESOURCE_AFDL.read_text(encoding="utf-8")),
    ]
    _modules, types_by_module = afidl.compile_module_set(sources)
    out = {}
    for spec, types in types_by_module.items():
        for schema in types:
            out[f"{spec}/{schema['name']}"] = schema
    return out


RESOURCE_TYPES = load_resource_types()


def shape_of_identity(identity):
    kind, payload = RESOURCE_TYPES[identity]["body"]
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


ROW_SHAPE = shape_of_identity("agentflow.resource.requirement@1/"
                              "ResourceRequirementRow")


def canonical_ace1(row):
    """The canonical ACE1 text of a canonical requirement row."""
    return sir.encode_value(ROW_SHAPE, sir.json_to_value(ROW_SHAPE, row))


# ---------------------------------------------------------------------------
# The oracle: identity, universe resolution, canonicalization
# ---------------------------------------------------------------------------

class ResourceOracleError(Exception):
    def __init__(self, kind, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind
        self.detail = detail


NAMESPACE_CHARS = set("abcdefghijklmnopqrstuvwxyz0123456789-")
HEX_CHARS = set("0123456789abcdef")


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


def type_identity(rtref):
    """The full nominal identity triple of a ResourceTypeRef."""
    return (rtref["schema"]["specification_id"],
            rtref["schema"]["type_name"],
            rtref["schema"]["schema_digest"]["hex"])


def check_identity(rtref):
    spec, name, digest = type_identity(rtref)
    if not is_specification_id(spec):
        raise ResourceOracleError(
            INVALID_RESOURCE_TYPE,
            f"specification id {spec!r} is not a SpecificationId")
    if not name or any(c.isspace() for c in name):
        raise ResourceOracleError(
            INVALID_RESOURCE_TYPE,
            f"type name {name!r} is not a valid AFIDL type name")
    d = rtref["schema"]["schema_digest"]
    if (d["algorithm"] != "sha256" or d["domain"] != "type-schema"
            or len(d["hex"]) != 64
            or not all(c in HEX_CHARS for c in d["hex"])):
        raise ResourceOracleError(
            INVALID_RESOURCE_TYPE,
            "pinned schema digest is not a TypeSchemaDigest")


def canonical_attributes(attributes):
    """NFC-normalized, non-empty, duplicate-free, key-ordered."""
    seen = set()
    out = []
    for attribute in attributes:
        key = nfc(attribute["key"])
        value = nfc(attribute["value"])
        if not key or not value:
            raise ResourceOracleError(
                INVALID_RESOURCE_ATTRIBUTE,
                "attribute key and value must be non-empty NFC text")
        if key in seen:
            raise ResourceOracleError(
                DUPLICATE_RESOURCE_ATTRIBUTE,
                f"attribute key {key!r} is declared twice (NFC-equal "
                "spellings are the same key)")
        seen.add(key)
        out.append({"key": key, "value": value})
    out.sort(key=lambda a: (a["key"].encode("utf-8"), a["value"].encode("utf-8")))
    return out


def canonicalize(row):
    """Structural canonicalization: positive unique ports in ascending
    order, canonical attribute lists, syntax-checked identity."""
    seen_ports = set()
    requirements = []
    for requirement in row["requirements"]:
        port = requirement["port_no"]
        if port <= 0:
            raise ResourceOracleError(
                INVALID_RESOURCE_PORT, "requirement port 0 is invalid")
        if port in seen_ports:
            raise ResourceOracleError(
                DUPLICATE_RESOURCE_PORT,
                f"requirement port {port} is declared twice")
        seen_ports.add(port)
        check_identity(requirement["resource_type"])
        requirements.append({
            "port_no": port,
            "resource_type": requirement["resource_type"],
            "required_attributes":
                canonical_attributes(requirement["required_attributes"]),
        })
    requirements.sort(key=lambda r: r["port_no"])
    return {"requirements": requirements}


def resolve_resource_type(rtref, descriptors):
    """Exact resolution: schema exists, digest matches, type explicitly
    recognized, descriptor coherent. No latest-schema fallback."""
    check_identity(rtref)
    spec, name, digest = type_identity(rtref)
    identity = f"{spec}/{name}"
    table = {f"{s}/{name_}": digests[name_]
             for s, digests in FIXTURE_DIGESTS.items() for name_ in digests}
    if identity not in table:
        raise ResourceOracleError(
            UNKNOWN_RESOURCE_TYPE,
            f"resource type {identity} does not resolve in the supplied "
            "schema universe (no latest-schema fallback)")
    if table[identity] != digest:
        raise ResourceOracleError(
            RESOURCE_TYPE_DIGEST_MISMATCH,
            f"resource type {identity} is pinned to {digest} but the "
            f"universe computes {table[identity]}")
    if (spec, name) not in descriptors:
        raise ResourceOracleError(
            UNKNOWN_RESOURCE_TYPE,
            f"schema {identity} is registered but not explicitly "
            "recognized as a resource type")
    if descriptors[(spec, name)]["digest_hex"] != digest:
        raise ResourceOracleError(
            RESOURCE_TYPE_DIGEST_MISMATCH,
            f"the recognition descriptor of {identity} pins a different "
            "digest than the universe computes")
    return descriptors[(spec, name)]


def validate(row, descriptors):
    canonical = canonicalize(row)
    for requirement in canonical["requirements"]:
        descriptor = resolve_resource_type(requirement["resource_type"],
                                           descriptors)
        vocabulary = set(descriptor["recognized_attributes"])
        for attribute in requirement["required_attributes"]:
            if attribute["key"] not in vocabulary:
                raise ResourceOracleError(
                    UNKNOWN_RESOURCE_ATTRIBUTE,
                    f"attribute key {attribute['key']!r} is not in the "
                    "descriptor vocabulary: unknown constraint meaning")
    return canonical


# ---------------------------------------------------------------------------
# Static satisfaction
# ---------------------------------------------------------------------------

def satisfies(candidate, requirement):
    check_identity(candidate["resource_type"])
    check_identity(requirement["resource_type"])
    candidate_attributes = canonical_attributes(candidate["attributes"])
    required = canonical_attributes(requirement["required_attributes"])
    if (type_identity(candidate["resource_type"])
            != type_identity(requirement["resource_type"])):
        raise ResourceOracleError(
            RESOURCE_REQUIREMENT_UNSATISFIED,
            "candidate resource type does not exact-match the required "
            "type (same short name or similar shape is never accepted)")
    pairs = {(a["key"], a["value"]) for a in candidate_attributes}
    for attribute in required:
        if (attribute["key"], attribute["value"]) not in pairs:
            raise ResourceOracleError(
                RESOURCE_REQUIREMENT_UNSATISFIED,
                f"candidate does not carry required attribute "
                f"{attribute['key']}={attribute['value']!r}")


# ---------------------------------------------------------------------------
# Requirement non-strengthening
# ---------------------------------------------------------------------------

def requirement_not_stronger(new_row, new_namespace, old_row, old_namespace,
                             descriptors):
    """Proves the new row accepts at least every candidate the old row
    accepted, within ONE port namespace. Returns ("proven", None) or
    ("unproven", reason); proven violations raise."""
    new_canonical = canonicalize(new_row)
    old_canonical = canonicalize(old_row)
    if new_namespace != old_namespace:
        raise ResourceOracleError(
            RESOURCE_REQUIREMENT_UNPROVEN,
            "resource port namespaces are different contract interfaces: "
            "equal port numbers do not name the same resource, and no "
            "cross-component proof exists without an explicit port mapping")
    for canonical_row in (old_canonical, new_canonical):
        for requirement in canonical_row["requirements"]:
            resolve_resource_type(requirement["resource_type"], descriptors)
    old_by_port = {r["port_no"]: r for r in old_canonical["requirements"]}
    for requirement in new_canonical["requirements"]:
        old_requirement = old_by_port.get(requirement["port_no"])
        if old_requirement is None:
            raise ResourceOracleError(
                RESOURCE_REQUIREMENT_STRENGTHENED,
                f"port {requirement['port_no']} requires a resource the "
                "previous contract did not require")
        if (type_identity(requirement["resource_type"])
                != type_identity(old_requirement["resource_type"])):
            return ("unproven", "retained_port_resource_type_changed")
        vocabulary = set(descriptors[(
            requirement["resource_type"]["schema"]["specification_id"],
            requirement["resource_type"]["schema"]["type_name"])]
            ["recognized_attributes"])
        if any(a["key"] not in vocabulary
               for a in requirement["required_attributes"]):
            return ("unproven", "unknown_attribute_semantics")
        old_pairs = {(a["key"], a["value"])
                     for a in old_requirement["required_attributes"]}
        for attribute in requirement["required_attributes"]:
            if ((attribute["key"], attribute["value"]) not in old_pairs):
                raise ResourceOracleError(
                    RESOURCE_REQUIREMENT_STRENGTHENED,
                    f"port {requirement['port_no']}: required attribute "
                    f"{attribute['key']}={attribute['value']!r} is new or "
                    "changed")
    return ("proven", None)


# ---------------------------------------------------------------------------
# Fixture builders (serde-form dicts)
# ---------------------------------------------------------------------------

def rtref(spec, name, digest=None):
    # Unknown fixture names get a well-formed placeholder pin: existence is
    # checked before the digest, so the unknown-type case fails with
    # unknown_resource_type either way.
    pinned = digest or FIXTURE_DIGESTS.get(spec, {}).get(name) or "0" * 64
    return {
        "schema": {
            "specification_id": spec,
            "type_name": name,
            "schema_digest": {
                "algorithm": "sha256",
                "domain": "type-schema",
                "hex": pinned,
            },
        }
    }


def attr(key, value):
    return {"key": key, "value": value}


def req(port, resource_type, *attributes):
    return {
        "port_no": port,
        "resource_type": resource_type,
        "required_attributes": list(attributes),
    }


def row(*requirements):
    return {"requirements": list(requirements)}


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
        "kind": "agentflow.conformance.resource-requirement.case",
        "case_kind": case_kind,
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_schemas": fixture_schemas(),
        "resource_types": descriptor_table(),
    }


def golden(case_id, note, requirement_row):
    canonical = validate(requirement_row, DESCRIPTORS)
    payload = base_case("golden", case_id, note)
    payload.update({
        "row": requirement_row,
        "canonical_ace1_utf8": canonical_ace1(canonical),
        "is_empty": not canonical["requirements"],
        "requirement_count": len(canonical["requirements"]),
    })
    write_case("golden", payload)


def negative(case_id, note, payload):
    """A validator or satisfaction rejection: the oracle must raise
    exactly this kind."""
    expected = payload["expected_kind"]
    if payload.get("operation") == "satisfies":
        try:
            satisfies(payload["candidate"], payload["requirement"])
        except ResourceOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    else:
        try:
            validate(payload["row"], DESCRIPTORS)
        except ResourceOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    payload = {**base_case("negative", case_id, note), **payload}
    write_case("negative", payload)


def compatibility(case_id, note, payload):
    """A compatibility or satisfaction judgment; the frozen expectation is
    checked against the oracle before freezing."""
    expected = payload["expected"]
    if payload.get("operation") == "satisfies":
        outcome = "satisfied"
        try:
            satisfies(payload["candidate"], payload["requirement"])
        except ResourceOracleError as error:
            if error.kind != RESOURCE_REQUIREMENT_UNSATISFIED:
                raise
            outcome = "unsatisfied"
        if outcome != expected["outcome"]:
            raise AssertionError(
                f"{case_id}: oracle outcome {outcome}, expected "
                f"{expected['outcome']}")
    else:
        verdict = None
        try:
            verdict = requirement_not_stronger(
                payload["new_row"], payload["new_namespace"],
                payload["old_row"], payload["old_namespace"],
                DESCRIPTORS)
        except ResourceOracleError as error:
            # The only expected raise is the cross-namespace fail-closed
            # comparison; proven strengthening uses the dedicated writer.
            if not (expected["outcome"] == "cross_namespace"
                    and error.kind == RESOURCE_REQUIREMENT_UNPROVEN):
                raise AssertionError(
                    f"{case_id}: unexpected oracle rejection "
                    f"{error.kind}") from error
        if expected["outcome"] != "cross_namespace":
            assert verdict is not None
            if verdict != (expected["outcome"], expected.get("reason")):
                raise AssertionError(
                    f"{case_id}: oracle verdict {verdict}, "
                    f"expected {expected}")
    payload = {**base_case("compatibility", case_id, note), **payload}
    write_case("compatibility", payload)


def compatibility_strengthened(case_id, note, payload):
    """A compatibility case whose correct outcome is the proven
    `resource_requirement_strengthened` rejection."""
    try:
        verdict = requirement_not_stronger(
            payload["new_row"], payload["new_namespace"],
            payload["old_row"], payload["old_namespace"],
            DESCRIPTORS)
    except ResourceOracleError as error:
        if error.kind != RESOURCE_REQUIREMENT_STRENGTHENED:
            raise AssertionError(
                f"{case_id}: oracle raised {error.kind}, expected "
                f"{RESOURCE_REQUIREMENT_STRENGTHENED}") from error
    else:
        raise AssertionError(
            f"{case_id}: expected strengthening rejection, oracle "
            f"returned {verdict}")
    frozen = {**base_case("compatibility", case_id, note), **payload}
    frozen["expected"] = {"outcome": "strengthened"}
    write_case("compatibility", frozen)


def equivalence(case_id, note, members, frozen_bytes):
    descriptors = DESCRIPTORS
    for member in members:
        canonical = validate(member, descriptors)
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

    repository = lambda: rtref(RESOURCES_SPEC, "Repository")
    bucket = lambda: rtref(RESOURCES_SPEC, "StorageBucket")
    queue = lambda: rtref(RESOURCES_SPEC, "QueueEndpoint")
    alt_repository = lambda: rtref(ALT_SPEC, "Repository")

    # --- Golden ----------------------------------------------------------

    golden("resource/golden/empty-requirement-row",
           "the empty row declares no external resource requirement; it "
           "does NOT mean the computation is pure, succeeds or is "
           "authorized",
           row())

    golden("resource/golden/single-typed-requirement",
           "one resource requirement, exactly pinned",
           row(req(1, repository())))

    golden("resource/golden/multiple-resource-ports",
           "several resource requirements at distinct ports",
           row(req(1, repository()), req(2, bucket()), req(3, queue())))

    golden("resource/golden/exact-attributes",
           "one exact-value attribute condition",
           row(req(1, repository(), attr("provider", "github"))))

    golden("resource/golden/multiple-attributes",
           "several exact-value conditions, declared out of canonical "
           "order",
           row(req(1, bucket(), attr("storage_class", "standard"),
                   attr("durability", "regional"),
                   attr("region", "eu-west-1"))))

    golden("resource/golden/same-type-distinct-ports",
           "the same resource type at two ports is two independent "
           "requirements: ports are interface-local coordinates, not "
           "resource identities",
           row(req(1, repository(), attr("provider", "github")),
               req(2, repository(), attr("provider", "gitlab"))))

    golden("resource/golden/same-short-name-different-family",
           "the same short name under two specifications is two DIFFERENT "
           "resource types; both may coexist in one row at distinct ports",
           row(req(1, repository()), req(2, alt_repository())))

    golden("resource/golden/canonical-ordering",
           "non-canonical input order: canonical bytes follow ascending "
           "port number and attribute key order, never the declaration "
           "order",
           row(req(2, queue(), attr("region", "eu-west-1"),
                   attr("protocol", "amqp")),
               req(1, repository(), attr("visibility", "private"),
                   attr("provider", "github"))))

    # --- Negative --------------------------------------------------------

    negative("resource/negative/invalid-port-zero",
             "port 0 is not a valid requirement port: ports are positive",
             {"row": row(req(0, repository())),
              "expected_kind": INVALID_RESOURCE_PORT})

    negative("resource/negative/duplicate-port",
             "the same port declared by two requirements is rejected: one "
             "interface has at most one requirement per port",
             {"row": row(req(1, repository()), req(1, bucket())),
              "expected_kind": DUPLICATE_RESOURCE_PORT})

    negative("resource/negative/unknown-resource-type",
             "the named schema does not exist in the universe; there is "
             "no latest-schema fallback",
             {"row": row(req(1, rtref(RESOURCES_SPEC, "ObjectStore"))),
              "expected_kind": UNKNOWN_RESOURCE_TYPE})

    negative("resource/negative/wrong-resource-type-digest",
             "the named schema exists but the pin differs: same logical "
             "name, different identity — never silently compatible",
             {"row": row(req(1, rtref(RESOURCES_SPEC, "Repository",
                                      digest="1" * 64))),
              "expected_kind": RESOURCE_TYPE_DIGEST_MISMATCH})

    negative("resource/negative/schema-not-recognized-as-resource-type",
             "a legal AFIDL schema alone is not a resource type: the "
             "fixture schemas outside the descriptor table are never "
             "promoted",
             {"row": row(req(1, rtref(RESOURCES_SPEC, "Repository"),
                             attr("provider", "github")),
                         req(2, rtref("agentflow.test.resources@1",
                                      "provider_hint"))),
              "expected_kind": UNKNOWN_RESOURCE_TYPE})

    negative("resource/negative/duplicate-attribute-key",
             "the same attribute key declared twice with different values "
             "is rejected: never silently overwritten",
             {"row": row(req(1, repository(), attr("provider", "github"),
                             attr("provider", "gitlab"))),
              "expected_kind": DUPLICATE_RESOURCE_ATTRIBUTE})

    negative("resource/negative/nfc-duplicate-attribute-key",
             "NFC-equivalent key spellings are the same key: the pair is "
             "rejected as a duplicate, not accepted as an equivalence "
             "input",
             {"row": row(req(1, bucket(),
                             attr("r\u00e9gion", "eu-west-1"),
                             attr("re\u0301gion", "us-east-1"))),
              "expected_kind": DUPLICATE_RESOURCE_ATTRIBUTE})

    negative("resource/negative/unknown-attribute",
             "an attribute key outside the type's descriptor vocabulary "
             "has no known constraint meaning",
             {"row": row(req(1, repository(), attr("provider", "github"),
                             attr("acceleration", "edge"))),
              "expected_kind": UNKNOWN_RESOURCE_ATTRIBUTE})

    negative("resource/negative/candidate-missing-required-attribute",
             "the candidate lacks a required attribute entirely",
             {"operation": "satisfies",
              "candidate": {"resource_type": repository(),
                            "attributes": [attr("provider", "github")]},
              "requirement": req(1, repository(),
                                 attr("provider", "github"),
                                 attr("visibility", "private")),
              "expected_kind": RESOURCE_REQUIREMENT_UNSATISFIED})

    negative("resource/negative/candidate-attribute-mismatch",
             "the candidate carries the key but with a different value",
             {"operation": "satisfies",
              "candidate": {"resource_type": repository(),
                            "attributes": [attr("provider", "github"),
                                           attr("visibility", "public")]},
              "requirement": req(1, repository(),
                                 attr("visibility", "private")),
              "expected_kind": RESOURCE_REQUIREMENT_UNSATISFIED})

    negative("resource/negative/candidate-wrong-type",
             "the candidate's type does not exact-match: same record "
             "shape under a different pin is a different resource type",
             {"operation": "satisfies",
              "candidate": {"resource_type": alt_repository(),
                            "attributes": [attr("provider", "github")]},
              "requirement": req(1, repository(),
                                 attr("provider", "github")),
              "expected_kind": RESOURCE_REQUIREMENT_UNSATISFIED})

    # --- Compatibility ---------------------------------------------------

    compatibility("resource/compatibility/identical-rows",
                  "identical requirement rows: the new contract accepts "
                  "exactly the old candidate set",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "proven"}})

    compatibility("resource/compatibility/new-removes-port",
                  "a removed requirement port never strengthens in the R "
                  "dimension (full interface compatibility is a later "
                  "contract-layer decision)",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github")),
                                  req(2, bucket(),
                                      attr("region", "eu-west-1"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "proven"}})

    compatibility_strengthened("resource/compatibility/new-adds-port",
                               "a new required port is strengthening: the "
                               "new implementation needs a resource the "
                               "old contract did not",
                               {"operation": "requirement_not_stronger",
                                "new_row": row(req(1, repository(),
                                                   attr("provider", "github")),
                                               req(2, bucket())),
                                "new_namespace": "component-a",
                                "old_row": row(req(1, repository(),
                                                   attr("provider", "github"))),
                                "old_namespace": "component-a"})

    compatibility("resource/compatibility/new-removes-attribute",
                  "fewer exact-value conditions accept every candidate the "
                  "old row accepted",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github"),
                                      attr("visibility", "private"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "proven"}})

    compatibility_strengthened("resource/compatibility/new-adds-attribute",
                               "an added exact-value condition narrows the "
                               "acceptable candidate set",
                               {"operation": "requirement_not_stronger",
                                "new_row": row(req(1, repository(),
                                                   attr("provider", "github"),
                                                   attr("visibility", "private"))),
                                "new_namespace": "component-a",
                                "old_row": row(req(1, repository(),
                                                   attr("provider", "github"))),
                                "old_namespace": "component-a"})

    compatibility("resource/compatibility/changed-nominal-type-same-port",
                  "a retained port with a different nominal resource type "
                  "is not provable: no implicit subtyping exists",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, bucket(),
                                      attr("region", "eu-west-1"))),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "unproven",
                                "reason":
                                    "retained_port_resource_type_changed"}})

    compatibility("resource/compatibility/same-short-name-different-digest",
                  "the same short name under a different pin is a "
                  "different resource type: the retained-port check is "
                  "nominal, not lexical",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, alt_repository())),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository())),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "unproven",
                                "reason":
                                    "retained_port_resource_type_changed"}})

    compatibility("resource/compatibility/unknown-attribute-implication-unproven",
                  "a NEW constraint on an attribute key outside the type's "
                  "vocabulary cannot be assumed conjunctive exact-value: "
                  "the proof fails closed as Unproven",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, repository(),
                                      attr("provider", "github"),
                                      attr("acceleration", "edge"))),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "unproven",
                                "reason": "unknown_attribute_semantics"}})

    compatibility("resource/compatibility/old-unknown-attribute-does-not-block",
                  "unknown keys on the OLD side only shrink the old "
                  "candidate set: the proof survives when the new side "
                  "stays inside the vocabulary",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "new_namespace": "component-a",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github"),
                                      attr("legacy_route", "on"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "proven"}})

    compatibility("resource/compatibility/cross-component-namespaces-not-merged",
                  "two components' port 1 are different resources by "
                  "default: comparing rows across namespaces is not a "
                  "static proof and fails closed",
                  {"operation": "requirement_not_stronger",
                   "new_row": row(req(1, bucket(),
                                      attr("region", "eu-west-1"))),
                   "new_namespace": "component-b",
                   "old_row": row(req(1, repository(),
                                      attr("provider", "github"))),
                   "old_namespace": "component-a",
                   "expected": {"outcome": "cross_namespace"}})

    compatibility("resource/compatibility/satisfied-candidate",
                  "a candidate carrying every required exact attribute "
                  "satisfies the requirement",
                  {"operation": "satisfies",
                   "candidate": {"resource_type": repository(),
                                 "attributes": [attr("provider", "github"),
                                                attr("visibility", "private")]},
                   "requirement": req(1, repository(),
                                      attr("provider", "github"),
                                      attr("visibility", "private")),
                   "expected": {"outcome": "satisfied"}})

    compatibility("resource/compatibility/satisfied-candidate-extra-attributes",
                  "a requirement is a lower bound: extra candidate "
                  "attributes are fine",
                  {"operation": "satisfies",
                   "candidate": {"resource_type": bucket(),
                                 "attributes": [attr("region", "eu-west-1"),
                                                attr("storage_class", "standard"),
                                                attr("durability", "regional"),
                                                attr("tier", "hot")]},
                   "requirement": req(1, bucket(),
                                      attr("region", "eu-west-1")),
                   "expected": {"outcome": "satisfied"}})

    # --- Equivalence -----------------------------------------------------

    equivalence("resource/equivalence/port-ordering",
                "requirement declaration order is not identity: different "
                "port orders produce identical canonical bytes",
                [row(req(1, repository(), attr("provider", "github")),
                     req(2, bucket(), attr("region", "eu-west-1"))),
                 row(req(2, bucket(), attr("region", "eu-west-1")),
                     req(1, repository(), attr("provider", "github")))],
                canonical_ace1(validate(
                    row(req(1, repository(), attr("provider", "github")),
                        req(2, bucket(), attr("region", "eu-west-1"))),
                    DESCRIPTORS)))

    equivalence("resource/equivalence/attribute-ordering",
                "attribute declaration order is not identity",
                [row(req(1, bucket(), attr("storage_class", "standard"),
                         attr("durability", "regional"))),
                 row(req(1, bucket(), attr("durability", "regional"),
                         attr("storage_class", "standard")))],
                canonical_ace1(validate(
                    row(req(1, bucket(), attr("durability", "regional"),
                            attr("storage_class", "standard"))),
                    DESCRIPTORS)))

    equivalence("resource/equivalence/nfc-equivalent-text",
                "NFC-equivalent attribute text canonicalizes to identical "
                "bytes (the value carries a precomposed vs decomposed "
                "accent)",
                [row(req(1, repository(), attr("provider", "caf\u00e9"))),
                 row(req(1, repository(), attr("provider", "cafe\u0301")))],
                canonical_ace1(validate(
                    row(req(1, repository(), attr("provider", "caf\u00e9"))),
                    DESCRIPTORS)))

    counts = {}
    for case in CASES:
        kind = case.split("/")[1]
        counts[kind] = counts.get(kind, 0) + 1
    print(f"frozen {len(CASES)} cases: {counts}")


if __name__ == "__main__":
    main()
