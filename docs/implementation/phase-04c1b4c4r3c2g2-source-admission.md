# SourcePackage / Owner / Slot / conflicts / actual admission

正式完整字段与闭合Variant均见model@5和document-occurrence.md。
SourcePackage={domain,owners:Seq<{anchor,container}>}。
Domain={namespace,domain_key}；Anchor={domain,owner_key,revision_key}。
四键均是1..4096字节的exact UTF8，不做NFC/大小写转换，不是全局认证键。
Container恰为declaration_bundle{bundle_name,declarations}、
import_manifest{manifest_name,imports}、workflow_source_collection{collection_name,sources}。
各entry完整保留实际声明名和完整RawDocument；不丢弃第三方未知字段后声称完整来源。

Occurrence=(完整Anchor,真实容器entry_ordinal:U64)。Owner版本中真实声明顺序固定，
不同真实slot或Owner可区分完整相等内容。同Owner/slot/content复用O，不因重新
分配对象变成新O。相同键不同内容BindingConflict；联合包相同Anchor不同完整
OwnerRecord为该族OwnerBindingConflict。单包重复Owner行MalformedSourcePackage。
检查整个实际包/显式联合包，不是全世界Owner数据库。

## 实现边界

occurrence_admission.rs接受完整SourcePackageMetadata、真实source_owner/slot、
actual_document借用、完整RawCandidate metadata、实际内部路径、physical关联、
完整Compiler及真实两个Context对象。全量结构、键、域、唯一性、文档、路径、
候选、物理矩阵及对象关联检查均先于任何Predicate。实际文档必须就是实际包内
所选条目的借用；另一个同值对象不能凭Eq冒充原Owner下的文档。

返回值只为unit或闭合错误域。没有受信任Source token、Proven、TypedFactSeed、
Scope字符串或公有Checked构造器。调用者在验证前明确提交包，本实现不按候选
补造单条目文档；没有包/Owner/revision/slot明确MissingDocumentOccurrenceBinding。
旧匿名native/physical源API在新Edition也不能取得完整新来源，返回缺occurrence。
原API在精确Analysis@5历史构建保持原样，未通过新代码重新解释。

本地借用指针只核验关联，不写入正式身份；跨进程可对完整显式输入重新选择，
不能认证发送端旧对象的历史来源。真正受控Native Producer仍未实现；当前API
是正式Raw类型的non-wire metadata表示，不是Contract wire decoder或已捕获事实。

## Physical / Native 初始允许矩阵

| origin | physical关联 | 结果 |
|---|---|---|
| physical_json_utf8完整原字节 | supplied同Owner/slot/同文档借用/同value_path | 可受检准入 |
| declared_native_document完整13字段序列 | not_supplied | 可受检准入，未知物理历史 |
| native | supplied | UnsupportedPhysicalAssociation |
| physical | not_supplied | UnsupportedPhysicalAssociation |

严格UTF8、重复JSON key、未知/缺少字段、错宽度和错候选拒绝；不NFC修复或重序列化
冒充原始字节。Native+supplied不伪造转换receipt。物理来源supplied混入另一O，即使
字节相同也拒绝。not_supplied不是空bytes。

Context九成员及analysis/canonical_validation真实同对象分区保持不变；内容相等
不证明别名，没有resolver_dependencies=[]或新的ContextRole来存Owner。
键和包只形成DeclaredOnly；额外authenticated/supported字段不能创造外部所有权。
DurableHandle/未知Owner Variant、未知结构化SourceDomain不支持；任意namespace
字符串并不因此成为受信任组织。默认Decoder/Compiler限制不放宽；显式受限测试
与默认D0 ResourceLimit分别记录。所有工作、深度、键长度、包大小均显式计费拒绝。
