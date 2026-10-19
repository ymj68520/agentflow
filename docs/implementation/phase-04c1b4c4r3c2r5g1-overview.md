# R5G1 — Native Relation Source 名义绑定设计

本文件及 NB-DRAFT 条款均为设计材料，不是 Registry、RuleId、Edition 或能力。
起始 HEAD：`774ff6046ec9f3fbd6a2b84101b6e72faf4bbe2f`；起始工作树干净。

目标：裁决 `R5-C-REL-NOMINAL-01`。完整注册清点与真实七例边界分别见
existing-binding-census.json 与 boundary-diagnostics.json；后者只能在真实门禁后生成。

设计推荐 Option A 的**直接输入边改绑**形式：在未分配编号的 successor Model 中，
仅将 `ContractProofContextContents.relation_snapshots` 三个 Option 的 Some 目标从
`RelationSnapshotContent` 改为 `NativeRelationSnapshotSource`，为这些边注册确切站点。
未来 Binder 从同一个实际 Native Snapshot 重新无损投影六个字段，不转换已有别名节点。
沿 Subject→context→contents→relation_snapshots→slot→Some 的既有方向保持可达。
不增加根、反向边、普通整数引用、NodeBody Variant 或虚构来源。

Source 已在当前 Catalogue 注册，六字段齐全，当前 ModelType rank 为754；缺的是
Raw 输入根闭包的**节点生产边**，不是 Source Descriptor 本身，也不是 Native 数据。
当前别名的 ModelType rank107、Source rank754 都不是 DagRank。本轮不分配任何新 Rank。

选择此方案依赖三个独立证明：A 数据在 C 前可得；未来确切输入边可注册且不改 A 意义；
Canonical DAG 按完整类型和子树重新生成可达 Source。旧 Model@7 图仍不具备该边，
不得因设计推荐而将旧图 retag 成 successor 图。

Owner/Occurrence 不塞入六字段 Source，也不凭调用次数制造来源。Native A 的相等、
完整类型化值的相等、Subject/Origin 的来源关联、C observation 的相等各自保留。
完整相等 Source 可共享，两个不同实际 Owner/Occurrence 的父边和 Origin 不能丢。

版本冻结：Analysis@8 / Model@7 / Report@7 / Obligation@7；70 Specs、522 Rules、
521 covered、1 intentional exception、0 missing、1131 Descriptors、2559 Refs、
97 Imports、ABI775、AFIDL17、HashV1 domains14、ACE1v1、317 Sites、66 Policies、
1297 Field/Variant。新增设计文件和 Oracle 不属于正式注册计数。

最终 Outcome 必须等工程、独立回归、原目录生成器双轮和字节冻结全部结束才决定。
动态权威回执为 `target/phase-04c1b4c4r3c2r5g1-final-runtime-results.json`，不是本设计说明。
Raw Full-field、Native Full Seed、Standalone、其它 T3、Total T3 均 INCOMPLETE；
T4 NOT ELIGIBLE，Positive RootExport NotAuthorized。注册和实施都须另行授权。
