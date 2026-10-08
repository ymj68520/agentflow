#!/usr/bin/env python3
"""Freezes the Phase 4A.2 failure-row conformance corpus.

INDEPENDENT implementation of the Failure Row rules
(spec/failure/failure-row.md): nominal failure type identity, exact schema
pinning, canonical ordering, duplicate rejection, the failure algebra
(containment, union, subtraction, handler propagation) and static control
propagation. The Rust implementation in crates/af-failure must agree with
the frozen expectations byte-for-byte; expected values are NEVER derived
from the Rust code.

Plumbing reuse (allowed): the independent AFIDL1 implementation supplies
the module compiler, canonical-form builder and TypeSchemaDigest
computation, and the semantic-ir corpus module supplies the ACE1 value
encoder. The failure ORACLE logic below is implemented here from scratch
and shares nothing with crates/af-failure.

Run from the repository root:  python tools/gen-failure-row-corpus.py
"""

import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "failure-row"
CORPUS_REVISION = 1

# Plumbing: the semantic-ir corpus module carries the ACE1 encoder and the
# independent AFIDL1 compiler (which it loads as `afidl`).
_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
FAILURE_AFDL = ROOT / "spec" / "idl" / "failure-row.afidl"

# Stable error kinds (spec/failure/failure-row.md section 13).
INVALID_FAILURE_TYPE = "invalid_failure_type"
UNKNOWN_FAILURE_SCHEMA = "unknown_failure_schema"
FAILURE_SCHEMA_DIGEST_MISMATCH = "failure_schema_digest_mismatch"
DUPLICATE_FAILURE_TYPE = "duplicate_failure_type"
FAILURE_NOT_DECLARED = "failure_not_declared"
FAILURE_ROW_UNPROVEN = "failure_row_unproven"

# ---------------------------------------------------------------------------
# Fixture schemas (explicit, immutable, frozen). The production failure
# universe is the future ComponentContract; these are TEST schemas.
# ---------------------------------------------------------------------------

FAILURES_SPEC = "agentflow.test.failures@1"
FAILURES_SOURCE = (
    "afidl 1;\n"
    "schema agentflow.test.failures@1;\n"
    "record PaymentDeclined { 1 reason: string; }\n"
    "enum QuotaExceeded { 1 monthly; 2 daily; }\n"
    "variant ValidationRejected { 1 schema_violation: string; "
    "2 unknown_field: string; }\n"
    "record InsufficientPermissions { 1 required_scope: string; }\n"
)
ALT_SPEC = "agentflow.test.failures-alt@1"
ALT_SOURCE = (
    "afidl 1;\n"
    "schema agentflow.test.failures-alt@1;\n"
    "record PaymentDeclined { 1 code: int64; }\n"
)


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
    FAILURES_SPEC: compile_fixture(FAILURES_SOURCE),
    ALT_SPEC: compile_fixture(ALT_SOURCE),
}


def fixture_schemas():
    return [
        {
            "specification_id": FAILURES_SPEC,
            "source": FAILURES_SOURCE,
            "types": [{"name": name, "digest_hex": digest}
                      for name, digest in sorted(FIXTURE_DIGESTS[FAILURES_SPEC].items())],
        },
        {
            "specification_id": ALT_SPEC,
            "source": ALT_SOURCE,
            "types": [{"name": name, "digest_hex": digest}
                      for name, digest in sorted(FIXTURE_DIGESTS[ALT_SPEC].items())],
        },
    ]


# ---------------------------------------------------------------------------
# Shapes (local walker over the failure-row module set; ACE1 plumbing from
# sir)
# ---------------------------------------------------------------------------

def load_failure_types():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("failure-row.afidl", FAILURE_AFDL.read_text(encoding="utf-8")),
    ]
    _modules, types_by_module = afidl.compile_module_set(sources)
    out = {}
    for spec, types in types_by_module.items():
        for schema in types:
            out[f"{spec}/{schema['name']}"] = schema
    return out


FAILURE_TYPES = load_failure_types()


def shape_of_identity(identity):
    kind, payload = FAILURE_TYPES[identity]["body"]
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


FAILURE_ROW_SHAPE = shape_of_identity("agentflow.failure.row@1/FailureRow")


def canonical_ace1(canonical_failures):
    """The canonical ACE1 text of a canonical (sorted, duplicate-free)
    failure list."""
    row = {"failures": canonical_failures}
    value = sir.json_to_value(FAILURE_ROW_SHAPE, row)
    return sir.encode_value(FAILURE_ROW_SHAPE, value)


# ---------------------------------------------------------------------------
# The oracle: identity, validation, canonicalization
# ---------------------------------------------------------------------------

class FailureOracleError(Exception):
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


def identity_of(ref):
    """The full nominal identity triple of a FailureTypeRef's payload."""
    return (ref["payload_type"]["specification_id"],
            ref["payload_type"]["type_name"],
            ref["payload_type"]["schema_digest"]["hex"])


def check_identity(ref):
    spec, name, digest = identity_of(ref)
    if not is_specification_id(spec):
        raise FailureOracleError(
            INVALID_FAILURE_TYPE,
            f"specification id {spec!r} is not a SpecificationId")
    if not name or any(c.isspace() for c in name):
        raise FailureOracleError(
            INVALID_FAILURE_TYPE,
            f"type name {name!r} is not a valid AFIDL type name")
    d = ref["payload_type"]["schema_digest"]
    if (d["algorithm"] != "sha256" or d["domain"] != "type-schema"
            or len(d["hex"]) != 64
            or not all(c in HEX_CHARS for c in d["hex"])):
        raise FailureOracleError(
            INVALID_FAILURE_TYPE,
            "pinned schema digest is not a TypeSchemaDigest")


def resolve_ref(ref, universe):
    """Exact resolution: the named schema must exist AND the pinned digest
    must match. No latest-schema fallback."""
    check_identity(ref)
    spec, name, digest = identity_of(ref)
    identity = f"{spec}/{name}"
    table = {f"{s}/{name_}": digest
             for s, digests in universe.items()
             for name_, digest in digests.items()}
    if identity not in table:
        raise FailureOracleError(
            UNKNOWN_FAILURE_SCHEMA,
            f"failure payload type {identity} does not resolve in the "
            "supplied schema universe (no latest-schema fallback)")
    if table[identity] != digest:
        raise FailureOracleError(
            FAILURE_SCHEMA_DIGEST_MISMATCH,
            f"failure payload type {identity} is pinned to {digest} but "
            f"the universe computes {table[identity]}")


def canonicalize(row, universe=None):
    """Structural canonicalization (syntax + duplicates + canonical sort);
    with a universe, every ref is exactly resolved first."""
    seen = set()
    failures = []
    for ref in row["failures"]:
        if universe is not None:
            resolve_ref(ref, universe)
        else:
            check_identity(ref)
        key = tuple(part.encode("utf-8") for part in identity_of(ref))
        if key in seen:
            raise FailureOracleError(
                DUPLICATE_FAILURE_TYPE,
                "failure type {}/{}/{}... is declared twice".format(*key.decode("utf-8").split("§") if False else identity_of(ref)))
        seen.add(key)
        failures.append(ref)
    failures.sort(key=lambda ref: tuple(
        part.encode("utf-8") for part in identity_of(ref)))
    return failures


def validate(row, universe):
    return canonicalize(row, universe)


def is_empty(canonical_failures):
    return not canonical_failures


# ---------------------------------------------------------------------------
# The algebra (pure set semantics over nominal identities)
# ---------------------------------------------------------------------------

def contains(a, b):
    """Φa ⊆ Φb: exact nominal membership only."""
    keys_b = {tuple(part.encode("utf-8") for part in identity_of(ref))
              for ref in b["failures"]}
    return all(tuple(part.encode("utf-8") for part in identity_of(ref))
               in keys_b for ref in a["failures"])


def union(a, b):
    """Φa ∪ Φb: deduplicated, canonically ordered."""
    merged = {}
    for ref in a["failures"] + b["failures"]:
        key = tuple(part.encode("utf-8") for part in identity_of(ref))
        merged.setdefault(key, ref)
    return {"failures": [merged[key] for key in sorted(merged)]}


def join(a, b):
    """Branch composition (SELECT / PARALLEL / RACE) is exactly set union."""
    return union(a, b)


def remove_handled(row, handled):
    """Φ \\ H: exact subtraction; H ⊄ Φ is a static rejection."""
    keys_row = {tuple(part.encode("utf-8") for part in identity_of(ref))
                for ref in row["failures"]}
    for ref in handled["failures"]:
        key = tuple(part.encode("utf-8") for part in identity_of(ref))
        if key not in keys_row:
            spec, name, digest = identity_of(ref)
            raise FailureOracleError(
                FAILURE_NOT_DECLARED,
                f"handled failure type {spec}/{name}/{digest[:8]}... is "
                "not declared by the child failure row")
    remaining = {}
    for ref in row["failures"]:
        key = tuple(part.encode("utf-8") for part in identity_of(ref))
        if key not in {tuple(part.encode("utf-8") for part in identity_of(h))
                       for h in handled["failures"]}:
            remaining.setdefault(key, ref)
    return {"failures": [remaining[key] for key in sorted(remaining)]}


def propagate_handler_failures(child, handled, handlers):
    """(Φ_child \\ H) ∪ Φ_handler_1 ∪ ... : handler failures propagate,
    unhandled child failures survive, H ⊆ child is enforced."""
    result = remove_handled(child, handled)
    for handler in handlers:
        result = union(result, handler)
    return result


# ---------------------------------------------------------------------------
# Control propagation
# ---------------------------------------------------------------------------

def union_all(rows):
    result = {"failures": []}
    for row in rows:
        result = union(result, row)
    return result


def sequence_row(rows):
    return union_all(rows)


def select_row(rows):
    return union_all(rows)


def parallel_row(rows):
    return union_all(rows)


def race_row(rows):
    return union_all(rows)


def iterate_row(body):
    """Types do not multiply with repetitions."""
    return {"failures": list(body["failures"])}


def retry_row(child):
    """Retry attempts are not a proof that failure is impossible."""
    return {"failures": list(child["failures"])}


def timeout_row(child):
    """TimedOut is a Control Outcome, never a failure type."""
    return {"failures": list(child["failures"])}


def scope_row(child):
    return {"failures": list(child["failures"])}


def compensation_scope_row(child):
    """Forward failures are never erased."""
    return {"failures": list(child["failures"])}


def await_row():
    """Waiting invents no business failure type."""
    return {"failures": []}


# ---------------------------------------------------------------------------
# Fixture builders (serde-form dicts)
# ---------------------------------------------------------------------------

def ref(spec, name, digest=None):
    # Unknown fixture names get a well-formed placeholder pin: existence is
    # checked before the digest, so the unknown-schema case fails with
    # unknown_failure_schema either way.
    pinned = digest or FIXTURE_DIGESTS.get(spec, {}).get(name) or "0" * 64
    return {
        "payload_type": {
            "specification_id": spec,
            "type_name": name,
            "schema_digest": {
                "algorithm": "sha256",
                "domain": "type-schema",
                "hex": pinned,
            },
        }
    }


def row(*refs):
    return {"failures": list(refs)}


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


def golden(case_id, note, signature):
    canonical = validate(signature, FIXTURE_DIGESTS)
    write_case("golden", {
        "kind": "agentflow.conformance.failure-row.case",
        "case_kind": "golden",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_schemas": fixture_schemas(),
        "signature": signature,
        "canonical_ace1_utf8": canonical_ace1(canonical),
        "is_empty": is_empty(canonical),
        "failure_count": len(canonical),
    })


def negative(case_id, note, payload):
    """A validator or algebra rejection: the oracle must raise exactly this
    kind."""
    expected = payload["expected_kind"]
    if payload.get("operation") == "require_proven_row":
        pass  # declarative: Unproven.require_row always fails closed
    elif payload.get("operation"):
        operands = payload["operands"]
        try:
            apply_operation(payload["operation"], operands,
                            payload.get("handlers", []))
        except FailureOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    else:
        try:
            validate(payload["signature"], FIXTURE_DIGESTS)
        except FailureOracleError as error:
            if error.kind != expected:
                raise AssertionError(
                    f"{case_id}: oracle raised {error.kind}, expected "
                    f"{expected}") from error
        else:
            raise AssertionError(f"{case_id}: expected rejection {expected}")
    write_case("negative", {
        "kind": "agentflow.conformance.failure-row.case",
        "case_kind": "negative",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_schemas": fixture_schemas(),
        **payload,
    })


def algebra(case_id, note, operation, operands, expected, handlers=None,
            property_kind=None):
    for operand in operands:
        validate(operand, FIXTURE_DIGESTS)
    write_case("algebra", {
        "kind": "agentflow.conformance.failure-row.case",
        "case_kind": "algebra",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_schemas": fixture_schemas(),
        "operation": operation,
        "operands": operands,
        "handlers": handlers or [],
        "expected": expected,
        "property": property_kind,
    })


def equivalence(case_id, note, members, frozen_bytes):
    for member in members:
        canonical = validate(member, FIXTURE_DIGESTS)
        if canonical_ace1(canonical) != frozen_bytes:
            raise AssertionError(
                f"{case_id}: equivalence member drifted off the frozen "
                "canonical bytes")
    write_case("equivalence", {
        "kind": "agentflow.conformance.failure-row.case",
        "case_kind": "equivalence",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_schemas": fixture_schemas(),
        "members": members,
        "canonical_ace1_utf8": frozen_bytes,
    })


def result_bytes(value):
    """canonical ACE1 of an operation result (a row dict or a failure
    list — both appear across the corpus builders)."""
    failures = value["failures"] if isinstance(value, dict) else value
    return canonical_ace1(failures)


def apply_operation(operation, operands, handlers):
    """Applies one algebra/control operation to VALIDATED operands."""
    if operation == "union":
        return union(operands[0], operands[1])
    if operation == "contains":
        return contains(operands[0], operands[1])
    if operation == "remove_handled":
        return remove_handled(operands[0], operands[1])
    if operation == "handle_propagate":
        return propagate_handler_failures(operands[0], operands[1], handlers)
    if operation in ("sequence_union", "select_union", "parallel_union",
                     "race_union"):
        return union_all(operands)
    if operation == "iterate_row":
        return iterate_row(operands[0])
    if operation == "retry_row":
        return retry_row(operands[0])
    if operation == "timeout_row":
        return timeout_row(operands[0])
    if operation == "scope_row":
        return scope_row(operands[0])
    if operation == "compensation_scope_row":
        return compensation_scope_row(operands[0])
    if operation == "await_row":
        return await_row()
    raise AssertionError(f"unknown operation {operation!r}")


def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()

    payment = lambda: ref(FAILURES_SPEC, "PaymentDeclined")
    quota = lambda: ref(FAILURES_SPEC, "QuotaExceeded")
    validation = lambda: ref(FAILURES_SPEC, "ValidationRejected")
    permissions = lambda: ref(FAILURES_SPEC, "InsufficientPermissions")
    alt_payment = lambda: ref(ALT_SPEC, "PaymentDeclined")

    # --- Golden ----------------------------------------------------------

    golden("failure/golden/empty-failure-row",
           "the empty row is the algebra identity; it does NOT mean the "
           "computation succeeds",
           row())

    golden("failure/golden/single-failure-type",
           "one nominal failure type, exactly pinned",
           row(payment()))

    golden("failure/golden/multiple-failure-types",
           "several distinct nominal failure types",
           row(payment(), quota(), validation()))

    golden("failure/golden/same-short-name-different-family",
           "the same short name under two specifications is two DIFFERENT "
           "failure types; both may coexist in one row",
           row(payment(), alt_payment()))

    golden("failure/golden/enum-backed-typed-failure",
           "an enum-backed failure payload",
           row(quota()))

    golden("failure/golden/record-backed-typed-failure",
           "a record-backed failure payload",
           row(permissions()))

    golden("failure/golden/variant-backed-typed-failure",
           "a variant-backed failure payload",
           row(validation()))

    golden("failure/golden/canonical-ordering",
           "non-canonical input order: canonical bytes follow the nominal "
           "identity byte order, never the declaration order",
           row(validation(), payment(), quota()))

    # --- Negative --------------------------------------------------------

    negative("failure/negative/duplicate-failure-type",
             "the same nominal type declared twice is rejected: a declared "
             "row is a set, not a multiset",
             {"signature": row(payment(), ref(FAILURES_SPEC,
                                              "PaymentDeclined")),
              "expected_kind": DUPLICATE_FAILURE_TYPE})

    negative("failure/negative/unknown-schema",
             "the named schema does not exist in the universe; there is no "
             "latest-schema fallback",
             {"signature": row(ref("agentflow.test.failures@1",
                                   "PaymentRefunded")),
              "expected_kind": UNKNOWN_FAILURE_SCHEMA})

    negative("failure/negative/wrong-schema-digest",
             "same logical name, different pinned digest: never silently "
             "compatible",
             {"signature": row(ref(FAILURES_SPEC, "PaymentDeclined",
                                   digest="ab" * 32)),
              "expected_kind": FAILURE_SCHEMA_DIGEST_MISMATCH})

    negative("failure/negative/invalid-nominal-reference",
             "a malformed specification id fails the nominal grammar",
             {"signature": row(ref("agentflow.test.failures",
                                   "PaymentDeclined")),
              "expected_kind": INVALID_FAILURE_TYPE})

    negative("failure/negative/same-name-schema-mismatch",
             "same short name, different specification: handled-set "
             "subtraction must not pretend the alt-family type matches",
             {"operation": "remove_handled",
              "operands": [row(payment()), row(alt_payment())],
              "expected_kind": FAILURE_NOT_DECLARED})

    negative("failure/negative/unproven-failure-containment",
             "Unproven is never the empty row: demanding a row from an "
             "unproven envelope fails closed",
             {"operation": "require_proven_row",
              "envelope": "unproven",
              "expected_kind": FAILURE_ROW_UNPROVEN})

    negative("failure/negative/handle-matcher-not-in-child-row",
             "a HANDLE matcher must match a typed failure the child "
             "actually declares",
             {"operation": "remove_handled",
              "operands": [row(payment()),
                           row(payment(), quota())],
              "expected_kind": FAILURE_NOT_DECLARED})

    negative("failure/negative/ambiguous-duplicate-handled-type",
             "an ambiguous handled set (the same nominal type twice) is "
             "rejected before any subtraction",
             {"signature": row(payment(), payment()),
              "expected_kind": DUPLICATE_FAILURE_TYPE})

    # --- Algebra ---------------------------------------------------------

    x = lambda: row(payment())
    y = lambda: row(quota())
    z = lambda: row(validation())

    # union identity
    assert result_bytes(union(x(), row())) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    assert result_bytes(union(row(), x())) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    algebra("failure/algebra/union-identity",
            "the empty row is the two-sided identity of union",
            "union", [row(), x()],
            {"canonical_ace1_utf8": result_bytes(validate(x(), FIXTURE_DIGESTS))},
            property_kind="identity")

    # union commutativity
    u_comm = union(x(), y())
    assert result_bytes(u_comm) == result_bytes(union(y(), x()))
    algebra("failure/algebra/union-commutativity",
            "union is commutative",
            "union", [x(), y()],
            {"canonical_ace1_utf8": result_bytes(u_comm)},
            property_kind="commutativity")

    # union associativity
    u_left = union(union(x(), y()), z())
    u_right = union(x(), union(y(), z()))
    assert result_bytes(u_left) == result_bytes(u_right)
    algebra("failure/algebra/union-associativity",
            "union is associative",
            "union", [x(), y(), z()],
            {"canonical_ace1_utf8": result_bytes(u_left)},
            property_kind="associativity")

    # union idempotence
    u_idem = union(x(), x())
    assert result_bytes(u_idem) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    algebra("failure/algebra/union-idempotence",
            "union with itself reproduces the row (the algebra dedupes; "
            "declarations reject)",
            "union", [x(), x()],
            {"canonical_ace1_utf8": result_bytes(u_idem)},
            property_kind="idempotence")

    # containment reflexivity and transitivity
    for probe in (row(), x(), row(payment(), quota())):
        assert contains(probe, probe), "reflexivity broke"
    algebra("failure/algebra/containment-reflexivity",
            "every row contains itself, including the empty row",
            "contains", [row(payment(), quota())],
            {"contains": True},
            property_kind="reflexivity")

    a_narrow, b_mid, c_wide = x(), row(payment(), quota()), \
        row(payment(), quota(), validation())
    assert contains(a_narrow, b_mid) and contains(b_mid, c_wide) \
        and contains(a_narrow, c_wide)
    algebra("failure/algebra/containment-transitivity",
            "exact containment is transitive where provable: "
            "a ⊆ b ∧ b ⊆ c ⟹ a ⊆ c",
            "contains", [a_narrow, b_mid, c_wide],
            {"chain_pairs": [[0, 1], [1, 2], [0, 2]]},
            property_kind="transitivity")

    # HANDLE subtraction
    removed = remove_handled(row(payment(), quota()), x())
    assert result_bytes(removed) == result_bytes(validate(y(), FIXTURE_DIGESTS))
    algebra("failure/algebra/handle-removes-exact-match",
            "the handled set removes exactly the matched nominal type",
            "remove_handled", [row(payment(), quota()), x()],
            {"canonical_ace1_utf8": result_bytes(removed)})

    # unhandled child failures survive
    residual = propagate_handler_failures(
        row(payment(), quota()), x(), [])
    assert result_bytes(residual) == result_bytes(validate(y(), FIXTURE_DIGESTS))
    algebra("failure/algebra/unhandled-child-failure-preserved",
            "unhandled child failures propagate verbatim",
            "handle_propagate", [row(payment(), quota()), x()],
            {"canonical_ace1_utf8": result_bytes(residual)})

    # handler failures propagate (two handlers)
    propagated = propagate_handler_failures(
        row(payment()), x(), [row(quota()), row(validation())])
    assert result_bytes(propagated) == result_bytes(
        validate(row(quota(), validation()), FIXTURE_DIGESTS))
    algebra("failure/algebra/handler-new-failure-propagated",
            "handler failures union into the result and are never treated "
            "as automatically handled",
            "handle_propagate", [row(payment()), x()],
            {"canonical_ace1_utf8": result_bytes(propagated)},
            handlers=[row(quota()), row(validation())])

    # branch unions
    for name, op in (("select", "select_union"), ("parallel", "parallel_union"),
                     ("race", "race_union")):
        merged = apply_operation(op, [x(), y()], [])
        assert result_bytes(merged) == result_bytes(
            validate(row(payment(), quota()), FIXTURE_DIGESTS))
        algebra(f"failure/algebra/{name}-branch-union",
                f"{name.upper()} branch composition is exactly set union",
                op, [x(), y()],
                {"canonical_ace1_utf8": result_bytes(merged)})

    # RETRY preserves
    retried = retry_row(x())
    assert result_bytes(retried) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    algebra("failure/algebra/retry-preserves-possible-failure",
            "retry attempts are not a proof that business failure is "
            "impossible: the row is preserved",
            "retry_row", [x()],
            {"canonical_ace1_utf8": result_bytes(retried)})

    # TIMEOUT does not introduce a failure type
    timed = timeout_row(x())
    assert result_bytes(timed) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    assert not contains(row(ref(FAILURES_SPEC, "TimedOut")), timed)
    algebra("failure/algebra/timeout-does-not-introduce-failure",
            "TimedOut is a Control Outcome, never a failure type: TIMEOUT "
            "adds no type to the child row",
            "timeout_row", [x()],
            {"canonical_ace1_utf8": result_bytes(timed)})

    # COMPENSATION_SCOPE does not erase
    comp = compensation_scope_row(x())
    assert result_bytes(comp) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    algebra("failure/algebra/compensation-does-not-erase-forward-failure",
            "compensation never erases forward failure facts",
            "compensation_scope_row", [x()],
            {"canonical_ace1_utf8": result_bytes(comp)})

    # ITERATE preserves types (no count duplication)
    it = iterate_row(x())
    assert result_bytes(it) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    algebra("failure/algebra/iterate-preserves-body-row",
            "ITERATE repeats occurrences, never failure TYPES: the body "
            "row is preserved verbatim",
            "iterate_row", [x()],
            {"canonical_ace1_utf8": result_bytes(it)})

    # AWAIT invents nothing
    aw = await_row()
    assert result_bytes(aw) == result_bytes(validate(row(), FIXTURE_DIGESTS))
    algebra("failure/algebra/await-invents-nothing",
            "waiting for a signal invents no business failure type",
            "await_row", [],
            {"canonical_ace1_utf8": result_bytes(aw)})

    # SCOPE passthrough
    sc = scope_row(x())
    assert result_bytes(sc) == result_bytes(validate(x(), FIXTURE_DIGESTS))
    algebra("failure/algebra/scope-passthrough",
            "SCOPE passes the child row through unchanged",
            "scope_row", [x()],
            {"canonical_ace1_utf8": result_bytes(sc)})

    # --- Equivalence -------------------------------------------------------

    members = [
        row(validation(), payment()),
        row(payment(), validation()),
    ]
    equivalence("failure/equivalence/declaration-ordering",
                "declaration order never changes the canonical bytes",
                members,
                result_bytes(validate(members[0], FIXTURE_DIGESTS)))

    equivalence("failure/equivalence/union-equals-declared",
                "a union result has exactly the canonical bytes of the "
                "equivalent declared row",
                [union(row(payment()), row(quota())),
                 row(quota(), payment())],
                result_bytes(validate(row(payment(), quota()),
                                      FIXTURE_DIGESTS)))

    equivalence("failure/equivalence/join-equals-union",
                "branch composition (join) has exactly the bytes of union",
                [join(row(payment()), row(quota())),
                 union(row(payment()), row(quota()))],
                result_bytes(validate(row(payment(), quota()),
                                      FIXTURE_DIGESTS)))

    print(f"\nfailure-row corpus: {len(CASES)} cases frozen "
          f"(revision {CORPUS_REVISION})")


if __name__ == "__main__":
    main()
