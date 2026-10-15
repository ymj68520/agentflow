"""Explicit Edition1 regeneration inputs, never Edition2 support or Expected.

All generators continue to execute/write in the original repository. Only
historical identity/type INPUTS come from the exact abc91d5 snapshot; old
Expected remains immutable and new-edition inputs use a separate namespace.
"""
from pathlib import Path
import hashlib
import os

ROOT = Path(__file__).resolve().parent.parent
BASELINE = "abc91d533b4d62e91226722e1d1fcaa30bd00883"
PINS = {
    "specifications": ("legacy-specifications.json", "b3dd17ee9e4d2f8b1fb86108abfa5fb667a551fda49927db1bfb52bdaf12882c"),
    "rules": ("legacy-rules.json", "599b94622e446bb7ffb2ac0f39c3c252d891273d123eae32bbb35dbbb28ef329"),
    "root_export": ("legacy-root-export.rs.txt", "c1b806e60072390d288f9283b83e6d09813ff4d88a5366220ba78efdfbfb92a9"),
}

def pinned_bytes(name):
    filename, expected = PINS[name]
    raw = (ROOT / "spec/conformance/g01-edition2-staging" / filename).read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError("historical input changed: " + filename)
    return raw

def historical_type_source(path):
    relative = path.relative_to(ROOT).as_posix()
    if relative == "crates/af-computation/src/root_export.rs":
        return pinned_bytes("root_export").decode("utf-8")
    return path.read_text(encoding="utf-8")

def assert_historical_output(path, value, encoding="utf-8", newline=None):
    """Calculate then verify frozen Edition1 output;never repair or overwrite it.

    This is an original-directory historical regeneration check,not execution
    in a mirror and not a claimed repair of the previous Windows I/O fault.
    """
    path=Path(path)
    if isinstance(value,str):
        if newline is None:value=value.replace("\n",os.linesep)
        elif newline not in ("", "\n"):value=value.replace("\n",newline)
        raw=value.encode(encoding)
    else:raw=bytes(value)
    if not path.is_file() or path.read_bytes()!=raw:
        raise ValueError("frozen Edition1 output mismatch;refusing overwrite: "+str(path))
    return len(raw)
