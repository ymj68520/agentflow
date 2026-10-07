# Phase 2B Implementation Report — Normative Registry / RuleId / Stable ABI Number Registry

> 状态：完成。本阶段只建立 AgentFlow 规范自身的稳定符号体系；未进入
> Semantic IR、Plan、Journal 或 Runtime。全部工作已通过验收清单。

## 1. Registry 架构

```text
spec/idl/specification-registry.afidl      authority（agentflow.spec.registry@1）
        │  Phase 2A compiler + 独立 Python 实现（双实现一致）
        ▼
per-type TypeSchemaDigest × 20             冻结于 AFIDL corpus（revision 2）
        │
        ▼
spec/registry/*.json（实例，canonical 字节形态）
  specifications.json   10 entries     kind/status closed enum + ArtifactFileDigest
  rules.json            47 rules       RuleId / area / anchor / refs / coverage
  abi-numbers.json      50 rows        field + enum_value, all active
  coverage-summary.json 1 exception    AF-ACE1-0001
spec/tables/rule-areas.json   5 areas   closed table
spec/conformance/suites.json  1 suite   SUITE-DIGEST-DOMAIN-SEPARATION
        │
        ▼
crates/af-spec（模型 + 交叉引用校验 + ABI 对照 + 覆盖率）
        │
        ▼
cargo run -p af-spec --bin spec-check -- check   （只读；错误 → 非零退出）
```

新 crate `af-spec` 依赖方向：`af-core ← af-idl ← af-spec`；无任何 runtime 语义。
模块布局：`model`（serde 模型）、`fs`（canonical JSON / sha256 / 路径安全 / 遍历）、
`anchors`（锚点 / provisional 标记 / RuleId / 标识符文法）、`tables`（封闭表 + drift）、
`abi`（编号提取 + `check_current_schema_against_abi_registry`）、`check`（编排）、
`bin/spec-check.rs`（CLI）。

## 2. 已登记的 SpecificationId（10，全部 frozen）

| SpecificationId | kind | normative source |
|---|---|---|
| `agentflow.encoding.ace1@1` | encoding | spec/encoding/ace1-grammar.md |
| `agentflow.digest.hash-v1@1` | digest | spec/digest/hash-v1.md（本阶段新物化） |
| `agentflow.digest.hash-domains@1` | digest_domain_registry | spec/tables/hash-domains.json |
| `agentflow.idl.afidl@1` | schema_language | spec/idl/afidl1-grammar.md |
| `agentflow.idl.type-schema@1` | schema_language | spec/idl/afidl1-grammar.md（§6–7） |
| `agentflow.core.machine-primitives@1` | schema | spec/idl/core/machine-primitives.afidl |
| `agentflow.idl.afidl-primitives@1` | closed_table | spec/tables/afidl-primitives.json |
| `agentflow.registry.rule-areas@1` | closed_table | spec/tables/rule-areas.json |
| `agentflow.registry.core@1` | registry | spec/registry/registry-spec.md（本阶段新物化） |
| `agentflow.spec.registry@1` | schema | spec/idl/specification-registry.afidl |

已冻结 identity 优先：`agentflow.core.machine-primitives@1` 未重命名（其 13 个
alias digest 已冻结）。未登记任何未实现的 Semantic 模块。`schema_digest` 字段
统一暂缺（Phase 2A D-5 deferred + 本阶段禁止新增 HashV1 domain；drift 由
ArtifactFileDigest + corpus digest golden + ABI 一致性三层防护）。

## 3. RuleId 格式与 Rule Area 集合

- RuleId：`AF-<AREA>-<4DIGIT>`（D-1）；禁止文档节号派生；唯一且不复用。
- Rule Area（封闭表，5 个）：`ACE1 / AFIDL / DIGEST / REGISTRY / SPEC`。

## 4. Normative Rule 数量与覆盖情况

**47 条规则，全部 `must` 级**；覆盖 46 / intentionally uncovered 1 /
missing 0。每条规则绑定 md 锚点 + conformance refs（fixture case_id、
parser-invalid 文件 stem 或 suite id）。

| Area | 规则数 | 代表规则 |
|---|---|---|
| ACE1 | 14 | schema-directed、UTF-8 canonical bytes、numeric-as-string、string/map-key NFC、unsigned-byte key ordering、duplicate-key rejection、float64 exact/non-finite/-0、base64url、timestamp/duration µs、option tagging、closed enum/variant |
| DIGEST | 7 | exact prefix、domain separator、SHA-256 only、domain retained、lowercase hex、unknown domain rejected、domain separation |
| AFIDL | 15 | language major、SpecificationId syntax、primitive universe closed、field/case/value_no positive+unique、source order nonsemantic、number binding semantic、named-ref validation、alias cycles、recursion restriction、TypeSchemaDigest formula |
| SPEC | 3 | SpecificationId unique、normative source integrity、frozen source not provisional |
| REGISTRY | 8 | RuleId syntax/unique、anchor exists、conformance ref resolves、ABI number never reassigned、ABI registry matches schema、ABI assignment unique、registry canonical form |

唯一 intentionally uncovered：**AF-ACE1-0001**（schema-directed 编码——架构
不变量，数据驱动 fixture 不可证伪；理由冻结于 coverage-summary.json）。
锚点已注入 `ace1-grammar.md`（14）、`afidl1-grammar.md`（15）；新文档
`hash-v1.md` 与 `registry-spec.md` 原生携带锚点。

## 5. ABI Number Registry 结构

`AbiNumberAssignment { schema_family, type_name, number_kind, number,
symbolic_name, status, first_seen_schema_digest? }`；`number_kind ∈
{field, variant_case, enum_value}`、`status ∈ {active, reserved, retired}`。
单行 + 状态表达历史（D-12）；retired 编号永久绑定其 symbolic_name。

一致性检查（`check_current_schema_against_abi_registry`）：未登记编号拒绝、
retired/reserved 编号使用拒绝、编号→新名拒绝（abi_number_reassigned）、名→
新号拒绝（abi_name_rebound）、active 行无 schema 对应物拒绝；reserved 行豁免。

## 6. active / retired / reserved 统计

- **active: 50** —— `agentflow.spec.registry@1` 自举：5 个 enum 的 22 个
  `enum_value` + 6 个 record 的 28 个 `field`（机器基元模块只有 alias，无编号，
  不登记）。
- **retired: 0**、**reserved: 0** —— 本阶段无历史分配、无提前圈占。

## 7. Cross-reference validator 支持内容

- Specification Registry：SpecificationId 语法（复用 af-idl）/ 唯一性 /
  kind/status 封闭枚举 / 源文件存在 + 路径安全（`..`、反斜杠、绝对路径拒绝）/
  ArtifactFileDigest 重算比对 / frozen 源无 provisional 标记。
- Rule Registry：RuleId 语法与 area 双重校验 / 唯一 / requirement level 封闭 /
  源归属（必须命中某个已登记 specification 的 source）/ 锚点存在（md 与
  afidl 双文法）/ (path, anchor) 唯一 / conformance refs 可解析 / 覆盖三分类。
- ABI Registry：行格式与标识符文法 / tuple 唯一 / symbolic name scope 唯一 /
  canonical 排序 / 全部 `spec/idl/**/*.afidl` 编译后逐 family 对照。
- Closed tables：hash-domains ↔ `DigestDomain::ALL`、afidl-primitives ↔
  `Primitive::ALL` 逐位 drift 检查；rule-areas 排序/重复/文法。
- Registry 文件字节形态：sorted-key pretty printer round-trip 逐字节验证。

## 8. 新增 Registry AFIDL schema 与 TypeSchemaDigest 数量

`spec/idl/specification-registry.afidl`（`agentflow.spec.registry@1`）：
**20 个类型**（9 alias + 5 enum + 6 record），产生 **20 个新 TypeSchemaDigest**，
由独立 Python 实现冻结于 AFIDL corpus revision 2（additive：旧 case 与旧
digest 逐字节不变，diff 仅 2 行 revision）。无新增 HashV1 domain。

## 9. Conformance / negative corpus 数量

`spec/conformance/spec-registry/`：**18 案例**（2 valid + 16 invalid），每个
invalid 案例是自洽迷你注册表上的单变异，覆盖 §六十二 全部清单：duplicate
SpecificationId、bad/duplicate RuleId、missing source/anchor、unknown
RuleArea、unknown ConformanceCaseId、duplicate ABI assignment、retired number
reuse / in use、name rebound、unregistered assignment、active assignment
missing、canonical order violation、provisional frozen source、content digest
mismatch、valid retired number。Rust runner 断言 outcome + 稳定 error kind。

## 10. spec-check 结果

```text
$ cargo run -p af-spec --bin spec-check -- check   （exit 0）
specifications: 10
normative rules: 47
rules covered: 46
rules intentionally uncovered: 1
rules missing coverage: 0
ABI assignments:
  active: 50
  retired: 0
  reserved: 0
AFIDL modules checked: 2
cross-reference errors: 0
```

## 11. 全工作区门禁结果

- `cargo fmt --all -- --check`：通过；
- `cargo clippy --all-targets --all-features -- -D warnings`：通过；
- `cargo test --all`：**86 通过 / 0 失败**（af-core 37、af-idl 27、af-spec 22）；
  Phase 1/1A ACE1 corpus（revision 2）与 Phase 2A AFIDL corpus 全部逐字节通过；
  `git diff` 确认 hash-domains.json 与既有 golden 零漂移；
- spec-check：exit 0（见上）；
- 四个生成器幂等：`gen-conformance.py`、`gen-afidl-corpus.py`（revision 2）、
  `gen-spec-registry-corpus.py`、`gen-registry-instances.py` 二次运行零 diff。

## 12. 决定与遗留

设计决定见 `docs/implementation/phase-02b-decisions.md`（D-1..D-17）。遗留
（均为显式 deferred，非阻塞）：

- SchemaModuleDigest（新 domain 或复用 `type-schema` 的显式决定，Phase 2A
  D-5 延续）；
- JSON 实例对 AFIDL canonical schema 的 ACE1 re-encode 验证（§五十二 允许
  后置；当前靠 Rust typed model + 逐字节 canonical form 检查）；
- JSON Schema / Protobuf projection、SDK codegen、full compatibility engine、
  spec release manifest —— 各自属于后续阶段；
- 规则库 additive 演进：SPEC/REGISTRY 领域后续随规范生长继续分配 RuleId。

## 13. git diff / commits 概览

分类提交（自 `10fbe97` 起）：build（af-spec crate）、feat（registry 模型与
校验器）、spec（registry 制品与锚点）、test（corpus revision 2 + spec-registry
corpus）、docs（本报告 + decisions）。工作树干净，未推送。
