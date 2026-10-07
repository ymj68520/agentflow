# AgentFlow ACE1 / Digest — Decision Register

> Phase 1A 状态。本文件不再是"开放问题清单"，而是 Phase 1 开放问题的
> **Decision Register**：每个原 OQ 条目已分类为
> `RATIFIED` / `RESOLVED_FROM_FROZEN_SPEC` / `REJECTED_AND_REPLACED` /
> `DEFERRED_TO_PHASE_2` / `DEFERRED_TO_SEMANTIC_VALIDATION` / `NON_NORMATIVE`。
>
> 规则：改变 Golden bytes 的决定不得 silent edit——必须记录规范版本变化
> 并更新冻结 corpus（当前 `corpus_revision = 2`，见 §尾）。

| # | 主题 | 状态 | 决定 |
|---|---|---|---|
| OQ-1 | ACE1 exact byte grammar | `RATIFIED_BY_IMPLEMENTATION_PHASE` | `spec/encoding/ace1-grammar.md` 自 Phase 1A 起为本仓库 ACE1 v1 第一版正式机器文法（已应用本阶段 Option 修正）。后续修改若改变 Golden bytes，不得 silent edit，必须记录规范版本变化并更新 corpus。 |
| OQ-2 | unit / null bytes | `RATIFIED` | 两者均使用 token `null`，由 schema nominal type 区分。这不意味着 `unit == null`——只表示 ACE1 在已知 schema 下允许两个 nominal primitive 共用同一 token representation。 |
| OQ-3 | Option encoding | `REJECTED_AND_REPLACED` | 原决定（`None → null`，`Some(v) → v`）正式废弃：它无法区分 `None`/`Some(null)`/`Some(unit)`/`Some(None)`，破坏 AFIDL closed type 组合闭包。**替换为**：`None → {"none":null}`，`Some(v) → {"some":v}`（single-case tagged object；exactly one key；key 必须为 `none`/`some`；`none` payload 必须为字面 `null`；`some` payload 按 T 校验）。 |
| OQ-4 | nested Option | `REJECTED`（禁止被撤销） | 原禁止 `option<option<T>>`/`option<unit>`/`option<null>` 的限制删除。Option 作为 sum type 在 bytes 中显式表达 case，因此任意有限合法嵌套均合法。不得为 wire ambiguity 削弱类型系统。`Shape::validate` 仅在未来 AFIDL validator 明确禁止真实递归/非法 schema 时才拒绝。 |
| OQ-5 | Decimal 规则 | `RATIFIED_FOR_ACE1_V1` | 无指数 canonical 形式；无前导零；分数 trailing-zero 规范化；`-0 → 0`；encoding 层文本精度无界。**只冻结 identity/canonical encoding**，不冻结未来 Workflow decimal arithmetic 精度。 |
| OQ-6 | integer `-0` | `RATIFIED` | canonical integer parser 拒绝 `-0`。 |
| OQ-7 | enum / variant JSON 表示 | `RATIFIED_FOR_ACE1_V1` | enum → canonical JSON string；variant → `{"case":payload}` single-key explicit tagged object。 |
| OQ-8 | float64 decimal 输入解析器 | `DEFERRED` / `NOT PART OF ACE1` | ACE1 canonical boundary 只接受 machine float value / exact bit identity。未来 permissive authoring parser（若有）不属于 Canonical Encoding。 |
| OQ-9 | Timestamp/Duration 日历范围 | `DEFERRED_TO_SEMANTIC_VALIDATION` | ACE1 v1 继续使用完整 Int64 microseconds machine range。 |
| OQ-10 | Digest domain 命名 | `RATIFIED` | kebab domain registry（`spec/tables/hash-domains.json`）为正式 v1 实现基线。禁止同时保留另一套 `agentflow.xxx.v1` 作为 HashV1 domain alias——一个 semantic domain 只能有一个 canonical domain string。 |
| OQ-11 | AFIDL grammar | `RESOLVED_IN_PHASE_2A` | AFIDL1 exact grammar 已于 Phase 2A 物化为 `spec/idl/afidl1-grammar.md`（repository normative source）；`machine-primitives.afidl` 已用正式语法重写、provisional 标记删除，并可被 `crates/af-idl` parser 真实解析、其 TypeSchemaDigest 已冻结于 corpus。详见 `docs/implementation/phase-02a-afidl-core.md` 与 `phase-02a-decisions.md`。 |
| OQ-12 | DigestRef Display format | `CONVENIENCE_ONLY` / `NON_NORMATIVE` | `sha256/{domain}/{hex}` 便利形式不参与 HashV1、ACE1、wire identity，除非未来独立规范。 |
| OQ-13 | String NFC | `RESOLVED_FROM_FROZEN_SPEC` | 非开放问题：冻结规范已规定 "Strings normalize to NFC"。所有 string primitive（record value / list item / map value / variant payload）一律 NFC。Map key 额外执行：normalize → unsigned UTF-8 sort → duplicate detection。 |

## Corpus provenance

- `corpus_revision = 1`：Phase 1 初次冻结（Option 采用 null-sugar 编码——已被 OQ-3 废弃）。
- `corpus_revision = 2`：Phase 1A——Option wire encoding 改为 explicit tagged
  object；新增 10 条嵌套 Option golden、10 条 Option/shape 负例；digest 部分
  仅追加 revision 字段，期望值与 revision 1 完全一致（已由 git diff 证明无
  无意变化）。
- 尚未发布正式 Spec Release，因此本次变化记为 ACE1 v1 pre-release artifact
  revision，无需升 Semantic Major；Phase 1A implementation report 已记录
  "Option wire encoding changed before v1 artifact freeze"。
