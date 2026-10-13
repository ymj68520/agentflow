#!/usr/bin/env python3
"""One-time 4C.1B.1 materialization audit, BEFORE registry allocation.

The accepted design is the input to this authoring tool. Its output is explicit
AFIDL; production codecs and the independent oracle consume registered AFIDL,
never this DSL parser. Does not modify any existing schema or registry.
"""
import hashlib
import importlib.util
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FAMILY = "agentflow.portable.proof.inputs@1"
SET_RULES = []
FILES = [
    "portable-proof-schema", "root-export-assertion-projection",
    "proof-context-manifest", "resolver-dependency-manifest",
    "compiler-semantics-identity",
]
IMPORTS = {
    "core": "agentflow.semantic.core@1", "ir": "agentflow.semantic.ir@1",
    "effect": "agentflow.effect.signature@1",
    "resource": "agentflow.resource.requirement@1",
    "flow": "agentflow.information.flow@1",
    "usage": "agentflow.usage.signature@1",
    "contract": "agentflow.component.contract@1",
    "execution": "agentflow.execution.requirements@1",
}
EXTERNAL = {}
for alias, names in {
    "core": ["DigestRefValue", "TypeSchemaRef", "PrimitiveConstant",
             "SemanticPrimitiveType", "ValueSlotId", "ExpressionId", "ResourceSlotId"],
    "ir": ["ComputationId", "RunParameterId"],
    "effect": ["EffectAtom", "ResourceSelector", "ScopeBinding", "EffectGuard",
               "EffectQualifier", "OccurrenceBounds", "OccurrenceCeiling", "EffectOperationRef"],
    "resource": ["ResourceTypeRef"],
    "flow": ["FlowSourceRef", "FlowConstraint", "FlowKind", "FlowSink",
             "ConfidentialityLabelRef", "TaintRef", "InstructionAuthority"],
    "usage": ["UsageMeterRef", "UsageSignature"],
    "contract": ["EffectFlowPartition", "RecoveryActionIdentity",
                 "RecoveryActionKind", "RecoveryUsageBinding"],
    "execution": ["ExecutionRequirements", "ExecutionCapabilityRef", "CapabilityAttribute",
                  "TrustDomainRef", "RegionRef", "RuntimeProfileRef"],
}.items():
    EXTERNAL.update({name: f"{alias}.{name}" for name in names})
EXTERNAL.update({
    "DigestRef": "core.DigestRefValue", "SchemaRef": "core.TypeSchemaRef",
    "IrPin": "core.DigestRefValue", "ContractPin": "core.DigestRefValue",
    "SchemaPin": "core.DigestRefValue", "WorkflowPrimitive": "core.SemanticPrimitiveType",
})
ALIASES = {
    "Text": "string", "Bytes": "bytes", "U32": "uint32", "U64": "uint64",
    "Ordinal64": "U64", "ContextId": "U32", "ResolverId": "U32",
    "CompilerManifestId": "U32", "AssertionId": "U32", "StructuralTypeId": "U32",
    "SpecId": "Text", "TypeName": "Text", "LogicalPath": "Text", "LowerHex64": "Text",
}
PRIMITIVES = {"unit", "null", "bool", "int32", "uint32", "int64", "uint64",
              "decimal", "float64", "string", "bytes", "timestamp", "duration"}


class Parser:
    def __init__(self, text):
        self.tokens = re.findall(r"[A-Za-z_][A-Za-z_0-9]*|[{}:;<>,()]", text)
        self.pos = 0

    def pop(self, expected=None):
        value = self.tokens[self.pos]
        self.pos += 1
        if expected is not None and value != expected:
            raise ValueError((expected, value))
        return value

    def peek(self):
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def shape(self):
        kind = self.pop()
        if kind in ("record", "variant", "enum"):
            self.pop("{")
            members = []
            while self.peek() != "}":
                name = self.pop()
                if kind == "enum":
                    item = None
                else:
                    self.pop(":")
                    item = self.shape()
                members.append((name, item))
                if self.peek() == ";":
                    self.pop()
            self.pop("}")
            return (kind, members)
        if kind in ("list", "seq", "set", "option", "CompleteCollection"):
            self.pop("<")
            inner = self.shape()
            if kind == "set":
                self.pop(",")
                key_tokens = []
                depth = 0
                while self.peek() != ">" or depth:
                    token = self.pop()
                    key_tokens.append(token)
                    depth += (token == "<") - (token == ">")
                key_expression = "".join(key_tokens)
                if key_expression.startswith("(") and key_expression.endswith(")"):
                    key_expression = key_expression[1:-1]
                node = ("set", inner, key_expression)
            else:
                node = (kind, inner)
            self.pop(">")
            return node
        return ("ref", kind)


def inputs():
    definitions = {}
    owners = {}
    for suffix in FILES:
        path = f"docs/implementation/phase-04c1a-{suffix}.md"
        text = (ROOT / path).read_text(encoding="utf-8")
        for block in re.findall(r"~~~text\n(.*?)~~~", text, re.S):
            block = re.sub(r"//[^\n]*", "", block)
            for match in re.finditer(r"^([A-Z][A-Za-z0-9]*) = (record|variant|enum)\b", block, re.M):
                name = match[1]
                if name in definitions or name.startswith("PpiAux"):
                    raise ValueError(f"duplicate/reserved name: {name}")
                definitions[name] = Parser(block[match.start(2):]).shape()
                owners[name] = path
    return definitions, owners


def materialize():
    definitions, owners = inputs()
    emitted = {}
    auxiliary_paths = {}

    def expr(node, owner, path, field_name=None):
        kind, body = node[:2]
        if kind == "ref":
            if body not in definitions and body not in ALIASES and body not in EXTERNAL and body not in PRIMITIVES:
                raise ValueError(f"undefined type {body}")
            return EXTERNAL.get(body, body)
        if kind == "CompleteCollection":
            inner_name = body[1] if body[0] == "ref" else body[0]
            complete_keys = {
                "ModuleContent": "specification_id",
                "SchemaRef": "SchemaRefKey",
                "EffectCatalogContent": "specification_id",
                "ResourceTypeContent": "SchemaRefKey",
                "FlowEffectSinkContent": "operation",
                "ConfidentialityLabelRef": "(catalog,name)",
                "TaintRef": "(catalog,name)",
                "UsageMeterContent": "meter",
                "ReconciliationOperationRef": "(catalog,name)",
                "ExecutionCapabilityContent": "capability",
                "TrustDomainRef": "(catalog,name)",
                "RegionRef": "(catalog,name)",
                "RegionCompatibility": "(from,to)",
                "ExecutionRuntimeProfileContent": "profile",
            }
            if inner_name not in complete_keys:
                raise ValueError(f"no registered complete-member key for {inner_name}")
            SET_RULES.append({
                "owner": f"{FAMILY}/{owner}",
                "field": field_name,
                "identity_key": complete_keys[inner_name],
                "element_type": inner_name,
                "field": field_name,
                "collection": "CompleteCollection.complete",
            })
            name = f"PpiAux{owner}P{path}G1"
            shape = ("variant", [("complete", ("list", body)),
                                  ("unavailable", ("ref", "UnavailableReason"))])
            add(name, shape, owner, path + "G1")
            return name
        if kind in ("list", "seq", "set", "option"):
            wrapper = "option" if kind == "option" else "list"
            if kind == "set":
                SET_RULES.append({
                    "owner": f"{FAMILY}/{owner}",
                    "field": field_name,
                    "identity_key": node[2],
                    "element_type": body[1] if body[0] == "ref" else body[0],
                })
            return f"{wrapper}<{expr(body, owner, path + ('O1' if wrapper == 'option' else 'L1'), field_name)}>"
        name = f"PpiAux{owner}P{path}"
        add(name, node, owner, path)
        return name

    def add(name, node, owner, path):
        if name in emitted:
            raise ValueError(f"name collision {name}")
        kind, members = node
        emitted[name] = None  # reserve before traversing children
        if name.startswith("PpiAux"):
            identity = (owner, path)
            if identity in auxiliary_paths.values():
                raise ValueError(f"path collision {identity}")
            auxiliary_paths[name] = identity
        fields = []
        for number, (member, child) in enumerate(members, 1):
            child_path = path + ("F" if kind == "record" else "C") + str(number)
            ty = None if kind == "enum" else expr(child, owner, child_path, member)
            fields.append({"number": number, "name": member, "type": ty})
        emitted[name] = {"kind": kind, "members": fields}

    for name, node in definitions.items():
        add(name, node, name, "")
    lines = [
        "// Portable Proof Inputs v1. Explicit additive ABI; see spec/portable/portable-proof-inputs.md.",
        "afidl 1;", f"schema {FAMILY};", "",
    ]
    for alias, family in sorted(IMPORTS.items()):
        lines.append(f"import {family} as {alias};")
    lines.append("")
    for name, target in ALIASES.items():
        lines.append(f"alias {name} = {target};")
    for name, item in sorted(emitted.items()):
        lines.extend(["", f"{item['kind']} {name} {{"])
        for field in item["members"]:
            tail = "" if field["type"] is None else f": {field['type']}"
            lines.append(f"    {field['number']} {field['name']}{tail};")
        lines.append("}")
    source = "\n".join(lines) + "\n"

    spec = importlib.util.spec_from_file_location("materialization_afidl", ROOT / "tools/gen-afidl-corpus.py")
    afidl = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(afidl)
    existing = sorted((ROOT / "spec/idl").rglob("*.afidl"))
    sources = [(str(p.relative_to(ROOT)), p.read_text(encoding="utf-8"))
               for p in existing if p.name != "portable-proof-inputs.afidl"]
    # The Python AFIDL module-set validator resolves all imported dependencies
    # and the entire named type graph, not just this module's local names.
    sources.append(("spec/idl/portable-proof-inputs.afidl", source))
    afidl.compile_module_set(sources)

    old_abi = json.loads((ROOT / "spec/registry/abi-numbers.json").read_text(encoding="utf-8"))
    baseline = ROOT / "spec/conformance/portable-proof-codec/baseline.json"
    if not baseline.exists():
        if len(old_abi["assignments"]) != 477:
            raise ValueError("baseline snapshot must precede registry allocation")
        frozen = {}
        for parent in ["spec/idl", "spec/conformance", "spec/tables"]:
            for path in sorted((ROOT / parent).rglob("*")):
                if path.is_file() and "portable-proof-codec" not in str(path):
                    frozen[str(path.relative_to(ROOT)).replace("\\", "/")] = hashlib.sha256(path.read_bytes()).hexdigest()
        baseline.parent.mkdir(parents=True, exist_ok=True)
        snapshot = {
            "start_head": "efc2a8d90a65432916a8dbf559510e2936f0d146",
            "abi": old_abi,
            "specifications": json.loads((ROOT / "spec/registry/specifications.json").read_text(encoding="utf-8")),
            "rules": json.loads((ROOT / "spec/registry/rules.json").read_text(encoding="utf-8")),
            "frozen_files": frozen,
        }
        baseline.write_text(json.dumps(snapshot, ensure_ascii=False, sort_keys=True, indent=2) + "\n",
                            encoding="utf-8", newline="\n")
    (ROOT / "spec/idl/portable-proof-inputs.afidl").write_text(source, encoding="utf-8", newline="\n")
    report = {
        "family": FAMILY, "aliases": ALIASES, "imports": IMPORTS,
        "imported_symbol_bindings": EXTERNAL, "named_types": emitted,
        "auxiliary_paths": auxiliary_paths, "design_owners": owners,
        "reserved_prefix": "PpiAux", "collisions": [],
        "numbered_assignments": sum(len(t["members"]) for t in emitted.values()),
        "set_rules": SET_RULES,
        "named_type_count": len(emitted) + len(ALIASES),
        "auxiliary_type_count": len(auxiliary_paths),
        "deferred_outside_bundle": ["PortableProofSubmission and origin/build sidecars",
                                    "VerifierSupportSnapshot local policy",
                                    "UpcasterDefinition future interface"],
    }
    (baseline.parent / "materialization.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8", newline="\n")
    print(json.dumps({key: report[key] for key in [
        "named_type_count", "auxiliary_type_count", "numbered_assignments", "collisions"]}))


if __name__ == "__main__":
    materialize()
