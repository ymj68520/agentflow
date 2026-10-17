# C2G2 — Document Occurrence Registered Staging

起点e623aab705c5f8269807201e112a32e67e6869bf，原树干净，2076非忽略文件。
本阶段注册C2G1 Option C，不实施Raw Hook/TypedFactSeed/完整Producer Oracle/
G02 Scope/Normalizer或正向RootExport。

实际无冲突分配：Analysis agentflow.root-export-checker@6，Model
agentflow.computation.proof-report.model@5，Report agentflow.g01.report-shape@5，
Obligation agentflow.g01.obligation@5。分配前核对旧58规范/479规则、路径与RuleId。
新增4规范、14规则0272–0285；当前62/493/492covered/1exception/0missing。
新增21描述符/46引用，当前704/2064；旧683 ranks及97历史导入保留。

当前254注册类CaseIds=170 shape+14 rule+51 metadata admission+8有界DAG子图+
9 Portable准入+2运行时Portable负例。88字段/Variant站点双向覆盖。
这些不是254个Rust tests，也不是Raw Capture。新Raw Capture CaseIds始终0。

新API只返回Result<(),…>，不生成Source身份字符串、Scope编码、Fact、Witness、
Checked或能力。完整SourcePackage内容与真实借用关联都保留；键只表示DeclaredOnly。
现有端口/Guard/Bytes/String/Bounds/Substitution/Countermodel算法均不改。

源码门禁HEAD：4ce26eb5ce32d291d638ddb7a6d1c2fd602fa385。
最终HEAD及实际门禁结果见target/phase-04c1b4c4r3c2g2-final-runtime-results.json。
仅在完整当前/五组历史/独立审计及最终原目录双轮均通过后裁决Outcome A。
