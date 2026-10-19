# R5-C — Relation C.source 的名义节点来源缺口

Outcome C — NEW FORMAL BINDING / STAGE GAP。发现号 `R5-C-REL-NOMINAL-01`。
不是 Native Validator 的 false-Proven，不撤销 G3 的输入域验证，也不是把
G02/T4 的实现未完成本身称为规范缺口。

起始完整 HEAD 为 `a9e296407c682a69f596be84ac8974eb870ebc18`，开始时工作树干净。
冻结元组为 Analysis@8 / Model@7 / Report@7 / Obligation@7。

## 第一条无法绑定的字段

```text
StructuralSubjectAdmission.observations[0]
  : CurrentContextRelationSchemaValidation
  .source : Node<NativeRelationSnapshotSource>

合法 G 中的实际来源：
RawValidationSubject.context.contents.relation_snapshots.effect_resource.some
  : RelationSnapshotContent
```

同一问题分别适用于 recovery_usage 和 flow_execution。所有 ModelType 的 owner
均为 `agentflow.computation.proof-report.model@7`，不是跨 Edition 混用。

当前 Descriptor 将三个 Option 的 item 都声明为 `ref:RelationSnapshotContent`；
后者的 body 是 `ref:NativeRelationSnapshotSource`。但 AF-SEMANTIC-0303 和
structural-binding 第 1 节明确规定 alias 委托 body/site，同时保留自己的名义
节点身份，不产生 cast。`Node<T>` 必须在确切载体中解析到确切名义 T。
C.source 的 output_record 和 w_value 两项引用策略又都要求 g_external、确切
`NativeRelationSnapshotSource`。没有 alias-to-Node 的许可规则。

因此 body 相同不能把 `Node<RelationSnapshotContent>` 当成所需的 Node。
这不是底层快照内容丢失：实际 family、revision、完整 pin/catalog/bytes 都已在 G。

## 最小真实复现与独立对照

新增测试 `raw_model7_capture_boundary`，不修改旧 G3 测试。输入使用原有
`c2r1/declaration_bundle/input-zero` 的实际 package、owner、slot、完整声明。
当前正式 17 个 AFIDL 模块真实编译，七个非 Relation 的 Universe/Catalog 分区为空。
每个适用 Snapshot 通过真实 Native 构造器建立，revision 为实际声明的
`g2-fixture`，不是反推历史 Schema。每次只保留一个 Some、其它两个 None。

分别覆盖三个 Slot 的 C Valid 与当前模块 Missing，再加全 None 对照，共 7 个
新 Boundary CaseIds。C 在 G 完成后真实执行；随后调用现有 Raw canonical_ports
Hook，全部保留真实 Port Zero / Violation / AF-CONTRACT-0004。

三个 Valid Some 的完整 G 都只有 `RelationSnapshotContent` 节点，所需
`NativeRelationSnapshotSource` 节点数为零。三个 Missing 情况还独立受 C 拒绝；
全 None 不执行 Snapshot 验证，不被本缺口阻断，亦不因此声称完整 Fact 已实现。
G Source/Context/Subject Carrier 的实际借用关联保留，C/Port 后 G 字节保持不变。

新 Python Oracle 从冻结 Descriptor、原始声明、真实规范源/Compiler 输入及独立
AFIDL 编译推导 Expected。逐节点、Body、Edge、Rank、Root、完整字节比较新 G，
再完整比较真实 C metadata、来源与 Port 结果；不从 Rust Actual 构造 Expected。
Boundary Comparator 的删除/双侧同时遗漏及伪造能力反例只记 Boundary 信用，
不计为 Raw Full-field Mutation。精确执行数与退出码由最终运行回执封存。

## 为什么不能在本轮修补

不能重标已有 G 节点；这违反实际输入字段声明和名义规则。
不能再复制一个相同 body 的 NativeRelationSnapshotSource 节点：现有唯一 Root
闭包无合法边到该名义节点，它会不可达；也不能补根、改 Root Profile 或重分配 Rank。
不能用 W-local 节点补 G-external 引用，不能用任意 U64、指针、Digest 或借用替代。
不能将 Some 改成 None。C Valid 也不能反向改变 G 的 Source Identity。

可能的修订方向包括 C.source 的正式目标、Alias 引用规则或输入图根/闭包规则；
本轮不选择、不实施任何一个方向。需要单独的正式设计裁决及后继注册。

## 停止边界与真实覆盖

已完成 R5-A Descriptor 清点：14 Root、690 个传递命名 Descriptor、3,106 条
声明 Field/Variant 行。这些是静态清单，包含其它 Profile 的条件分支，不是
Raw 运行时分母或生产信用。

历史 177 个事件、176 个原本适用 Fact、2,464 个 Root Instance 全部保留。
本轮新增 7 个真实边界事件，其中 3 个 C Missing 禁止完整 Fact；另 4 个属于
otherwise-applicable。合计事件要求 184、原本适用 Fact 180、Root 要求 2,520，
完整 Raw Root 比较为 0。完整 Nested/Relation/Observation/Source Association
分母在本停止点仍 UNKNOWN（机器文件 null），不填零，不冒充已闭合。

没有实现新的 Raw Producer/Collector；没有修改 G3 核心、旧 Raw Hook、正式
规范/Registry/Expected/Oracle、Native Equality、Family Pin、F1 lowering。
新旧 Strict Raw Total 均必须保持非零；新版 Exit 2 是本轮正式失败关闭结果。
Native Full Seed、Standalone、其它 T3、Total T3 仍 INCOMPLETE，T4 NOT ELIGIBLE，
Positive RootExport NotAuthorized。后续只等待本缺口的独立正式裁决。
