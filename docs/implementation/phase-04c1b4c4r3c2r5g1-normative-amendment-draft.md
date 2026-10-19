# Native Source Binding Amendment — 草案，未登记

以下 NB-DRAFT 编号不是正式 RuleId。本轮不分配 SpecId、Edition、ModelType rank 或 Variant。
所有 MUST 字样只表达拟议规范内容，**不改变当前任何注册含义**。

## NB-DRAFT-01 — 真实来源与精确六字段

Successor Raw pre-capture 输入中，每个 Some 的 NativeRelationSnapshotSource MUST 来自
已 source-admitted 的同一 actual Context 命名 Slot 的完整 Native Snapshot。保留 kind、
schema_family、revision、完整 pin、canonical_content、closed typed catalog；A integrity
采用既有0286/0287/0293约束。不得以 C、内容相等、digest、构造历史猜测代替来源。
None MUST 不产生 Source，不执行 snapshot/schema C。

## NB-DRAFT-02 — 三条正式生产边

Successor ContractProofContextContents.relation_snapshots 的 effect_resource、
recovery_usage、flow_execution 各 Option item MUST 为 exact ref NativeRelationSnapshotSource。
用相同的实际六字段无损投影重建，不使用旧 Node<RelationSnapshotContent> 的 cast。
原 Alias 在旧/其它声明路径的含义不变。Source type 已有，无须再添加同义 Source 类型。

注册改动闭包（不得只修改顶层 JSON）：

- 新 Context contents 的 relation_snapshots inline record Descriptor 与三 Option Descriptor。
- 对应 record/option 的 opaque site 定义、三 option_some child target、value binding。
- 经 RawContextBinding/StructuralDeclaredInputs/RawValidationSubject/ScopeSubject 到
  Source/Target/Origin/Basis/Witness/Contradiction/Evidence/FactKey 的新 owner 引用闭包。
- 当前六字段 NativeSource 的完整声明/site/ModelType 以及 C 源目标在新 tuple 中一致绑定。
- input/output eligibility、required field/variant/reference inventories 从实际新声明重算。
  不能预填新正式计数；旧317/66/1297保持原版。

## NB-DRAFT-03 — 单向阶段与借用关联

Actual declared inputs → source admission → pre_capture_binding → actual C → complete Fact
assembly MUST 单向。Source admission 保留实际 package/Owner/O/member/physical 与
analysis/canonical_validation 的 genuine same-context 关联。Source child 在 C 之前产生；
C output 不得作为 G source identity/body/child/rank 的输入。Frozen G 在后续阶段不可写。

## NB-DRAFT-04 — 固定引用域与来源校验

C.source 在 output_record 与 w_value MUST 均保留 g_external、exact Source target。
消费需在同一 enclosing G 解析正确 nominal type，并验证 Subject/Slot/actual Context 关联。
C.context/target/algorithm/current full definition closure/lookup/validation 按既有规范保留。
No W-local、output-local、cross-carrier、cross-edition 或 alias 兜底。

## NB-DRAFT-05 — 唯一根、完整 typed equality 与 rank

Root roster MUST 仍是一个 RawValidationSubject。Source 通过 Context 原正式父路径可达。
保持0217：完整 Descriptor/field/width/closure 校验，bottom-up exact typed interning，
height/type/body/child canonical ordering、零起连续 U64、所有子先于父、无不可达/冗余节点。
NodeBody 仍原八种；不追加 Source 专用 case。Type registration rank 不是 DagRank。
未来所有 ranks 必须由新 tuple 的全图重算，不复用旧 Alias rank。

## NB-DRAFT-06 — 身份分层

Native A Eq、typed canonical content、originating Source association、C observation MUST
分开。相同完整 typed Source 可以 intern；所有不同实际 Owner/O/Slot 的父关联、Origin
与输出 source path 必须保留。Slot kind/family/catalog Variant 精确受检。C Valid/Missing
不重定义 Native A，不能从当前 C 恢复构造历史或使不同 Scope 的完整 Fact 自动相等。

## NB-DRAFT-07 — 历史与版本隔离

新输入 Descriptor/站点含义与 Compiler support content 改变，MUST 使用新 exact Model、
Analysis、Report/Obligation tuple 及对应 semantics/source registration；编号后续单独登记。
旧70Specs/522Rules/1131Descriptors/2559Refs/97Imports和全部旧 Expected 保持原字节。
旧 graph 不可 retag；缺 actual native/source association 的历史输入不可自动迁移。
可重新提供真实 A/source/context 时，须按新 Compiler 独立重新绑定/验证，不继承 C 或能力。

## NB-DRAFT-08 — 失败关闭与非授权

使用已有 StructuralFailureDomain/Reason/Stage/path/expected_type/observed_type 精确表达
可复用失败；分类见 failure-admission-matrix。未知/缺失/错误/资源/未实现不可混淆。
不得把 stage/source failure 降成 port violation、empty evidence、C Valid 或 evaluated
Unproven。Missing C 保留真实 port event/verdict，禁止完整 Fact。G1/G2 的文档或注册
不能制造 VerifiedInputGraph/native receipt/CompleteView/RootExport 权限。

## 修订面分类

| 对象 | 必需正式修订 | 后续 implementation-only | 保持权威 / 历史隔离 |
| --- | --- | --- | --- |
| Normative source / Spec / Rule | 新版上述语义条款与依赖，正式 ID 待分配 | 无 | 旧源字节不改 |
| Descriptor / Site | 三 Some targets、父 inline/site 与递归 owner closure | 根据登记表生成新 binder 输入 | 旧 alias/站点独立 |
| ModelType rank | Source 已存在；本设计无需新类型 | 新全图 rank 依原算法 | 旧 rank prefix/意义不改 |
| Source input contract | 明确 actual A 投影正式生产边 | actual borrowed association admission | 不增构造历史 |
| G/W / C.source | successor owner closure 登记，域不改 | exact G/Subject/Slot 检查 | g_external 不降级 |
| Raw Subject / Basis/Witness | successor Context 传递闭包 | 不在G1实施 Fact producer | 所有 Origin/O 保留 |
| Report / Obligation | tuple/nominal closure 不同须新 exact edition | 以后全字段消费者接入 | 旧完整身份不重释 |
| CompilerSemanticsIdentity | 完整新源/Registry/role/支持表绑定 | 实现后验证真实 compiler | 不仅换版本字符串 |
| Portable Admission | exact tuple/完整 source 选择与拒绝混版 | 后续 portable integration | 不升级 metadata 为能力 |
| NodeBody / ACE1 / Hash domains / ABI / AFIDL | 本方案无新增或含义变化 | 原算法复用 | 全部冻结 |
| Native Eq / pin / bytes / revision / C verdict / port validator | 无 | 无 | 不重开 R3-CTX-REL-01 |

正式登记必须按实际新 closure 再算 Registry/Descriptor/Site 数量，不能从这份草案分配。
若登记发现需要改变 Native Source 字段、Eq 或 C Valid 含义，停止并另申请扩大裁决范围。
