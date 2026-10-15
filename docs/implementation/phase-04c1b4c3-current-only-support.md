# Current-only admission — actual staged behavior

Independent source-built inputs live in `spec/conformance/g01-edition2-staging`. They are constructed afresh from registered source bytes and unchanged AFIDL declarations,not by renaming an old Expected report. Old complete bytes appear only as explicit rejection inputs.

| Input | Actual required boundary |
| --- | --- |
| Old label +old complete content | D4 UnsupportedSemantics;only pinned old build can validate historical positive |
| New label +old content | D4 InvalidIdentity |
| Old label +new content | D4 InvalidIdentity |
| New exact complete content | D4 UnsupportedSemantics with explicit Registered/NotImplemented/NotAuthorized detail |
| Unknown edition | D4 UnsupportedSemantics |
| Missing normative source | D3 IncompleteContent |
| Noncanonical ACE1 | D1 InvalidCanonicalEncoding |
| Caller edition_supported field | D0 InvalidCanonicalEncoding;closed schema |
| Same label +internally consistent changed rule registry | D4 InvalidIdentity |
| Native source missing /identity resource bound | IncompleteContent /ResourceLimit |
| Unknown report shape/obligation token | UnsupportedSemantics via native registration probe;no decoder/capability |

Priority:resource/canonical closed shape,then D2 references,D3 internal content,then D4 exact owner/support. Errors are typed,never ordinary static Violation. Native complete claims undergo full-byte checks;unknown/old claims do not silently become current. Current-state matches do not create SupportedPortableStaticInputs while support/activation is unavailable.

The old in-process return carrier is retained only as a tagged unsupported diagnostic:its typed edition_admission_failure is separate from its legacy Unproven placeholder. No dimensions were evaluated and no full G01 report exists. Public callers cannot set production support. An old report cannot be submitted to mint Checked. See [constructor review](phase-04c1b4c3-capability-fail-closed.md).
