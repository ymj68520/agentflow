# Source/Origin/FactKey / observations / DAG

源链条为SourcePackage→OwnerAnchor/真实slot→DocumentOccurrenceBinding→
RawInputOrigin.document_binding→RawParsedInput→RawValidationSubject→Source.raw_validation。
同一RawInputOrigin又进入RawValidationOrigin→Origin→FactKey。
Basis/Witness/EvidenceRef/Prerequisite/Consumer中的原Subject/Source引用均指向
这一完整新闭包。没有最后拼接UUID，也没有把外层slot塞入旧source_ordinal。
RawDocument的两个内容Variant不改；完整文档仍作为真实输入保留。

容器document slot、文档内部声明entry、Port original/sorted ordinal及Observation
是五个独立坐标。RawDocumentMember与DeclaredStep仍需指同一原成员；新Owner不
替代内部路径。不同O能产生不同正式输入身份；同一完整Scope/C/K/Predicate/
成员及非观察payload复用一个Origin，合法Observation只增加边。跨Scope不强行合并，
同FactKey冲突payload仍InvalidFacts。模型/输入投影相等测试不是Validator已产FactKey。

新注册模型使这些身份可表示，非权威Metadata可验证实际包与O关联；没有实现
完整Scope或Canonical ObligationIdentity构造。手工Metadata key比较、shape probe
和归约身份的规范方程均不能冒充实际Raw Producer Fact Capture。

AF-SEMANTIC-0217不改：相等typed内容共享，真实不同Owner/slot的binding为不同
typed值，不靠重复相等节点/换rank/随机盐/无关根绕行。Package→owner→容器→
文档的描述符闭包有100个输入类型，无Scope/Fact/Witness/Obligation/Report/binding
回边；独立审查递归验证完整闭包，不是只搜字段名。

执行的8个DAG负控仅覆盖三种已注册scalar/ordinal类型的有限只读子图：共享值、
不同slot、数学2<10、重复相等节点、换rank、不可达/悬空引用、环、output类型/
随机额外字段。另有完整CanonicalValueDag envelope缺字段/错宽度shape负例。
这是明确的注册负控，不是完整SourcePackage Scope编码器或T4Normalizer。
因此原设计向量37–40等需要完整Scope构造的条件仍Not Executed，不借这些小图改标。

原Raw Port算法完全冻结：一个Case、Input-before-Output、U32稳定数值排序、原/
排序ordinal分离、真实前驱、早失败停止，Positive=P、Zero=V、Duplicate=V。
SourceAdmission错误不转成Port V/U或全Schema/Guard/RootRecovery/RootExport判定。
canonical_ports、invalid_contract_report、Guard/Bytes/String/Bounds/Substitution均未改。
