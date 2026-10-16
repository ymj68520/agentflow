"""Independent declaration/trace Expected for the R3 Outcome-B diagnostic.

NOT full seed conformance. Never imports, invokes, or copies Rust Actual.
Old Expected, registry, and all normative sources are read-only.
"""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("native_guard_capture", ROOT / "tools/oracle/native_guard_capture.py")
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)


def main():
    folder = ROOT / "spec/conformance/g01-t3-native-capture"
    folder.mkdir(parents=True, exist_ok=True)
    rows = oracle.vectors()
    expected = [{"case_id": v["case_id"], "input": v,
                 "expected_trace": oracle.expected(v)} for v in rows]
    assert len({r["case_id"] for r in expected}) == 53
    manifest = {"namespace": "g01-t3-native-capture", "kind": "IndependentNativeTraceExpectedManifest",
                "source_head": "a0ef89e71ac31405ba05df619f585b5830aa8c92",
                "case_ids": [r["case_id"] for r in expected], "case_count": 53,
                "certified_t3_capture_cases": 0,
                "full_seed_conformance": False, "complete_scope": "PendingG02",
                "normalizer": "NotImplemented", "positive_capability": "NotAuthorized",
                "excluded": ["complete G02 Scope", "T4 canonical obligations", "T6 D10 report",
                             "non-native T3 producers", "Native seed full-field closure"],
                "expected_source": "independent declarations, AFIDL schema pins, frozen native rules"}
    for name, value in [("expected-traces.json", expected), ("manifest.json", manifest)]:
        (folder / name).write_text(json.dumps(value, ensure_ascii=True, sort_keys=True, indent=2) + "\n", encoding="utf-8", newline="\n")
        print("wrote " + str((folder / name).relative_to(ROOT)))


if __name__ == "__main__": main()
