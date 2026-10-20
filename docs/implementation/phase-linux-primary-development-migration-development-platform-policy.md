# Linux Primary Development Policy v1

Status: Active engineering routing policy; Development Acceptance is not yet Passed.
Authority: explicit human L2-PIVOT-01 directive (SHA256
`6ac84865e18b0566ca5af51d592391fce507e14db7e86cf6985e37440743977d`).
Registered separately in `engineering/policies/index.json`; this is not a product
Spec/ABI/Edition/Compiler registration and grants no RootExport capability.

Primary development: verified `ssh fwq`, workspace
`~/projects/agent-r5g2-linux-validation/workspace`, branch
`codex/linux-primary-migration`. Rust/Cargo stay at installed 1.91.1, Python3.12.3.
Windows `E:/Projects/agent` remains original audit/backup, not daily development.
The desktop chat executor has not automatically moved hosts; AGENTS.md routes
future engineering work through SSH. Do not modify frozen checkouts or global PATH.

Development Acceptance requires Linux fmt, Clippy, unit/integration tests,
spec-check, independent Oracle and negative-case results at the actual source
identity. The restored 252 static cases are one subset, not full Rust acceptance.
Cross-target Compatibility requires separately measured Windows x64 target work.
Windows Release Acceptance requires real Windows x64 release execution; neither
Linux PASS nor a GNU cross-build implies Windows Verified or MSVC equivalence.

Runtime build artifacts: `../build/linux-primary-v1`. Use project-local venv and
isolated Cargo home from the policy JSON. Cargo test subprocess PATH must include
that venv: several frozen Rust tests invoke `python`. No new global aliases.
No VM/Wine, sudo, automatic Rust upgrades, dependency drift or automatic Push.
