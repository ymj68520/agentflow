# C2R1 — Occurrence-aware Raw Capture（Outcome D）

起点：`23a25b69d65166be3ec870ec1177ddd6ea78dc69`。固定代码门禁版本：
`dbdbba88e22e064b50e63c6b168b248d84ab3e9b`。

本次实现真实来源准入、封闭 Raw 元数据适配、真实 canonical_ports 分支事件、
本地 Basis/Origin/Target 视图与密封 Collector。它仍不是完整的正式
LocalTypedFactSeed、RawValidationFragment、G01 FactKey 或 Full-field Oracle。
不得把事件级比较通过或 Rust 测试退出 0 写成 Outcome A。

独立输入共 87 CaseIds：63 个来源准入成功，其中 59 个产生 Port 事件；
24 个来源负例。独立主 Capture 清单为 125 个已执行 Port 检查。三个空端口案例和
一个先行 Identity 失败案例没有 Port Fact，并非漏捕获。实际通过与完整退出
必须同时核对本前缀运行账本，不能以这些规划数替代执行证据。

正式 Fact 的 14 个根字段中，仅 role/evaluation/status/raw_validation_basis
有局部视图映射；这不等于这些字段的完整正式闭包已获证明。另 10 类根字段
尚未物化，对 125 个事件至少留下 1,250 个必需根字段实例缺口，另有嵌套字段
缺口。严格 Python 门禁在局部比较通过后仍返回 **2**；`--report` 的退出 0
只表示成功生成 Outcome D 诊断，不是完整 Raw Conformance 成功。

## Trust boundary

调用者必须先实际提供完整 SourcePackageMetadata、OwnerAnchor、真实 Slot 和
所选 RawDocument 的借用。适配器不会从候选值创建包、Owner 或历史物理字节。
来源只具 DeclaredOnly 意义；没有外部所有权、持久 Handle 或全球唯一性认证。
同内容不同 Owner/Slot 不合并，同完整来源重复观察不新增来源身份。

Compiler/Context 仍是实际完整借用对象；两角色必须引用同一个九成员 Context。
没有 resolver_dependencies 默认值。Native 源所有声明被完整转换为实际十三字段
Rust 值，保留原序列与文本；Physical 保留原始字节并先执行严格来源关联。
元数据适配器仅支持这一封闭 Raw 根，不是通用 wire decoder 或 Scope encoder。

局部 Proven 仅证明该次正数/唯一性比较。整个 Contract、Canonical/Checked 状态、
RootExport 和任何运行时权限均不能由此提升。Normalizer/G02/Native Full Seed/
Standalone/剩余 T3 不恢复；T4 NOT ELIGIBLE，Positive RootExport NotAuthorized。

## 交付导航

同前缀 source-producer、port-hook-collector、independent-conformance、
gates-and-findings、final-report 合并覆盖本阶段文档主题。
机器材料为 frozen-inputs、raw-input-corpus、producer-site-manifest、
occurrence-binding-manifest、expected-event-manifest、expected-field-manifest、
actual-comparison-manifest、mutation-manifest、historical-isolation、
design-vector-status、gate-index。
原目录全路径原始字节图、逐项日志与最终完整 HEAD 保留在
`E:/Projects/agent/target/phase-04c1b4c4r3c2r1-*`，避免提交自引用。
