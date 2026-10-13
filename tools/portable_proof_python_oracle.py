"""Independent Python ACE1 and TypeSchema oracle for the portable codec corpus.

This module has no Rust subprocess, production-code import, or Actual-results
dependency. Its typed-value writer follows the registered ACE1 grammar.
"""

import hashlib
import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
AFIDL_SPEC = importlib.util.spec_from_file_location(
    "portable_afidl_oracle", ROOT / "tools" / "gen-afidl-corpus.py")
AFIDL = importlib.util.module_from_spec(AFIDL_SPEC)
AFIDL_SPEC.loader.exec_module(AFIDL)


def _escape(text):
    out = []
    for char in text:
        code = ord(char)
        if char == '"':
            out.append('\\"')
        elif char == "\\":
            out.append("\\\\")
        elif char == "\b":
            out.append("\\b")
        elif char == "\t":
            out.append("\\t")
        elif char == "\n":
            out.append("\\n")
        elif char == "\f":
            out.append("\\f")
        elif char == "\r":
            out.append("\\r")
        elif code < 0x20:
            out.append("\\u%04x" % code)
        else:
            out.append(char)
    return '"' + "".join(out) + '"'


def ace1(value):
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return _escape(str(value))
    if isinstance(value, float):
        raise TypeError("binary floating point is never an ACE1 numeric source")
    if isinstance(value, str):
        normalized = unicodedata.normalize("NFC", value)
        if normalized != value:
            raise ValueError("portable v1 identities must be NFC before encoding")
        return _escape(value)
    if isinstance(value, list):
        return "[" + ",".join(ace1(item) for item in value) + "]"
    if isinstance(value, dict):
        normalized = {}
        for key, item in value.items():
            canonical_key = unicodedata.normalize("NFC", key)
            if canonical_key != key or canonical_key in normalized:
                raise ValueError("map and record keys must be unique NFC")
            normalized[key] = item
        keys = sorted(normalized, key=lambda key: key.encode("utf-8"))
        return "{" + ",".join(_escape(key) + ":" + ace1(normalized[key]) for key in keys) + "}"
    raise TypeError(f"unsupported closed ACE1 fixture value {type(value).__name__}")


def canonical_bytes(value):
    return ace1(value).encode("utf-8")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def hash_v1(domain, payload):
    return hashlib.sha256(b"AF\x00" + domain.encode("ascii") + b"\x00" + payload).hexdigest()


def b64url(raw):
    import base64
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def ref(schema, name, digest):
    return {
        "specification_id": schema,
        "type_name": name,
        "schema_digest": {"algorithm": "sha256", "domain": "type-schema", "hex": digest},
    }


def compile_schema_assets(paths):
    sources = [(path, (ROOT / path).read_text(encoding="utf-8")) for path in paths]
    modules, types_by_module = AFIDL.compile_module_set(sources)
    schemas = []
    refs = {}
    for spec_id in sorted(types_by_module, key=lambda item: item.encode("utf-8")):
        for schema in types_by_module[spec_id]:
            payload, digest = AFIDL.type_schema_digest(schema)
            identity = (schema["schema"], schema["name"])
            schemas.append({
                "reference": ref(*identity, digest),
                "canonical_content": b64url(payload),
            })
            refs[identity] = ref(*identity, digest)
    return modules, types_by_module, schemas, refs
