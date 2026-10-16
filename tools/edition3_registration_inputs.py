"""Exact G2-start registry inputs for unchanged Edition3 regeneration only."""
from pathlib import Path
import hashlib
ROOT=Path(__file__).resolve().parent.parent
PINS={'specifications':'345ec14a89c6bc774991dc955dc1c7f4185a34ac2c8b0677646ae3d52a98a230','rules':'034568ba56c84e0cd5715d06aa4302e682d0247bd9b4b9b81668de35ce545726'}
def pinned_bytes(name):
 b=(ROOT/f'spec/conformance/g01-native-guard-staging/edition3-{name}.json').read_bytes()
 if hashlib.sha256(b).hexdigest()!=PINS[name]:raise ValueError('Frozen Edition3 input changed: '+name)
 return b
