# 来源准入与信任边界（草案，未注册）

真实来源声明由提交者提供；Compiler 负责结构、域、实际选择和完整内容一致性。
声明键的存在证明“提交者在此来源域中声明了这个 Owner/slot”，不证明该提交者
拥有某个组织、仓库、文件或远程系统。Declared Provenance ≠ Authenticated Provenance。

未来 admission 的确定顺序，所有失败先于 Raw Predicate/Fact：

1. 校验精确已支持新版本元组及全部封闭 schema；未知 Owner/schema Variant 拒绝。
2. 要求实际 SourcePackage、非空合法 Domain/Owner/revision，缺少任何一个返回
   MissingDocumentOccurrenceBinding。只有 D/D 的两份对象也走此路径。
3. 对输入包内全部 OwnerRecord 检查域、唯一 Anchor、完整容器、无冲突重复来源。
   不仅验证请求的那个条目后跳过包内其余 Owner 冲突。
4. 检查所指定 Owner、OccurrenceKey 和实际容器选择；缺 key 返回 Missing…，
   错域、未知 Owner、越界或错误容器返回 InvalidOccurrenceAssociation。
5. 校验完整实际文档与声明 slot 相等；联合来源中相同 A/key 不同内容返回
   BindingConflict。不同 kind/完整容器复用同 A 为 OwnerBindingConflict。
6. 执行冻结 physical/native 源解析/结构/entry/candidate 准入；路径错误不得修复。
7. 检查真实 Compiler、同一九成员 Context 对象以及既定角色分区；对象身份不能
   由内容相等推出。Portable 未来完整来源证明仍需另行实现 G02，不用布尔值代替。
8. 只有真实 Producer 在分支前取得的结构化证据才可能形成非权威 Raw Seed。
   来源准入本身仅得来源关联，不得创建 Proven、Witness 或 Root Checked。

草案失败名不是已注册 enum/rank：MissingDocumentOccurrenceBinding、
InvalidOccurrenceAssociation、BindingConflict、OwnerBindingConflict、
MalformedSourcePackage、UnsupportedOwnerVariant、UnsupportedOwnerSchema、
UnsupportedPhysicalAssociation、InvalidPhysicalInput、InvalidContextBinding、UnsupportedModel、ResourceLimit。
未来注册须精确 materialize，不能把任意 String 错误当开放 proof Variant。
失败不回落到 content-only Source，不返回完整空 Capture，也不改写正式 Verdict。

上述Missing…是来源准入的语义失败类别。若Portable原始内容根本缺少构造完整
typed binding所需的字段，closed codec可以先报IncompleteFacts（缺occurrence
binding），同样拒绝；不能为了返回某个名字而先填一个假的完整typed值。

OwnerBindingConflict 是 BindingConflict 失败族中的闭合 owner_record 情形；
document_entry 情形是同Owner/key的实际文档冲突。若包本身已有Owner冲突，先拒绝
该冲突而不继续选择条目。它们不是将来任意String扩展的开放错误域。

未来Native接口须从实际SourcePackage借用所选slot，形成内部受控的借用关联；
Producer只消费这份真实选择。另一个Owner中相等值的文档不能凭Eq取得原选择的
历史来源。缺少这种本地来源关联必须拒绝，不把caller bool当证据。借用地址检查
仅在进程内确认关联，绝不写入Portable身份。Portable端重建输入包并选择条目，
验证的是本次显式来源输入；无法单靠Portable字节证明发送端某个旧内存对象当年
来自该Owner。需要历史认证的主张仍未建立。这里只设计该受控边界，不实施接口。

## 明确不验证的主张

不验证外部 publisher 身份、历史文件创建时间、组织所有权、网络服务签名、
全球唯一性、预算/运行时权限。namespace 即使写成受信任组织名称也只是一项声明。
任何试图从 owner 字符串/声明标签取得 Authenticated/Established 的请求都拒绝。
需要外部认证的边界仍 NotEstablished，现有六个 Root 独立边界不能被此输入消除。

生成地址/随机 UUID/时钟/调用计数/临时 wrapper 来“补齐”缺失来源均禁止。
一个持久系统早已分配的标识与本次临时生成的标识不同，但仍必须有已登记的
持久来源适配和真实关联证据；本设计初始版本不支持该适配。

推荐未来显式有界 admission：key每项≤4096原始UTF8字节、Owner≤4096、
包内文档条目总数≤262144；总输入/深度/work沿原系统有界接口计费，超限
ResourceLimit。它们是待注册的新输入限制，不修改现行8MiB physical、96步路径、
默认1MiB scalar或32MiB input限制；大型合法输入可明确安全拒绝，不默默放宽默认值。
