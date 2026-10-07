# Phase 2A Implementation Report — AFIDL1 Core Schema Language / Parser / Type Schema Identity

> 完成日期：2026-10-07。本报告只记录：实现了什么、对应冻结规范、代码路径、
> corpus 路径、验证命令、显式决定、未实现内容。不进入 Semantic IR。

## 1. 实现了什么

AFIDL1 从"文档中冻结的 schema 概念"成为真正可解析、验证、canonicalize、
计算 TypeSchemaDigest 的机器 Schema Language：

```text
AFIDL1 exact grammar (spec/idl/afidl1-grammar.md)
        ↓ hand-written lexer + recursive-descent parser（SourceSpan）
        ↓ typed source AST（crates/af-idl）
        ↓ fail-closed validation
        ↓ Canonical TypeSchema machine model
        ↓ ACE1（af_core::encode，冻结 wire grammar）
        ↓ HashV1("type-schema", bytes)
        ↓ TypeSchemaDigest
```

正式冻结的构造：`primitive`（13 个 closed builtin）、`ref`（NamedRef →
full logical identity）、`list<T>`、`map<string,T>`、`option<T>`（完整继承
Phase 1A：`option<null>`/`option<unit>`/`option<option<T>>` 全合法）、
`record`、`variant`、`enum`（含显式 `value_no`）、`alias`（nominal identity
≠ target identity，cycle 拒绝），以及 `field_no` / `case_no` / `value_no`
（u32、>0、唯一、canonical 按编号升序）。

## 2. 对应冻结规范

- Phase 1/1A：ACE1 wire grammar（`spec/encoding/ace1-grammar.md`）、HashV1、
  `type-schema` digest domain、machine primitive 集合、Option tagged 编码。
- 本阶段语法与 identity 模型按任务 §八/§五十一/§五十二 推荐方案冻结：
  `afidl 1;` + `schema <ns>@<major>;` + 类型无 `@N`。

## 3. 代码路径

```text
crates/af-idl/
  Cargo.toml           # 依赖仅 af-core（path）；dev: serde/serde_json/hex
  src/lib.rs           # compile() 主入口 + canonicalize + 单测
  src/span.rs          # SourceSpan（诊断专用，不进 digest）
  src/error.rs         # AfidlError（21 个结构化变体 + 稳定 kind()）
  src/specification.rs # SpecificationId 严格解析/规范显示
  src/primitives.rs    # Primitive 封闭枚举（13）
  src/lexer.rs         # ASCII lexer，注释/空白跳过，kebab 支持
  src/ast.rs           # source AST（span 化，与 canonical 分离）
  src/parser.rs        # recursive descent，fail-closed
  src/validate.rs      # dup/ref/alias-cycle/recursion/empty 检查
  src/canonical.rs     # CanonicalTypeSchema + ACE1 编码 + TypeSchemaDigest
  tests/conformance_afidl.rs  # 冻结 corpus 消费者 + drift 检查
tools/gen-afidl-corpus.py    # 独立 Python 实现，冻结 golden
```

`af-core` 本阶段**零改动**（Phase 1/1A corpus 全量回归通过）。

## 4. Spec 制品与 Corpus 路径

```text
spec/idl/afidl1-grammar.md               # AFIDL1 exact grammar（normative source）
spec/idl/core/machine-primitives.afidl   # 正式语法重写；provisional 标记已删除
spec/tables/afidl-primitives.json        # closed primitive 表
spec/conformance/afidl/parser-valid/     # 29 个有效源文件（含 10 个 mutation 文件）
spec/conformance/afidl/parser-invalid/   # 33 个无效源文件 + manifest.json（冻结 error kind）
spec/conformance/afidl/schema-golden.json          # 30 cases 的冻结 canonical ACE1 bytes
spec/conformance/afidl/type-schema-digest-golden.json  # 64 个冻结 digest + 10 个 mutation 断言
```

Golden 由**独立 Python 实现**（lexer/parser/canonicalizer/ACE1 encoder/
hashlib）生成并物理冻结；Rust 测试只消费冻结值，运行时从不生成 expected。
machine-primitives.afidl 本身入 corpus（case_id `machine-primitives`），其
13 个 alias 的 TypeSchemaDigest 已冻结。

## 5. 身份与编号模型（摘要）

- 三层身份：`afidl 1`（语言）→ `agentflow.example@1`（SpecificationId =
  namespace @ semantic major）→ 类型本地名；完整 identity =
  `<SpecificationId>/<TypeName>`；文件路径不参与 identity。
- `field_no`/`case_no`/`value_no`：显式源码编号，u32、`1..=UINT32_MAX`、
  声明内唯一（编号与名字独立唯一）；canonical 成员顺序 = 编号升序。
  编号是 durable ABI：改编号/名/类型改 digest；源码重排（编号绑定不变）
  不改 digest（golden equivalence/mutation 双向证明）。
- 枚举编号：冻结文档只写 field_no/case_no；enum 采纳 `value_no`（显式
  Decision D-1，理由见 decisions 文档），不替代 ACE1 值表示。
- 重编号绑定不同 meaning 的"编号不复用"原则已写入 grammar 文档 §4；
  跨版本强制检查留待 Compatibility 阶段（本阶段只做单编译单元唯一性）。

## 6. TypeSchemaDigest 公式

```text
TypeSchemaDigest = SHA256("AF\0" || "type-schema" || "\0" || ACE1(CanonicalTypeSchema))
```

Canonical model 与 ACE1 的对应关系（schema-of-schemas）见 grammar 文档 §6。
递归 TypeExpr 通过按值深度推导的有限 Shape 展开编码（D-7），ACE1 wire
grammar 零改动。SchemaModuleDigest 明确 deferred（D-5），未新增任何 hash
domain。

## 7. 验证命令及结果

```text
cargo fmt --all -- --check                            ✅
cargo clippy --all-targets --all-features -- -D warnings  ✅ 0 warning
cargo test --all                                      ✅ 64 tests
  （af-core 37：Phase 1/1A 全量回归 ✅；af-idl 27：21 单测 + 6 conformance）
python tools/gen-afidl-corpus.py                      ✅ 幂等（二次运行零 diff）
python tools/gen-conformance.py                       ✅ Phase 1 生成器幂等保持
```

静态自检（任务 §六十九）：AST 无 Any；canonical model 无 SourceSpan；
digest 不依赖源文件路径/comments/声明顺序/成员源顺序；编号非隐式 index
（负例 field_no=0 与 mutation 对证明）；hash domain 仍为冻结的
`type-schema`；ACE1 wire grammar 未修改；未进入 Semantic IR（无 Workflow/
Task/Computation/ValueRef/ResourceSlot/EffectSignature 等任何符号）。

## 8. 显式决定

全部记录于 `docs/implementation/phase-02a-decisions.md`（D-1..D-12），核心：
enum `value_no`（D-1）、类型级 @major 移除（D-2）、递归类型 Phase 2A scoped
restriction（D-3）、UnknownPrimitive 由 NamedRef 解析吸收（D-4）、
SchemaModuleDigest deferred（D-5）、canonical Shape 有限展开（D-7）。

## 9. 本阶段明确未实现的内容

- Semantic IR / Plan / Resolver / RunManifest / Journal / Reducer / Runtime
  Command / Worker / Effect —— 未触碰，无 placeholder；
- RuleId Registry / NormativeRuleDescriptor（Phase 2B）；
- SchemaModuleDigest（D-5，待 domain 决定）；
- 跨文件多文件 module 合并（grammar 允许，实现按单文件编译单元）；
- `afidl fmt`（任务 §六十三 允许暂缓）、debug pretty-printer；
- Code generation（Rust/TS/Protobuf/JSON Schema，后续 Projection 阶段）；
- Compatibility/upcaster engine（任务 §六十五 暂缓）。

## 10. Git 概览

见最终交付说明（分类提交：workspace/scaffolding、af-idl 源码、spec 制品、
corpus+生成器、测试、文档；af-core 未改动）。
