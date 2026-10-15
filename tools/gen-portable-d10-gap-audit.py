#!/usr/bin/env python3
"""Additive D10 inventory and independent D9 preconditions; fail closed on gaps."""

import argparse
import json

from oracle.d10_gap_audit import artifacts


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    outputs = artifacts()
    # Finish all independent derivations/shape checks before writing anything.
    for path, value in outputs.items():
        encoded = json.dumps(value, ensure_ascii=False, indent=2) + "\n"
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != encoded:
                raise SystemExit("D10 gap audit drift: " + str(path))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(encoded, encoding="utf-8")
    print("D10 inventory=26; D9 independent Expected=24; D10 full-field Expected=0; Outcome C, NOT COMPLETE")


if __name__ == "__main__":
    main()
