# Owner/Occurrence 的完整声明域

下列是一个确定的、有限、封闭的设计 schema；所有名字前缀 Draft::，尚未注册。
不声称现有 Import/Workflow 文档已经实现此格式，也不涉及 WorkflowVersion。

```text
KeyText       = exact UTF8 bytes, 1..4096 bytes; no NFC repair or case folding
Domain        = {namespace:KeyText, domain_key:KeyText}
OwnerAnchor   = {domain:Domain, owner_key:KeyText, revision_key:KeyText}
BundleEntry   = {declaration_name:KeyText, document:Node<RawDocument>}
ImportEntry   = {import_name:KeyText, document:Node<RawDocument>}
WorkflowEntry = {source_name:KeyText, document:Node<RawDocument>}
OwnerContainer =
  declaration_bundle {bundle_name:KeyText, declarations:Seq<BundleEntry>}
| import_manifest {manifest_name:KeyText, imports:Seq<ImportEntry>}
| workflow_source_collection {collection_name:KeyText, sources:Seq<WorkflowEntry>}
OwnerRecord = {anchor:OwnerAnchor, container:OwnerContainer}
SourcePackage = {domain:Domain, owners:Seq<OwnerRecord>}
OccurrenceKey = {entry_ordinal:U64}
DocumentOccurrenceBinding = {
  source_package:Node<SourcePackage>,
  source_owner:OwnerAnchor,
  occurrence_key:OccurrenceKey,
  actual_document:Node<RawDocument>
}
Assurance = declared_only       // 不是 caller 可选择的 authenticated 布尔值
```

namespace/domain_key 是实际提交来源声明域的显式键，例如用户提交的来源包标识；
owner_key/revision_key 是该包在验证前已有的 Owner/版本声明。Compiler 要求调用者
实际提交完整 SourcePackage。它不是按候选对象或每次调用自动组装的包装。
没有来源域、Owner 或键时不能从内容猜出一个；也不提供默认 namespace。

SourcePackage 在这里本身就是实际声明输入。不能把一个更大第三方 manifest 截去
未知字段后声称这个子集是其完整来源；格式不匹配应 UnsupportedOwnerSchema，
或显式提交一个新的声明域，仅表达本次新声明、不表达原文件的历史认证来源。
三种封闭容器完整保留各自以上全部字段，不接受任意 JSON/opaque payload 扩展。

OccurrenceKey 的 ordinal 从该 Owner 容器真实声明序列读取，分别指向
declarations/imports/sources。不是验证顺序、Report 下标或本地分配数；它与文档
内部 entry0 是两层不同坐标。容器顺序冻结于 Owner 的不可变 revision。
新版本调整容器或文档需实际的新 revision 声明，不能暗改旧 Owner。

## 关联与冲突

1. 包内 OwnerAnchor.domain 必须等于 SourcePackage.domain。
2. owners 中 Anchor 唯一；重复 Owner 行即 MalformedSourcePackage。对不同输入包
   做联合审查时，相同 Anchor 的完整 OwnerRecord 必须相等，否则 OwnerBindingConflict。
3. 按完整 Anchor 找到唯一真实 Owner，按 U64 ordinal 选择该类容器真实文档条目。
   未找到或越界是 InvalidOccurrenceAssociation。
4. binding.actual_document 必须与所选条目完整 nominal typed value 相等。
   同 A/ordinal 绑定两个不同 D 必须 BindingConflict，不能任选一个或生成新身份。
5. 同 A/ordinal/D 的重复使用引用同一个来源；无需在 owners 中重复声明 Owner。
6. 本地 admission 还须检查所用原生文档来自这份实际已提交容器的所选条目；
   不能仅凭外部另一同值对象的孤立 Eq 断言证明其历史来源。新的显式提交关联可以
   验证，但只能说明本次声明，不能倒推旧调用当年属于哪个 slot。

包内、合并包或重放审查的冲突可由完整输入检查；无输入历史时不能宣称全球从未
复用同一个键。独立进程收到同一完整包会得出相同结果，收到相冲突包的联合审查
会拒绝。此设计不引入外部全局登记服务、Resolver 或持久化状态实现。

## 四种真实来源的裁决

| 来源 | 谁提供 / 检查什么 | 本设计结果 |
|---|---|---|
| Actual Declaration Bundle | 调用者提交完整上述 bundle、声明域和 Owner；检查真实条目与内容 | 可准入，DeclaredOnly |
| Actual Import Manifest | 调用者提交完整上述 manifest；不凭 import_name 进行网络解析 | 可准入，DeclaredOnly |
| Actual Workflow Source Collection | 调用者提交完整上述 collection；不假造运行时 workflow root | 可准入，DeclaredOnly |
| Externally Established Durable Source Handle | 实际持久系统的 namespace/key/immutable revision，加系统实际返回内容及完整关联证明才可能成立 | 不在初始闭合 Variant 中，UnsupportedOwnerVariant；不能把任意 handle 字符串当证据 |

Durable handle 方案不是被改名为 OwnerKey 就自动受信。将来若需要支持，必须另行
定义有限 provider/receipt schema、精确验证策略、外部信任根和重放/版本绑定，
完成新的正式授权；本次没有定义通用认证回调、任意 verifier 或 publisher capability。
