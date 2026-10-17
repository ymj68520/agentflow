# Engineering gates / freshness / limits

代码门禁HEAD=4ce26eb5ce32d291d638ddb7a6d1c2fd602fa385。只有最后固定代码
上的完整fmt --check、all-target/all-feature Clippy -D warnings、cargo test --all -j2、
spec-check均成功才验收；开发focused/Portable通过不是完整门禁替代。
实际退出、Cargo pass records、组数、哈希及最终完整HEAD见本阶段final-runtime-results。

注册Corpus254 CaseIds分列170 shape、14 registered-rule、51 metadata admission、
8有界DAG子图及11 Portable（9矩阵+2运行时变异）；88/88字段/Variant站点。
Rust二进制包含复用的21旧fixture记录，不冒充21新CaseIds。35旧Root输入通过
当前Native/direct/private/alternate路径重验能力拒绝，不列为新254注册CaseIds。
全部Raw Capture CaseIds=0；不是Full G01 Fact Conformance。

独立audit-document-occurrence-staging.py核对完整old58/479 JSON、2065原路径
raw哈希、704/2064描述符闭包、新21/替换3类型、旧683 ranks、97 selected imports、
90 live declarations、每个shape/admission/rule/DAG/Portable真实回执、DO→源→规则→
Descriptor→CaseId→Compiler映射与48向量转换。形状Expected由schema派生，Metadata
负例由独立设计手写；它们明确不是独立Raw Predicate Full-field Oracle。

原Native默认独立差分需新鲜53/53，F1需26 lowering+36 native inputs/40变异，
原51完整Actual保持一致。不能把TRACE_ONLY_MATCH当Native Full Seed完成。
五组历史构建按legacy-isolation.md各自精确源运行，9542 Git blobs前后核对。
历史CaseIds、Cargo pass records与新注册CaseIds分别统计。

原目录最终提交后实际36个tools/gen-*.py（新增1个），连续完整两轮；全部72次
exit0且baseline/round1/round2/final完整路径及原始字节哈希相等才通过。
不使用镜像、不拼接失败片段；新增Registry/Descriptor/Corpus及文档先纳入基线。
若I/O失败，失败日志保留，必须从最终树重新连续完整两轮。不可用旧阶段文件图。

## 开发记录

发生一次Registry输出open的Windows OSError22，原日志dev-registry-final.log保留，
原目录重跑成功；没有修改旧Expected或用副本替代。提交时遇到先前留下的空index.lock，
确认无Git进程后移入本阶段target证据目录保留，未重置索引。曾有补丁上下文未匹配，
工具未写入；按实际格式重新应用。一次Clippy要求将测试u64→usize改为checked conversion。

新建noncanonical负例最初把阶段写成D0。冻结AF-SEMANTIC-0176和Codec规定合法JSON
非规范字节在D1拒绝，确认后修正的是新registration-case标签，并新增D0/D1实际阶段
断言及回执；原规范/旧Expected/正式算法均未变，没有production false-Proven。

完整DAG/Scope/normalized identity仍NotImplemented。执行的DAG子图负控不会被用来
把原设计中的完整SourcePackage Scope向量改为Executed。Default Decoder限制保持
32MiB input/1MiB scalar；新Portable测试显式128MiB input/32MiB scalar只是在既有
有界API中抵达D4，默认D0 ResourceLimit为独立负例。未修改生产默认限制。
