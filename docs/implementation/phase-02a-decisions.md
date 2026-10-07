# Phase 2A Decisions — AFIDL1 Core

> 本文件记录 Phase 2A 中冻结文档未唯一确定、但实现必须做出的设计决定。
> 每条都是显式 Decision，不是 open question；除非改变 Golden bytes，否则
> 无需重新开放。

| # | 主题 | 决定 | 理由 |
|---|---|---|---|
| D-1 | enum 编号（`value_no`） | **采纳**。enum 值携带显式 `u32` 编号（1..=UINT32_MAX、唯一），canonical 顺序按 `value_no` 升序。 | 冻结文档只明确 field_no/case_no。enum 若无稳定编号，其演化 identity 只能退化为 source order（隐性 ABI，不可接受）。最小一致方案是给 enum 与 field/case 相同的编号待遇。编号只参与 schema identity；ACE1 value 仍是值名字符串（Phase 1A 冻结不变）。 |
| D-2 | 类型级 `@major` 移除 | **采纳任务 §五十一 推荐方案**：只有 `afidl 1;`（语言层）与 `schema <id> @<major>;`（family 层）两层；声明不带 `@N`，major 由 module 继承。完整 identity 为 `<SpecificationId>/<TypeName>`。 | 避免module @1 + type @2 的语义不清问题；grammar 更小；两种写法并存被禁止。 |
| D-3 | 递归类型 | **Phase 2A scoped restriction**：direct 或 indirect（经 record/variant 字段、alias 展开、任意 list/map/option 包装）的 recursive NamedRef 一律拒绝（`recursive_type_not_supported`）。 | inline structural recursion 无法产生有限 ACE1 value；循环 canonical schema hash 超出本阶段范围。此限制是本阶段范围决定，**不**声称来自冻结文档；未来若引入显式 reference/object 语义可解除。 |
| D-4 | UnknownPrimitive 错误类别 | 不单独存在。未知（未登记）小写标识符在 type 位置一律解析为 `NamedRef`，由 validation 报 `unknown_type_reference`。 | 任务 §七 明确指示此行为；parser 不得接受"可能是 primitive"的标识符。 |
| D-5 | SchemaModuleDigest | **Deferred**。本阶段只实现单类型 `TypeSchemaDigest = HashV1("type-schema", ACE1(schema))`。 | 模块 digest 若使用新 domain `schema-module`，属于 normative registry addition（Phase 1 registry 无此域，任务 §三十四/§三十五 禁止顺手加）；若复用 `type-schema` 则需要显式规范决定。二者均留待后续阶段批准。 |
| D-6 | 空 aggregate | 空 `record` 合法（ACE1 编码 `{}`）；空 `variant` / 空 `enum` 拒绝（`empty_variant` / `empty_enum`）。 | 空变体/空枚举是无居所类型，ACE1 无法编码任何值；空 record 有真实用途（marker）。 |
| D-7 | Canonical schema 的 ACE1 Shape 展开 | schema-of-schemas 的 `TypeExpr` 是递归类型，而 af-core `Shape` 是有限树。对每个 canonical schema，按其自身表达式深度 +1 构造有限 Shape 展开后用冻结的 `af_core::encode` 编码。 | 编码字节与理想无限 Shape 完全一致；未修改 ACE1 wire grammar（Phase 1 corpus 全量回归证明）；深度 0 的不可达 stub 一旦被访问将以 TypeMismatch 显式失败。 |
| D-8 | Identifier 字符集 | AFIDL 语法与标识符限定 ASCII；不做 case folding、不做 Unicode 归一化。lexer 允许 `-` 进入小写 token（为 kebab namespace），但 field/case/value name 位置严格校验 `[a-z][a-z0-9_]*` 并拒绝 `-`。 | 任务 §十 建议；标识符 normalization 必须显式而非隐式。ACE1 value 字符串的 Unicode 规则不适用于 schema 标识符层。 |
| D-9 | 整数字面量 | leading zeros 是 lex error；数值必须 fit `u32`；`0` 可 lex 但在 field/case/value 编号位置被 validation 拒绝。 | 确定性 source 纪律；编号是数值而非文本，canonical 模型只存数值。 |
| D-10 | machine-primitives.afidl 的形态 | primitives 保持为 builtin TypeExpr（不声明为用户类型）；该文件声明 13 个 nominal alias（`Unit = unit` 等）使自身成为可解析 AFIDL1 module，其 13 个 TypeSchemaDigest 已冻结于 corpus；conformance 测试执行 enum == JSON 表 == afidl 别名目标集合 三方 drift 检查。 | 任务 §三（.afidl 必须成为 source of truth 且可解析）+ §五十五（primitive 不做用户声明，避免不必要递归）。alias identity ≠ primitive identity，与 nominal 区分原则一致。 |
| D-11 | 上下文关键字 | `afidl/schema/alias/enum/record/variant/list/map/option` 仅在语法位置解释为关键字；field/case/value 名称允许复用这些词。第二处 `schema` 声明报 `duplicate_schema_declaration`；其他未知声明关键字报 `parse_error`。 | grammar 最小化 + fail closed（任务 §四十二）。 |
| D-12 | kebab namespace 段 | ns 段允许连字符（`agentflow.core.machine-primitives`）。lexer 将 `-` 并入小写 token（AFIDL 无减法，无歧义），name 位置由 parser 拒绝 `-`。 | 既有 registry 文件名已使用 kebab；SpecificationId 校验器按 `[a-z][a-z0-9-]*` 严格验证。 |
