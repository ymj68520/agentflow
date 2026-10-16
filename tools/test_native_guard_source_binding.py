"""Negative laws for the user-authorized dual source binding, not Capture."""
import copy,json,re,unittest
from pathlib import Path
from native_guard_source_binding import ROOT,check
class SourceBindingLaws(unittest.TestCase):
 def setUp(self):
  self.model=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
  self.manifest=json.loads((ROOT/'spec/conformance/g01-native-guard-staging/historical-imports.json').read_bytes())
  self.path='crates/af-contract/src/recovery_guard.rs';self.live=(ROOT/self.path).read_text(encoding='utf-8')
 def test_exact_live_imports(self):self.assertEqual(check(self.model,self.manifest)['live_declarations_compared'],90)
 def test_unrelated_implementation_body_is_not_type_identity(self):check(self.model,self.manifest,{self.path:self.live+'\nfn r3_body_probe() { let _ = 1u32; }\n'})
 def test_live_field_type_change_rejects(self):
  changed=self.live.replace('port_no: u32','port_no: u64',1)
  self.assertNotEqual(changed,self.live)
  with self.assertRaises(AssertionError):check(self.model,self.manifest,{self.path:changed})
 def test_variant_rank_change_rejects(self):
  changed=self.live.replace('FrozenGuardContainment,\n    ZeroPossibleActions,','ZeroPossibleActions,\n    FrozenGuardContainment,',1)
  self.assertNotEqual(changed,self.live)
  with self.assertRaises(AssertionError):check(self.model,self.manifest,{self.path:changed})
 def test_snapshot_content_change_rejects(self):
  other=copy.deepcopy(self.manifest);other['sources'][self.path]['content_hex']+='00'
  with self.assertRaises(AssertionError):check(self.model,other)
 def test_both_sided_corrupt_snapshot_rejects(self):
  a=copy.deepcopy(self.model);b=copy.deepcopy(self.manifest)
  a['historical_import_sources'][self.path]['content_hex']+='00';b['sources'][self.path]['content_hex']+='00'
  with self.assertRaises(AssertionError):check(a,b)
if __name__=='__main__':unittest.main()
