"""Gap detection/shape tests, NOT D10 semantic conformance closure."""

import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from oracle import field_contract as FC
from oracle.d9_algebra import OracleGap
from oracle.d10_gap_audit import (ROOT, OUT, GAP_ID, artifacts, gap_ledger,
                                 materialize, registration, require_full_d10_expected)


class D10GapAuditTests(unittest.TestCase):
    def test_original_groups_and_materialized_identities(self):
        cases = registration()
        variants = [case for case in cases if case.get("mutations")]
        self.assertEqual(len(variants), 20)
        values = [materialize(case) for case in variants]
        self.assertEqual(len({raw for _, raw in values}), 20)
        self.assertEqual([bundle["root_export_assertions"][0]["assertion"]
                          ["execution_mappings"][0]["source_node"] for bundle, _ in values],
                         [str(node) for node in range(2, 22)])

    def test_available_d9_expected_is_full_shape_checked_but_not_d10(self):
        outputs = artifacts()
        manifest = outputs[OUT / "manifest.json"]
        self.assertEqual(manifest["d9_expected_count"], 24)
        self.assertEqual(manifest["d10_full_field_expected"], 0)
        gaps = [row for row in manifest["cases"] if row["expected_d9_file"] is None]
        self.assertEqual(len(gaps), 2)
        self.assertTrue(all("/p31-" in row["case_id"] for row in gaps))
        for path, value in outputs.items():
            if path.parent == OUT and path.name != "manifest.json":
                self.assertEqual(FC.validate(value["facts"], FC.ref("D9Facts")),
                                 value["instance_coverage"])

    def test_full_d10_expected_fails_closed_for_every_original_case(self):
        for case in registration():
            with self.subTest(case_id=case["case_id"]):
                bundle, _ = materialize(case)
                with self.assertRaisesRegex(OracleGap, GAP_ID):
                    require_full_d10_expected(bundle)

    def test_obligation_is_required_exact_order_not_reason_prose(self):
        field = FC.DEFS["RootExportReport"]["fields"]["unresolved_obligations"]
        self.assertNotEqual(field.get("classification"), "DiagnosticOnly")
        self.assertEqual(field["ordering"], "exact")
        self.assertEqual(field["element"], {"kind": "string"})
        self.assertEqual(FC.DEFS["LeafProof"]["fields"]["reason"]["classification"],
                         "DiagnosticOnly")

    def test_all_normative_gap_citations_resolve(self):
        for finding in gap_ledger()["findings"]:
            for reference in finding.get("normative_sources", []):
                path, anchor = reference.split("#")
                self.assertIn('<a id="' + anchor + '">', (ROOT / path).read_text(encoding="utf-8"))

    def test_missing_registered_mutation_member_is_not_repaired(self):
        case = copy.deepcopy(registration()[0])
        case["mutations"] = [{"op": "set", "path": ["not_registered"], "value": []}]
        with self.assertRaises(KeyError):
            materialize(case)

    def test_lossy_projection_is_not_a_full_content_contract(self):
        identity = FC.DEFS["CompilerIdentity"]["fields"]
        self.assertEqual(identity["registry_content_bytes"]["kind"], "u64")
        manifest = FC.DEFS["ContextManifest"]["fields"]
        self.assertNotIn("execution_universes", manifest)
        self.assertEqual(manifest["usage_meter_count"]["kind"], "u64")


if __name__ == "__main__":
    unittest.main()
