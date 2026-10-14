"""Finite Rust declaration inventory for conformance, never a semantic oracle.

This reads type declarations, not Rust results or function bodies. The frozen
snapshot is reviewed separately; normal generation validates against it and
cannot silently accept a new production field. Recursive edges are references.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ROOTS = [
    "af-computation::DerivedWorkflowContractSummary",
    "af-computation::RootRecoverySummary",
    "af-computation::RootExportAssertionIdentity",
    "af-computation::RootExportProofReport",
    "af-computation::CheckedRootExportCompatibility",
    "af-portable::StaticContextProjection",
]
CONTAINERS = {"Vec", "Option", "Box", "BTreeMap", "BTreeSet", "HashMap", "HashSet", "Arc", "Rc", "Cow", "PhantomData"}
PRIMITIVES = {"String", "str", "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64", "T", "Self"}


def clean_comments(text):
    # Preserve strings and lifetimes; declaration delimiters never occur in a
    # type's quoted attribute, and attributes are removed below.
    text = re.sub(r"/\*.*?\*/", lambda m: " " * len(m[0]), text, flags=re.S)
    text = re.sub(r"//[^\n]*", lambda m: " " * len(m[0]), text)
    return re.sub(r"#\[[^\]]*\]", "", text, flags=re.S)


def split_top(text, delimiter=","):
    stack, start, parts = [], 0, []
    for index, char in enumerate(text):
        if char in "<([{":
            stack.append(char)
        elif char in ">)]}":
            if stack:
                stack.pop()
        elif char == delimiter and not stack:
            parts.append(text[start:index].strip())
            start = index + 1
    parts.append(text[start:].strip())
    return [part for part in parts if part]


def balanced(text, start):
    opener = text[start]
    closer = {"{": "}", "(": ")", "<": ">"}[opener]
    depth = 1
    for index in range(start + 1, len(text)):
        if text[index] == opener:
            depth += 1
        elif text[index] == closer:
            depth -= 1
            if depth == 0:
                return text[start + 1:index], index + 1
    raise ValueError("unclosed Rust declaration")


def field(part, position):
    part = re.sub(r"^pub(?:\([^)]*\))?\s+", "", part.strip())
    match = re.match(r"(\w+)\s*:\s*(.*)$", part, re.S)
    name, full_type = (match[1], match[2]) if match else (str(position), part)
    full_type = re.sub(r"\s+", " ", full_type).strip()
    return {"field": name, "full_type": full_type,
            "option_presence": "explicit_none_or_some" if re.search(r"\bOption\s*<", full_type) else "required",
            "collection": bool(re.search(r"\b(?:Vec|BTreeMap|BTreeSet|HashMap|HashSet)\s*<", full_type))}


def declarations():
    result = {}
    for path in sorted((ROOT / "crates").glob("*/src/**/*.rs")):
        crate = path.relative_to(ROOT / "crates").parts[0]
        text = clean_comments(path.read_text(encoding="utf-8"))
        for match in re.finditer(r"\b(struct|enum)\s+(\w+)", text):
            kind, name = match.groups()
            cursor = match.end()
            while cursor < len(text) and text[cursor].isspace():
                cursor += 1
            if cursor < len(text) and text[cursor] == "<":
                _, cursor = balanced(text, cursor)
            while cursor < len(text) and text[cursor].isspace():
                cursor += 1
            if cursor >= len(text) or text[cursor] not in "{(":
                continue
            body, _ = balanced(text, cursor)
            key = crate + "::" + name
            owner = {"owner_type": key, "kind": kind, "source": path.relative_to(ROOT).as_posix()}
            if kind == "struct":
                owner["fields"] = [field(part, i) for i, part in enumerate(split_top(body))]
            else:
                alternatives = {}
                for part in split_top(body):
                    tag = re.match(r"(\w+)", part)
                    if not tag:
                        raise ValueError(f"invalid variant {key}: {part}")
                    rest = part[tag.end():].strip()
                    if rest.startswith(("{", "(")):
                        contents, _ = balanced(rest, 0)
                        alternatives[tag[1]] = [field(item, i) for i, item in enumerate(split_top(contents))]
                    else:
                        alternatives[tag[1]] = []
                owner["variants"] = alternatives
            if key in result and result[key] != owner:
                # Name collisions in distinct private modules are not resolved by
                # runtime output; any reachable ambiguity is an explicit error.
                result[key] = {"ambiguous": True, "declarations": [result[key], owner]}
            else:
                result[key] = owner
    return result


def references(full_type, crate, index):
    refs, terminals = set(), set()
    for token in re.findall(r"(?:\b[a-z_]+::)*\b[A-Za-z_][A-Za-z_0-9]*", re.sub(r"'\w+", "", full_type)):
        name = token.split("::")[-1]
        if name in CONTAINERS or name in PRIMITIVES or name in {"mut", "dyn", "static"}:
            continue
        candidates = [key for key in index if key.endswith("::" + name)]
        qualified_crate = token.split("::")[0].replace("_", "-") if "::" in token else crate
        preferred = qualified_crate + "::" + name
        if preferred in index:
            candidates = [preferred]
        if len(candidates) == 1:
            refs.add(candidates[0])
        elif len(candidates) > 1:
            raise ValueError(f"ambiguous type reference {crate}: {full_type}: {candidates}")
        else:
            terminals.add(token)
    return sorted(refs), sorted(terminals)


def inventory():
    index = declarations()
    queue, graph, terminals = list(ROOTS), {}, set()
    while queue:
        name = queue.pop(0)
        if name in graph:
            continue
        owner = index[name]
        if owner.get("ambiguous"):
            raise ValueError(f"ambiguous reachable type {name}")
        graph[name] = owner
        slots = owner.get("fields", []) + [item for items in owner.get("variants", {}).values() for item in items]
        for slot in slots:
            refs, external = references(slot["full_type"], name.split("::")[0], index)
            slot["type_references"] = refs
            slot["nominal_scalar_leaves"] = external
            terminals.update(external)
            queue.extend(refs)
    scalar_types={}
    for source,macro in [("crates/af-ir/src/ids.rs","ir_id"),("crates/af-semantic/src/ids.rs","local_id")]:
        text=clean_comments((ROOT/source).read_text(encoding="utf-8"))
        if "pub struct $name(u32)" not in text:
            raise ValueError("nominal-id macro representation changed: "+source)
        for match in re.finditer(macro+r'!\(\s*(\w+)\s*,\s*"([^"]+)"\s*\)',text,re.S):
            if match[1] in terminals:
                scalar_types[match[1]]={"scalar_type":"u32","namespace":match[2],"source":source,"macro":macro,"presence":"nominal scalar; zero representable, checked IR requires positive id"}
    if set(scalar_types)!=terminals:
        raise ValueError("unresolved nominal scalar leaves: "+str(terminals-set(scalar_types)))
    return {"schema_version": 1, "roots": ROOTS, "types": {key: graph[key] for key in sorted(graph)},
            "nominal_scalar_leaves": sorted(terminals),"nominal_scalar_definitions":{key:scalar_types[key] for key in sorted(scalar_types)}}


def slots(graph):
    for name, owner in graph["types"].items():
        for item in owner.get("fields", []):
            yield name + "." + item["field"], owner, item
        for variant, fields in owner.get("variants", {}).items():
            yield name + "::" + variant + ".$tag", owner, {"field": "$tag", "full_type": "variant_tag"}
            for item in fields:
                yield name + "::" + variant + "." + item["field"], owner, item
