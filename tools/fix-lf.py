#!/usr/bin/env python3
"""One-off: make the corpus generators emit LF explicitly and normalize any
CRLF files in the working tree back to LF (the repository is LF-forced via
.gitattributes). Safe to run multiple times."""

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BS = chr(92)  # backslash
NL = chr(10)  # newline

OLD = (
    'path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "'
    + BS + 'n", encoding="utf-8")'
)
NEW = (
    'path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "'
    + BS + 'n", encoding="utf-8", newline="' + BS + 'n")'
)

for name in ("tools/gen-conformance.py", "tools/gen-afidl-corpus.py"):
    path = ROOT / name
    text = path.read_text(encoding="utf-8")
    if OLD in text:
        path.write_text(text.replace(OLD, NEW), encoding="utf-8", newline=NL)
        print("patched", name)
    elif NEW in text:
        print("already patched", name)
    else:
        raise SystemExit(f"writer line not found in {name}")

fixed = []
for path in ROOT.rglob("*"):
    if path.is_file() and ".git" not in path.parts and "target" not in path.parts:
        data = path.read_bytes()
        if b"\r\n" in data:
            path.write_bytes(data.replace(b"\r\n", b"\n"))
            fixed.append(str(path))
print("normalized CRLF files:", len(fixed))
for item in fixed:
    print("  ", item)
