# Normative amendment draft — NOT REGISTERED

以下DO-01..DO-14只是设计条款标签，不是RuleId/规范注册/版本分配。
未来注册必须给全部Draft类型完整封闭Descriptor、宽度、字段/Variant顺序、
ModelRef owner、完整来源与规则映射；本阶段不对未来Type Rank或Edition编号做分配。

**DO-01 / 四种身份。** MUST 区分完整内容、声明文档occurrence、该occurrence内
成员与输出观察；不得从内容相等推断occurrence相等或从重复观察推导新来源。

**DO-02 / 实际来源。** MUST要求实际完整SourcePackage、Owner域/版本声明和
真实条目选择；只有内存对象、匿名内容或每次调用重建的包装时MissingDocumentOccurrenceBinding。

**DO-03 / 闭合Owner。** MUST只接受声明bundle/import manifest/workflow collection
三个封闭容器及完整字段；其它owner/handle/schema Unsupported，不得String fallback。

**DO-04 / 声明域。** MUST检查namespace/domain/Owner/revision完整且实际来自
提交输入，不得自动创造；只建立DeclaredOnly，不验证publisher身份或全球唯一性。

**DO-05 / Owner一致。** MUST全量检查包内Owner Anchor唯一、域匹配及完整容器；
联合输入中同Anchor但不同完整OwnerRecord为OwnerBindingConflict；不可按查询次序任选。

**DO-06 / 条目与冲突。** MUST以真实容器声明序列U64 ordinal选择文档；相同
Owner/key/content重复使用同一来源，不同Owner或不同key可以区分同内容文档。
同Owner/key不同内容必须BindingConflict；记录历史不足不得宣称全球无冲突。

**DO-07 / 双层路径。** MUST同时保存外层Owner/slot与内层RawDocumentMember/
DeclaredStep路径，解析到同一实际原始成员，不能相互隐式cast或补造外层容器。

**DO-08 / Physical/Native。** MUST保存完整原物理字节或完整Native十三字段序列。
新profile准入矩阵为physical+supplied同binding、native+not_supplied；其余
UnsupportedPhysicalAssociation。禁止重序列化冒充历史字节或同值双来源转换证明。

**DO-09 / 完整传播。** MUST从RawInputOrigin.binding经Subject/Source及
RawValidationOrigin/Origin传播至FactKey/支持/证据/矛盾/fragment所有引用；
拒绝Source、Origin、Physical、Witness指向不同occurrence，即使内容完全相同。

**DO-10 / DAG与Scope。** MUST保持AF-SEMANTIC-0217完整typed equality和最小
可达DAG；相同内容共享节点，不同声明binding以不同真实typed值区分。输入包不
引用Fact/Report/Scope/自身binding，不存在生成结果回流或身份固定点。

**DO-11 / 复制与观察。** MUST仅对完整key及所有非观察payload相等的Fact union
合法观察；冲突InvalidFacts。观察/消费者不写入SourcePackage，不制造新O。

**DO-12 / 冻结语义。** MUST保持单个Raw Port Case、数学U32稳定排序、真实
original ordinal、Positive/Zero/Duplicate verdict与执行前缀；保持九Context同对象
分区，无Resolver/ContextRole替代Owner，无generic schema-validity新Profile。

**DO-13 / 版本与迁移。** MUST注册新的完整受影响身份闭包，不重用旧label/content；
旧来源仅在精确旧Edition解释，无新历史依据不自动升级。显式新提交/重新验证必须
标为新的来源建立过程，不声称恢复旧Origin。

**DO-14 / 权限与界限。** MUST拒绝未知模型/不完整来源/错误角色/错误关联，
超限显式ResourceLimit，不以空集合成功；来源准入不是Predicate执行、Checked、
Authenticated、Established、CompleteView、Normalizer或RootExport授权。

依赖审查对象为现行0198/0200/0210/0213/0214/0217/0219/0221、0261–0271及
相关旧native/substitution闭合类型；未来登记需给精确直接依赖，不能把本清单称为
完整transitive attribution证明。G02/F03仍开放。
