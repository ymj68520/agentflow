# Port Hook / Prefix / Local Collector

canonicalize_contract 与 observe_raw_ports 共用 canonicalize_observed 和同一个
canonical_ports。原组件 Identity 校验仍在 Input 之前，Input 仍在 Output 之前。
排序前给实际元素附带 original ordinal，按实际 U32 稳定排序；返回值去掉 sidecar。
原来的 number==0 || number==previous、首错及 Schema 查询位置保持不变。

拒绝分支在 ContractError 字符串构造前记录 PortOccurrence、实际前驱和
Zero/Duplicate；正分支在 Schema resolve 前记录 PositiveFresh。前驱起始 None，
不会把 previous_number=0 哨兵伪造成一个输入声明。两方向单独重置命名空间。
Schema 查询失败保留此前真实局部 P，后续端口没有事件。
Identity 失败记录 ComponentIdentity stop 且没有端口事件。没有把未到达事件
编码成 evaluated Unproven，也不把其它 Canonical 错误通用化为本 Predicate。

PortEvent 只有来自真实运行的 RawPortObservation 才能构造密封 Capture。
公开的事件数据副本不能提交给 Collector 来制造 Proven。
LocalRawSeed 状态、Witness、事件字段私有；RawLocalCapture 字段私有，关联
Target/Origin/Basis 的方法还检查 Seed 属于本 Capture，不能拿等值外部副本换源。

## 保留的本地类型化数据

- occurrence：direction/original_ordinal/sorted_ordinal/U32 port_no。
- basis：真实完整 Subject 借用、occurrence、真实 predecessor、condition、
  selected_error 和完整现行 RuleContentIdentity。
- zero witness：occurrence/U32(0)/minimum U32(1)。
- duplicate witness：occurrence、真实前驱 original ordinal 和相等正数。
- target：完整 Subject、实际编译 ComponentContract 类型定义、完整规则、方向
  和 positive_unique_io_ports，绝非 early-rejected Contract 的 Canonical digest。
- origin view：完整来源、方向/original/source ordinal、mapping=None 与声明路径。

DeclaredStep 从 Subject.input.candidate.(inputs|outputs)[original].port_no 解析。
RawDocumentMember 从真实文档内部 Entry 接到同一原始成员；外部 Owner Slot 不
混入内部路径。测试逐项保留字段/类型名/原 ordinal 并核对完整候选值。
局部类型标签尚未完成正式 ModelType {owner,name} 的独立投影，这是已报告缺口。

## Collector 边界

RawLocalCollector 只接收密封的真实 RawLocalCapture。validate 对原始声明、
排序位置/前驱、真实事件带与 Seed 数量、P/V/Witness 一致性作检查。合并前
预检冲突，拒绝后不改写已收集结果。8 项私有 Payload 损坏回归覆盖 false-P、
缺 Witness、错方向/两 ordinal/前驱/condition/最小值，重复来源合并不增 Fact。

Collector 自身是显式有限联合审查：保留所有实际提交包，包括准入成功但无
适用 Port Fact 的包；增加新包前调用原正式联合包准入检查。相同 Anchor 的
不同完整 Owner 记录以 Source(OwnerBindingConflict) 拒绝，即使两个各自的
Capture 都已单独准入。新增真实双 Capture 回归验证原子拒绝，已有 Fact/包
集合不变；该回归与 8 项私有 Payload 损坏分开记录，不计为新的 Capture CaseId。

CaptureFacts 和真实 ContractDiagnostic 是两个受控本地观察。后者只接受
真实 PortRejection stop，并重新取得原 Contract 诊断；不是从错误文本推断
Witness。合法错误载体与本地 Fact 可成为两个观察，不成为两个来源身份。
这些仍是本地观察枚举，不是已经物化的正式 ObservationSite/typed map-key path。

缺失的正式 FactKey、EvidenceRef/primary、Prerequisite/Consumer、完整
TypedContradiction 与 ObservationSite 使完整 LocalTypedFactSeed 仍 INCOMPLETE。
没有 CompleteView、Canonical ObligationIdentity、Scope encoder、Normalizer 或
CheckedRootExport 构造路径。普通 Canonical 诊断成功也不能提升为 Checked。
