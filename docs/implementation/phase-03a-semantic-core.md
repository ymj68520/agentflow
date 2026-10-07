# Phase 3A Implementation Report — Core Semantic Type Universe

> 状态：完成。本阶段实现 Workflow Semantic Layer 的**数据模型底座**：workflow
> primitive universe、Semantic Type DAG、ValueRef/ValueSlot、Pure Expression、
> ResourceSlot 引用模型。未实现任何 13 个 Computation Node、`SemanticIRDocument`、
> Executable Plan、Journal、Reducer 或 Runtime。

## 1. semantic-core AFIDL 最终类型列表（29）

`spec/idl/semantic-core.afidl` — `schema agentflow.semantic.core @1;`

| 类别 | 类型 |
|---|---|
| Nominal id aliases (4) | `SemanticTypeId` `ValueSlotId` `ExpressionId` `ResourceSlotId` |
| Primitive universe (1) | `SemanticPrimitiveType` (enum, value_no 1..9) |
| Digest / schema ref (3) | `DigestRefValue` `TypeSchemaRef` `SemanticTypeNodeBody` |
| Type DAG (2) | `SemanticTypeNode` + `SemanticTypeNodeBody` |
| Value slots & constants (2) | `ValueSlot` `PrimitiveConstant` |
| Expression payload records (13) | `RecordFieldExpr` `MapEntryExpr` `RecordConstructBody` `VariantConstructBody` `EnumValueBody` `ListConstructBody` `MapConstructBody` `OptionNoneBody` `OptionSomeBody` `FieldGetBody` `EqualsBody` `NotBody` `AndBody` `OrBody` `IfThenElseBody` |
| Expression root (2) | `ExpressionBody` (variant, case_no 1..15) `ExpressionDef` |
| Resource (2) | `ResourceSlot` `ResourceSlotId` |

说明：AFIDL1 没有内联 record 类型，因此每个复合 variant payload 用命名 record 承载
（13 个 payload record）；它们只是 payload 载体，不是独立领域对象，已在模块注释中
声明。全部 29 个类型的 `TypeSchemaDigest` 已冻结。

## 2. Workflow primitive universe（9）

```text
value_no:  1      2      3      4       5        6       7      8          9
          unit   null   bool   int64   decimal  string  bytes  timestamp  duration
```

`int32` / `uint32` / `uint64` / `float64` 为 machine-only。唯一闸门
`SemanticPrimitiveType::try_from_keyword` 对这四个返回
`unsupported_workflow_primitive`。Rust 侧以字边界检查断言
`semantic-core.afidl` 的**声明代码**不出现 `int32`/`uint64`/`float64`，且
`uint32` 仅用于 local id 与编号字段。

## 3. SemanticType DAG 模型

```text
SemanticTypeNode { id: SemanticTypeId, body: SemanticTypeNodeBody }
SemanticTypeNodeBody (variant) {
  1 primitive   : SemanticPrimitiveType
  2 schema      : TypeSchemaRef          // 叶子：digest-pinned 命名类型
  3 list        : SemanticTypeId         // 引用同文档其它节点
  4 map_string  : SemanticTypeId
  5 option      : SemanticTypeId
}
```

- 任意有限嵌套（`option<list<map<string,T>>>`）无需递归 AFIDL schema；
- validator：id > 0、唯一、引用必须存在、**DAG 必须无环**（`semantic_type_cycle`）；
- canonical ordering 按 id 升序；alias 在语义层展开为目标结构
  （`alias UserId = string` 的语义类型即 `primitive:string`）。

## 4. TypeSchemaRef exact pinning 模型

```text
DigestRefValue { algorithm, domain, hex }          // 唯一 AFIDL digest 表示
TypeSchemaRef  { specification_id, type_name, schema_digest: DigestRefValue }
```

验证顺序：digest 格式（`sha256` / `type-schema` / 64 小写 hex，否则
`invalid_digest_ref`）→ identity 语法 → identity 存在于显式 `TypeSchemaUniverse`
（否则 `unknown_type_schema`）→ **pinned digest == 实际 canonical digest**
（否则 `type_schema_digest_mismatch`）。无 latest-resolve：universe 是显式只读输入
（测试中由 AFIDL compile output 构建）。

## 5. ValueSlot 模型

```text
ValueSlot { id: ValueSlotId, type: SemanticTypeId }
```

immutable SSA 绑定（一 producer / 零或多 consumer）；本阶段只校验 shape、正
id、重复 id、类型有效性。**不**定义 `ValueOrigin` / producer 引用 / RunParameter /
ComputationOutput——留 Phase 3B dataflow。`value_slot` 表达式的 lowering 需要显式
binding（Phase 3A 无 producer），未提供则 `value_slot_not_bound`。

## 6. Expression closed opcode 集合（15）

```text
constant          value_slot
record_construct  variant_construct   enum_value
list_construct    map_construct
option_none       option_some
field_get
equals            not   and   or
if_then_else
```

明确不含：算术（add/subtract/multiply/divide/mod）、字符串拼接、regex、索引、
`match_variant`/`is_case`/`unwrap_case`，以及任何 fallible operation
（division by zero / index out of range / parse failure 均不可能发生）。

## 7. Expression type-check 规则

`infer_expression_type` 逐 case 实现：

| body | 规则 |
|---|---|
| `constant` | 由 case 推导 primitive 类型；bytes 走 canonical base64url 校验 |
| `value_slot` | slot 声明类型 |
| `record_construct` | pin 必须解析为 record（否则 `expected_record_schema`）；按 `field_no` 绑定；未知/重复/缺失/类型不符分别报错；要求恰好声明字段集 |
| `variant_construct` | pin 必须为 variant；`case_no` 存在；payload 类型匹配 |
| `enum_value` | pin 必须为 enum；`value_no` 存在 |
| `list_construct` | 所有元素的推导类型 == `element_type` |
| `map_construct` | key NFC 化、NFC 后重复拒绝、unsigned UTF-8 排序；value 类型匹配 |
| `option_none` | 结果 `option<inner_type>`（inner 必须显式） |
| `option_some` | 结果 `option<T>`（从 payload 推导，不另存） |
| `field_get` | 输入必须是 record schema；按 `field_no` 取字段类型 |
| `equals` | 两侧类型必须相等，且类型可比较（primitive 或 enum，否则 `type_not_comparable`） |
| `not`/`and`/`or` | 操作数必须 bool，结果 bool |
| `if_then_else` | condition 必须 bool；两分支类型必须相等 |
| 全部 | `derived_type == declared result_type`，否则 `expression_result_type_mismatch` |

## 8. ResourceSlot 模型

```text
ResourceSlot { id: ResourceSlotId, requirement_type: TypeSchemaRef }
```

逻辑需求声明，**不是**连接/凭据/活对象；独立 namespace（value slot 1 与 resource
slot 1 可共存）；requirement 用 `TypeSchemaRef`（contract schema）而非
`SemanticTypeId`（值类型）。未实现 `ResourceHandle`、binding snapshot、
`BindingSnapshot`。

## 9. 新增 Rule / ABI / Specification 统计

- **Rule Area**：新增 `SEMANTIC`（封闭表 6 个 area）；
- **规则**：新增 **24 条** `AF-SEMANTIC-0001..0024`，全部 must 级；规则总数
  **47 → 71**；coverage 70 covered / 1 intentionally uncovered（仍是
  `AF-ACE1-0001`）/ 0 missing；
- **ABI 行**：50 → **132**（+82：semantic-core 的 22 个 enum value + 60 个 field）；
  全部 active，0 retired，0 reserved。行由生成器从 AFIDL 源**自动派生**（消除手抄）；
- **Specification**：10 → **12**（新增 `agentflow.semantic.core@1`（kind `schema`）与
  `agentflow.semantic.types@1`（kind `schema_language`））；AFIDL 模块 2 → 3；
- **Suites**：1 → 7（新增 6 个 semantic suite，承载不可由 fixture 表达的
  MUST 规则：purity、closed expression set、namespace separation、
  no-new-domain、resource immutability、no runtime handle）。

## 10. Golden / Negative corpus 数量

`spec/conformance/semantic-core/`（revision 1，由独立 Python 实现生成）：

- **34 golden cases**：9 个 primitive constant、nested list/map/option、
  schema-ref、value-slot、record/variant(×2)/enum construct、list/map(×3)、
  option none/some、field_get(×2)、equals(×2)、not/and/or、if_then_else(×2)、
  canonical ordering(×4)；
- **55 negative cases**：machine-only primitive(×4)、schema ref(×5)、type DAG(×6)、
  value slot(×4)、expression(×8)、bool/equals(×5)、if(×2)、record(×6)、
  variant(×3)、enum(×2)、field_get(×2)、list/map(×3)、option_some 推导(×1)、
  resource(×3)、constant payload(×1)；
- **2 等价组**：`canonical/expressions-sorted`↔`shuffled`、
  `canonical/types-sorted`↔`shuffled`（source order 不改变字节）。

每条 golden 冻结 `derived_type` + `ace1_utf8` + `ace1_hex`；每条 negative 冻结
`error_kind` + 拒绝层（`check` / `deserialize`）。Rust 测试只消费冻结值。

## 11. TypeSchemaDigest 新增数量

**29 个**（`semantic-core` case，AFIDL corpus **revision 2 → 3**，纯 additive）。

**旧 digest 零漂移经机器验证**：从 `HEAD` 版本读取旧 golden，与重新生成的结果
逐键比对——84 个旧 `(case_id, type_name)` digest 全部 `unchanged`，仅新增 29 个。
Phase 1/1A ACE1 与 digest corpus、`spec/tables/hash-domains.json` 均无 diff。

## 12. spec-check 结果

```text
specifications: 12          （+2）
normative rules: 71         （+24）
rules covered: 70
rules intentionally uncovered: 1
rules missing coverage: 0
ABI assignments: active 132 / retired 0 / reserved 0
AFIDL modules checked: 3    （+1）
cross-reference errors: 0
```

## 13. workspace 测试 / 门禁结果

- `cargo fmt --all -- --check`：通过；
- `cargo clippy --all-targets --all-features -- -D warnings`：通过；
- `cargo test --all`：**136 通过 / 0 失败**（af-core 37、af-idl 33、af-spec 22、
  af-semantic 44）；
- `spec-check`：exit 0（见上）；
- **5 个生成器全部幂等**（二次运行零 diff）：`gen-conformance`、
  `gen-afidl-corpus`、`gen-spec-registry-corpus`、`gen-registry-instances`、
  `gen-semantic-core-corpus`。

## 14. 静态自检（§一百一十七 全部满足）

由 `crates/af-semantic/tests/static_surface.rs` 强制：

```text
no Any / serde_json in the semantic model        ✅ （仅 dev-dependency 用于测试）
no Workflow opcode (13 nodes)                    ✅
no Journal / RuntimeCommand / ResourceHandle     ✅
no clock / random / environment                  ✅ （Cargo 依赖 + 源码双重断言）
no network                                       ✅
no Registry latest lookup                        ✅ （显式 universe，无 af-spec 依赖）
no mutable workflow variable                     ✅ （无 mutation 操作面）
no ResourceHandle runtime object                 ✅
no new HashV1 domain                             ✅ （域表仍 14 项；无名 14 域之外的域）
```

## 15. 显式 deferred 项

见 `docs/implementation/phase-03a-decisions.md`（D-1..D-28）。要点：

- `ValueOrigin` / `ValueProducerRef` / RunParameter / ComputationOutput 绑定（Phase 3B）；
- 表达式算术、字符串操作、索引、`match_variant` 系列（按 Semantic IR 实际需求扩展）；
- record/list/map/option 的深相等；
- `SemanticIRDocument`（含 body/control/dataflow）与 document digest；
- `SchemaModuleDigest`（沿用 Phase 2A D-5）；
- **`semantic_spec` 作为 SpecificationKind 新成员**：为避免 Phase 2B 两个已有
  digest 漂移（§一百零四 硬约束）而暂缓；`agentflow.semantic.types@1` 现登记为
  `schema_language`。这需要未来一个允许 additive registry-kind 演进的阶段；
- JSON 实例对 AFIDL schema 的 ACE1 re-encode 验证（Phase 2B 延续）。

## 16. 阻塞 Phase 3B 的问题

**无阻塞**。Phase 3B 可直接消费：9 原语闭集、类型 DAG（含无环性与 id 规则）、
exact pinning、slot identity、15 项纯表达式（含类型检查器）、resource slot namespace
与 canonical ordering。13 个 Computation Node 只需在这些底座上增加节点类型与
dataflow 边（producer/consumer 一致性），不需要修改本阶段的任何冻结字节。
