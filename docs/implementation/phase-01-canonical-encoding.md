# Phase 1 Implementation Report — Canonical Encoding / Digest Foundation

> 完成日期：2026-10-07。本报告只记录：实现了什么、对应冻结规范、代码路径、
> corpus 路径、验证命令、未解决问题、明确未实现内容。不引入下一阶段架构。

## 1. 实现了什么

AgentFlow v1 最底层的机器确定性基础库（Rust，Cargo workspace，Edition 2024，
单一 foundational crate `af-core`）：

1. **ACE1 typed canonical encoder** — deterministic、schema-directed、UTF-8
   JSON subset。输入是封闭的 typed machine value 模型（`Value`）+ 显式
   `Shape`，输出 byte-for-byte 确定的 `Vec<u8>`。覆盖全部 13 个 AFIDL machine
   primitives（`unit/null/bool/int32/uint32/int64/uint64/decimal/float64/
   string/bytes/timestamp/duration`）与容器（`record/variant/enum/list<T>/
   map<string,T>/option<T>`）。
2. **ACE1 strict canonical decoder** — canonical bytes → typed value；
   `decode(encode(v)) == v` 且 `encode(decode(canonical)) == canonical`；
   一切 valid-looking 但非规范的输入（空白、未排序/重复 key、非 NFC、非规范
   转义、裸 JSON number、padded base64url、`-0` float bits、非规范 decimal
   文本、trailing bytes、unknown/missing field、unknown enum/variant）一律
   **拒绝而非修复**。
3. **Primitive validators/parsers** — 严格 canonical 整数文本解析器（int32/
   uint32/int64/uint64，精确范围、禁 `+`/前导零/`-0`、全程 i128 整数运算、
   不经过 IEEE-754）。
4. **HashV1** — `SHA256("AF\0" || domain || "\0" || payload)`，14 个冻结
   digest domain 的封闭 registry。
5. **DigestRef** — nominal typed digest identity（alg=sha256 / domain /
   64 位小写 hex），构造即校验，严格拒绝大写 hex、错误长度、非 hex、未知
   算法/域。
6. **Golden / Negative Conformance Corpus** — 由独立 Python 实现
   （`tools/gen-conformance.py`）一次性生成并物理冻结在 `spec/conformance/`；
   Rust conformance tests 只消费冻结值，从不在运行时生成 expected。

## 2. 对应冻结规范

- `docs/`（三份内容相同的冻结文档，以 §24.1 Canonical Encoding / AFIDL v1、
  §22.3 Identity/Digest/Serialization、§22.13 Type Contract 为实现依据；
  §22.21/§25.8 将 "ACE1 exact byte grammar" 列为待物化硬门槛，本阶段予以
  物化）。
- 结构性落地区别（unit/null 字节形式、Option::None 编码、decimal 规则等）
  全部记录在 `docs/implementation/phase-01-open-questions.md`（OQ-1..OQ-13），
  未擅自当作已冻结规范。

## 3. 代码路径

```text
Cargo.toml                                # workspace（resolver 3, lints: forbid unsafe, clippy all+pedantic）
crates/af-core/
  Cargo.toml                              # deps: sha2, unicode-normalization, base64（运行时仅此三个）
  src/lib.rs                              # 文档 + re-exports
  src/error.rs                            # AceError / DigestError + 稳定 kind() 类别
  src/shape.rs                            # Shape/Record/Variant/Enum + validate（NFC/唯一性/option 嵌套拒绝）
  src/value.rs                            # 封闭 Value 模型 + Float64/Timestamp/Duration nominal 包装
  src/decimal.rs                          # 精确 decimal 解析与规范化（字符串承载，无 bignum/无 float）
  src/base64url.rs                        # unpadded base64url 编码 + 严格 canonical 解码
  src/encode.rs                           # ACE1 encoder（排序/转义/类型门禁）
  src/decode.rs                           # ACE1 strict decoder（canonical-only）
  src/primitive.rs                        # 公开 canonical 整数文本解析器
  src/digest.rs                           # HashV1 + DigestDomain registry + DigestRef
  tests/conformance_encoding.rs           # golden/negative corpus 消费者（含 determinism、round-trip、等价断言）
  tests/conformance_digest.rs             # hash golden / domain separation / registry drift 检查
  tests/distinctions.rs                   # missing/null/unit/Option 五态不折叠的显式证明
tools/gen-conformance.py                  # corpus 一次性独立生成器（Python hashlib/unicodedata/base64/struct）
```

## 4. Spec 制品与 Golden corpus 路径

```text
spec/encoding/ace1-grammar.md             # ACE1 精确字节文法（OQ 物化）
spec/idl/core/machine-primitives.afidl    # machine primitive 集合物化
spec/tables/hash-domains.json             # 14 个冻结 digest domain registry
spec/conformance/encoding/primitive-golden.json    # 74 cases
spec/conformance/encoding/string-golden.json       # 19 cases（NFC 等价对）
spec/conformance/encoding/collection-golden.json   # 33 cases（排序/等价输入）
spec/conformance/encoding/negative.json            # 67 cases（encoder 39 + decoder 28）
spec/conformance/digest/hash-v1-golden.json        # 17 cases
spec/conformance/digest/domain-separation.json     # 14 domains × 1 payload
spec/conformance/digest/negative.json              # 7 cases
```

Golden 总量：**126 golden + 105 negative/domain/digest = 231 条冻结制品**。
全部 expected 值由独立 Python 实现生成后冻结；开发过程中双实现交叉验证实际
抓到并修复过一处分歧（Python 生成器 decimal 输入规范化缺失）。

## 5. 执行过的验证命令及结果

```text
cargo fmt --all -- --check                          # 通过（clean）
cargo clippy --all-targets --all-features -- -D warnings   # 通过（0 warning/error）
cargo test --all                                    # 通过：16 lib + 7 digest + 4 encoding + 7 distinctions = 34 tests
python tools/gen-conformance.py                     # 可复现冻结 corpus（幂等）
```

测试内部额外验证（任务 §29 自检映射）：

- **Determinism**：每个 golden 文件的全部 case 编码两次，逐 case 断言
  byte-identical（`run_golden_file`）。
- **Map ordering**：`map/unsorted-input` 与 `map/sorted-input-equivalent`
  两条 golden 的冻结字节相同；实现中无 HashMap/HashSet（grep 证实），为
  ordered Vec + 显式 unsigned-byte sort。
- **Unicode**：分解/组合输入 golden 等价对（é、Hangul U+AC01）；NFC 等价
  重复 map key 在 encoder 拒绝、非 NFC 内容在 decoder 拒绝。
- **Numeric precision**：`int64/beyond-float53`（2^53+1）golden；整数解析
  全程 i128；`uint64/max`、`int64/min` golden。
- **Float bits**：12 条 bit-pattern golden（含 `-0`→`+0`、subnormal、
  max-finite）；NaN/±Inf 在 encoder 与 decoder 双侧拒绝。
- **Domain separation**：同一 payload × 14 domain 冻结 digest 两两不同 +
  属性测试。
- **No Any escape hatch**：grep 证实 `src/` 无 serde_json/Any/Box<dyn>；
  fixture harness 中的 serde_json 仅存在于 tests（工具链，非 normative model）。
- **No runtime creep**：`src/` 恰为 10 个基础模块（见 §3），无 IR/Plan/
  Journal/Reducer/Command 等任何后续阶段模块。

## 6. 未解决的规范问题

见 `docs/implementation/phase-01-open-questions.md`（OQ-1..OQ-13）。核心三项：

- **OQ-1**：ACE1 exact byte grammar 是本阶段按冻结约束物化的，需在正式
  grammar freeze 时批准；
- **OQ-2/OQ-3**：unit/null 字节形式与 Option::None 编码（显式 `null` vs
  字段缺省）是文档未唯一确定的行为，已按可辩护方案落地并记录备选；
- **OQ-10**：§22.3 与 §24.1 的 domain 字符串风格冲突，按"后者优先"采用
  §24.1。

均不阻塞本阶段基础实现；修改面已被限制在单一模块 + corpus。

## 7. 本阶段明确未实现的内容

- Semantic IR、Executable Plan、Workflow Compiler、Resolver/LockGraph、
  RunManifest、Journal、Reducer/Interpreter、SemanticExecutionState、
  Runtime Commands、Worker/Scheduler、Effect Gateway、Policy、Identity、
  Audit、Checkpoint、Visual Editor、Agent Loop、LLM/MCP/A2A、HTTP Server、
  数据库持久化 —— 全部未开始，无 placeholder；
- RunId/ComputationId/InvocationId/AttemptId/CommandId/EventKey 等 derived
  ID 体系（属后续阶段上下文规则）；
- AFIDL parser/compiler（任务明确不要求；`.afidl` 制品与实现间有 drift 检查）;
- permissive input parser（canonicalization 与 permissive 解析是不同概念，
  本阶段只交付 canonical boundary 两侧：typed encoder + strict decoder）；
- 跨语言 ACE1 交叉验证的自动化 CI（Python 生成器已构成事实上的第二实现；
  双语言 conformance runner 留待 corpus/CI 阶段）。

## 8. Git diff 概览

见最终交付说明（新增：workspace 清单、`crates/af-core/` 全部源码与测试、
`spec/` 五个规范制品 + 七个 corpus 文件、`tools/gen-conformance.py`、
`docs/implementation/` 两份文档；未触碰 `docs/` 三份冻结文档）。

---

## 9. Phase 1A Addendum — ACE1 Artifact Freeze / Open Questions Ratification

> 本节由 Phase 1A 追加。**Phase 1A 在 v1 artifact freeze 之前改变了 Option
> wire encoding**；因尚未发布正式 Spec Release，记为 ACE1 v1 pre-release
> artifact revision（corpus_revision 1 → 2），未升 Semantic Major。

### 9.1 Option wire encoding 变更（OQ-3 REJECTED_AND_REPLACED）

```text
Phase 1（已废弃）:            Phase 1A（正式批准）:
  None    → null                None    → {"none":null}
  Some(v) → <canonical-v>       Some(v) → {"some":<canonical-v>}
```

原因：null-sugar 编码无法区分 `None`/`Some(null)`/`Some(unit)`/`Some(None)`，
破坏 AFIDL schema-directed closed type system 的组合闭包。新编码为
single-case tagged object：exactly one key；key 必须为 `none`/`some`；
`none` payload 必须为字面 `null`；`some` payload 按内层 type 校验。

### 9.2 删除的非法类型限制（OQ-4）

删除 `Shape::validate` 中的三条人为限制：
`option<option<T>>`、`option<unit>`、`option<null>` 现全部合法且可区分
（`{"some":{"none":null}}` 等）。`Shape::validate` 对 Option 仅递归校验
内层 type。

### 9.3 Corpus 变更（corpus_revision 1 → 2）

- Python 独立生成器同步实现新 Option 规则（仍保持独立实现，未从 Rust 取值）；
- Golden：`126 → 136` 条（新增 10 条嵌套/边界 Option：option<string/null/unit>
  的 None 与 Some、option<option<string>> 的 None/Some(None)/Some(Some("x"))、
  option<option<null>> 的 Some(Some(null))），并冻结 §六 断言：
  `encode(None) != encode(Some(null)) != …`（Rust 测试与 Python 生成器双侧断言）；
- Negative：`67 → 76` 条（删除 1 条已失效的嵌套禁止负例，新增 9 条：`null`
  sugar 拒绝、`none` 非 null payload、空对象、未知 key、双成员（排序/未排序）、
  重复 key、`null`-into-option TypeMismatch、重复字段名 shape 负例）；
- **Cross-check（§十一）**：Python expected bytes == 冻结 fixture bytes ==
  Rust ACE1 bytes——由「独立生成器生成冻结值 + Rust conformance 只消费冻结值
  + 生成器内嵌属性断言」三方机制保证；
- digest 三个文件仅追加 `corpus_revision` 字段，期望值零变化（git diff 证明
  HashV1/DigestRef 行为无无意变化）。

### 9.4 OQ Register 重分类

原 OQ-1..OQ-13 已全部重分类（RATIFIED / RESOLVED_FROM_FROZEN_SPEC /
REJECTED_AND_REPLACED / DEFERRED_*），见
`phase-01-open-questions.md`（现为 Decision Register）。`ace1-grammar.md`
状态升级为 repository normative source；`machine-primitives.afidl` 顶部
保留 AFIDL syntax provisional 标记（OQ-11 → Phase 2）。

### 9.5 Phase 1A 验证

```text
cargo fmt --all -- --check                           ✅
cargo clippy --all-targets --all-features -- -D warnings  ✅
cargo test --all                                     ✅ 37 tests（16 lib + 7 digest + 4 encoding + 10 distinctions）
python tools/gen-conformance.py                      ✅ 幂等（二次运行 working tree 不变）
```

新增结构化错误类别：`AceError::InvalidOptionEncoding`（kind
`invalid_option_encoding`）。
