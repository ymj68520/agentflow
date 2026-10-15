# Current-only admission and failure priorities

Production owns Registered/Supported/Activated/release states. Current successor is Registered / NotImplemented / NotAuthorized / NotAuthorized. There is no caller-controlled supported/activation override. Registered descriptors neither enable old direct proofs nor let standalone fragments satisfy root CompleteView.

|Input|Actual staged result|
|---|---|
|Complete successor/native identity|Identity recognized; positive proof denied|
|Complete successor/Portable|D4 UnsupportedSemantics, no capability|
|Exact historical Edition2 in current build|UnsupportedSemantics|
|Edition2 labels with current full registry|InvalidIdentity|
|Successor labels with old full model/registry|InvalidIdentity|
|Missing required new model source|Native IncompleteContent; Portable D3 IncompleteContent|
|Unknown predicate/model/edition|UnsupportedModel / UnsupportedSemantics|
|Missing one context role|Shape probe IncompleteFacts|
|Noncanonical ACE1 spacing|D1 InvalidCanonicalEncoding|
|Caller support/activation fields|D0 InvalidCanonicalEncoding (closed input shape)|
|Resource excess|ResourceLimit, no partial identity or capability|
|Standalone as root|UnsupportedModel|

Bounds/closed transport decoding precede canonical/content/support checks. Thus a missing source can reject in D3 before D4 support. Admission failure is never a substitution dimension Violation. Known old malformed-but-noncurrent content is not a positive path and may be unsupported before any semantic comparison. Future diagnostic refinements need no activation and must preserve priority.
