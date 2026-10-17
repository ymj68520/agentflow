# Future independent Python Oracle — algorithm design only

本文件是未来算法设计与手算，不提供可执行Producer Oracle，也不调用Rust Actual
建立Expected。48向量全部Not Executed，当前C2复现工具不是本Oracle实现。

## 独立输入

实际提交的完整SourcePackage、Owner容器、Occurrence选择、完整RawDocument、
原字节（若提供）、候选和两个内部路径、完整Compiler与九Context内容和真实
association证据，以及未来准确登记的closed raw case/descriptor/rule内容。
不接受一个最终FactKey作为Expected的输入，不把report reason当Owner/Witness。

## 拟议独立推导步骤（未实施）

1. 用未来固定封闭schema验证全部声明域/Owner/容器/宽度/边界，拒绝unknown。
   独立扫描整个包构建完整Owner索引；检查重复Anchor和联合包完整值冲突。
2. 从输入OwnerAnchor与真实容器entry选择O，拒绝缺少/错域/越界及无来源关联。
   对每个(A,key)保存完整选定文档作完整typed比较，不只比较digest或计数。
3. 从绑定的实际文档解析真实内部路径、候选和physical关联；执行唯一允许矩阵。
   InputOrigin、Physical、Basis/Witness的O不一致即拒绝，不能取Actual提供的一个为准。
4. 根据真实C/K association独立建立roles。内容相等本身不是对象同一性证据。
   Portable重验证属于对显式完整输入的重建/验证，不认证旧进程地址或历史Owner。
5. 独立推导完整subject/source/origin坐标。仍需保留DeclaredStep与RawDocumentMember
   两条路径；外层O不替换内部路径。
6. 按冻结Raw Port语义从原声明建立应执行事件名单：identity先行，Input再Output，
   数学U32稳定排序、保留原ordinal及真实前驱，检查之后才能schema resolve。
   提前错误停止访问，哨兵不是前驱；0/duplicate V带精确类型Witness，fresh P无Witness。
7. 在未来完整registered Field Roster与应用事件Roster下独立推导FactKey、Basis、
   Witness、references、support及必要Observation/Consumer links。它们不来自Actual
   已输出事件的列表；遗漏的适用事件必须被独立名单检出。
8. 手算/独立typed规范化的Owner/Occurrence输入仍使用不变的最小DAG规则；
   相等内容共享，不同O binding不相等。输入和输出DAG各自有限，无固定点。
9. 只对相同完整Scoped FactKey且非观察payload全部一致的副本union合法观察。
   不同C/K/Scope不因O相等合并；同key矛盾InvalidFacts。
10. 以独立required-field instances检查Expected、Actual、Compared三份roster；
    即使Expected与Actual同时删掉一个必需字段，也因独立required名单而失败。
    未来Comparator mutations需包括错Owner、错slot、两边同删、漏事件、改O不改
    Source/Origin/Physical、观测次数生成新Origin和伪造外部认证。这里不执行。

## 三个独立手算

固定完整输入包S、Compiler C、Context K：Owner A slot0=X、slot1=X；
O0=(A,0)、O1=(A,1)。F输入中的binding不同，所以Source/Origin不同；X仍可共享。
同O0在report-site-1及report-site-2暴露，输入S/C/K/Predicate/Coordinate不变，
FactKey相同，观察集合为{site1,site2}，不是两个Origin。
仅X/X没有S/A/key，无法进行步骤2，MissingDocumentOccurrenceBinding；不会得到
“空但完整”的Expected事实集合。错Owner的实际borrow关联同样不能由内容相等补足。

## Coverage边界

新Oracle必须有独立输入推导和源码外完整字段合同，不能以262 registration cases、
129 descriptor sites、53 trace matches、F1 mutations或48手算设计向量替代。
所需Producer、lossless Scope、Normalizer以及Native/D10完整Oracle仍各自未完成。
没有完成这些实施工作之前，本设计算法不构成Raw Seed验收或RootExport能力来源。
