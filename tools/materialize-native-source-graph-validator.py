"""Mechanical version isolation of the existing METADATA-ONLY verifier.

No production input_graph/input_binder code is copied, called or modified.
The algorithm and hard metadata bounds remain byte-identical except imports.
"""
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
p='crates/af-computation/src/structural_graph_metadata.rs'
raw=(R/p).read_bytes();baseline=json.loads((R/'target/phase-04c1b4c4r3c2r5g2-start-raw-map.json').read_bytes())
assert hashlib.sha256(raw).hexdigest()==baseline[p]
text=raw.decode('utf8').replace('use crate::structural_staging::{','use crate::native_source_graph_support::{').replace('super::structural_staging::STRUCTURAL_MODEL_ID','super::native_source_staging::MODEL')
out=R/'crates/af-computation/src/native_source_graph_metadata.rs'
out.write_bytes(text.encode('utf8'))
print('Version-isolated metadata verifier; algorithm/bounds unchanged; no producer')
