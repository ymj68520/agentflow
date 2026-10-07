# Phase 1 — Open Questions & Materialized Decisions

> 依据任务规则第 五.5 条：无法从冻结文档唯一确定的行为不擅自发明为"规范"，
> 全部显式记录于此。每一条都标注了：文档依据、本阶段的落地方案、以及待
> 确认事项。**这些条目不阻塞本阶段交付；它们等待正式 AFIDL/grammar freeze
> 时批准或修正。**

| # | 问题 | 文档状态 | 本阶段落地决策 | 待确认 |
|---|---|---|---|---|
| OQ-1 | ACE1 **exact byte grammar**（record/variant/enum/option 的 JSON 结构形式、转义表细节）在 docs 中被列为"待物化硬门槛"（§22.21、§25.8），冻结的只有核心约束（§24.1：NFC、key 排序、数值字符串化、missing≠null、closed fail-closed） | 待物化 | 以 `spec/encoding/ace1-grammar.md` 物化完整文法；严格遵守全部冻结约束，结构性选择（见 OQ-2~6）单独列出 | 正式 grammar freeze 时逐条批准 |
| OQ-2 | `unit` 与 `null` 的字节表示：文档把二者列为不同 machine primitives，但未给出各自的字节形式 | 文档无规定 | 二者都编码为 JSON `null`；它们是不同 nominal 类型，由 schema 位置消歧（typed encoder 拒绝交叉赋值）。任何选择都会与其他空形式碰撞（`{}`/`[]`），`null` 是最自然的渲染 | 若 freeze 要求 unit 有独立字节形式（如 `{}`），改动仅影响 encoder/decoder 各一 arm 与 corpus |
| OQ-3 | `Option<T>::None` 的编码：显式 `null` vs 字段缺省（absent key）。文档原文 "Missing and null are distinct; Option/Variant are explicit" 存在两种读法 | 文档无唯一确定 | 采用 **None → 显式 `null`**：使 "Option explicit" 成立（None 有自己的字节形式），同时 "missing ≠ null" 保持为强约束——canonical bytes 中**不存在**缺省字段状态，required 字段缺失是错误。备选方案（None → absent key）已在此记录，若 freeze 采纳备选，只需改 encoder/decoder 的 option arm + corpus | 二选一确认 |
| OQ-4 | 嵌套 option（`option<option<T>>`、`option<unit>`、`option<null>`）在上述任一方案下都不可无损表示 | 文档无规定 | `Shape::validate` 直接拒绝（`invalid_shape`），与 protobuf 禁止嵌套 optionality 的行业惯例一致 | 若 v1 需要嵌套 option，需要专门的 tagged 包装设计 |
| OQ-5 | Decimal 的精确规则：文档只说 "明确 decimal 表示" + "字符串规范化"。scale/精度界、指数表示法、输入规范化范围均未定 | 文档无规定 | 物化于 `spec/encoding/ace1-grammar.md` §4：无指数记法；前导零拒绝；trailing 分数零规范化；`-0`→`0`；精度无界（字符串承载，Phase 1 只做 identity/canonicalization，不做算术） | scale 上限、是否允许输入端指数记法 |
| OQ-6 | 整数文本 `-0`：文档禁止 `+` 与不规范前导零，未提 `-0` | 文档无规定 | 视为非规范文本，**拒绝**（integer primitives 不存在负零值） | 确认 |
| OQ-7 | Enum / Variant 的 JSON 形式：文档只说 "closed / fail closed" | 文档无规定 | enum → 值名 JSON 字符串；variant → 单键对象 `{"case":payload}`；enum 值/变体 case/record 字段名均要求 NFC 且唯一 | 确认 |
| OQ-8 | Float64 的**输入**形式：冻结规范只定义了 16-char hex 输出形式，未定义是否有十进制文本输入路径 | 文档只定义输出 | 本阶段 float64 值只能经 `f64` 位模式或有限 `f64` 构造；canonical bytes 一律 hex 文本；不存在十进制→float 的解析路径（避免引入宿主 float formatting） | 是否需要 decimal-text 输入解析器（将来若需要，也只属于 permissive parser，不属于 canonical boundary） |
| OQ-9 | Timestamp/Duration 的语义范围：文档只规定 int64 microseconds，未限制日历/数值范围 | 文档无规定 | Phase 1 使用完整 i64 域，不做日历验证；nominal 区分由类型系统保证 | 后续阶段若定义可表示范围（如 RFC3339 可渲染域），在 validator 层收紧 |
| OQ-10 | Digest domain 命名冲突：§22.3 示例使用 `agentflow.plan.v1\0` 风格字符串，§24.1（更靠后的冻结基线）使用裸 kebab 名（`plan-core` 等）并冻结 HashV1 公式 | 冲突 | 按规则"后者优先"采用 §24.1：裸 kebab 域名 + 集中登记表（`spec/tables/hash-domains.json`，14 个域） | 若 §22.3 风格才是意图，仅需改 registry 表与 enum 字符串 |
| OQ-11 | AFIDL1 本身的语法尚未冻结（文档说 AFIDL1 是 schema source of truth，但没给文法） | 待物化 | `spec/idl/core/machine-primitives.afidl` 用最小声明式记法物化 primitive/containers 集合；有 drift 检查测试保证其与实现一致；未实现 AFIDL parser（按任务 §十五 不要求） | 正式 AFIDL 文法定稿后重写该文件 |
| OQ-12 | `DigestRef` 的组合展示形式（`sha256/{domain}/{hex}`）是本阶段为调试方便提供的，非 wire format | 文档无规定 | 实现为 `Display`/`from_display`，文档标注 convenience-only | wire 格式（若有）留待 ID/protocol 阶段 |
| OQ-13 | Map value 的 NFC 归一是否也适用于 value 中的 string？文档只对 key 明确要求 NFC | 文档对 key 明确、对 value 推断 | string primitive（无论位置）统一 NFC——否则 map value 中的等价 Unicode 会破坏 "equivalent input → same bytes" | 确认 |

## 生成器与 corpus 的一致性说明

`tools/gen-conformance.py` 是 corpus 的**一次性独立生成器**（Python 独立实现
ACE1 + HashV1）。在开发过程中它成功抓到过一次两实现分歧（decimal 输入规范化
缺失），验证了双实现交叉冻结的价值。corpus 中的期望值已经物理冻结；测试永不
在运行时生成期望值。若未来修改冻结规则，必须：改本文件 + 改生成器 + 重新
冻结 + 在 phase 报告中记录 corpus 版本变化。
