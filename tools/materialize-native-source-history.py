"""One-time exact Analysis8 registry snapshot, before any successor registry write."""
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
D=R/'spec/conformance/g01-native-source-staging'
base=json.loads((R/'target/phase-04c1b4c4r3c2r5g2-start-raw-map.json').read_bytes())
for name in ['specifications','rules']:
 source=R/f'spec/registry/{name}.json';out=D/f'analysis8-{name}.json'
 expected=base[source.relative_to(R).as_posix()]
 if out.exists():
  assert hashlib.sha256(out.read_bytes()).hexdigest()==expected
 else:
  data=source.read_bytes();assert hashlib.sha256(data).hexdigest()==expected
  D.mkdir(parents=True,exist_ok=True);out.write_bytes(data)
print('Exact Analysis8 Registry source bytes retained; no identity conversion')
