"""Frozen Edition2 INPUTS for original-directory historical regeneration."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent.parent
FOLDER=ROOT/"spec/conformance/g01-successor-staging"
PINS={"specifications":"d3afa5f16fba4cffb9f1a802e296ea1f33d844f5a75c792e116d8eef5254471f","rules":"8f93ec2e54bbeb248b518f2f60152695b2708a7bf57d3239415f588463d7ff3f"}
def pinned(kind):
 data=(FOLDER/("edition2-"+kind+".json")).read_bytes()
 # Bind complete rows to the R2 starting registry,not a current-edition label.
 manifest=json.loads((FOLDER/"edition2-input-pins.json").read_bytes())
 if hashlib.sha256(data).hexdigest()!=PINS[kind] or manifest[kind]!=PINS[kind]:raise ValueError("frozen Edition2 input drift "+kind)
 return data
