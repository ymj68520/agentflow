# Gates / findings / readiness

固定代码门禁 HEAD：dbdbba88e22e064b50e63c6b168b248d84ab3e9b。
逐项实际退出、日志 SHA、Cargo pass-record/组数、最终 HEAD 及是否 clean，
以 E:/Projects/agent/target/phase-04c1b4c4r3c2r1-final-runtime-results.json 为准。
不存在用 focused 片段拼接完整门禁的验收，也没有把严格 Oracle exit 2 当作成功。

最终代码整轮四项 exit 0：Cargo 436 条通过记录、75 组、0 failed/ignored。
现行 254、Native 53、F1 26+36/40 的独立核验通过。原子联合包拒绝补强后的
完整整轮重新执行；ce35243 的先前整轮保留为 superseded，不拼接进最终证据。

正式身份冻结为 Analysis@6 / Model@5 / Report@5 / Obligation@5。
62 Specifications /493 Rules /492 covered /1 exception /0 missing；
704 Descriptors /2064 references /97 historical imports；ABI775 /AFIDL17 /
HashV1 domains14。原 2118 路径中除 5 个明确授权实现文件外，2113 原始字节冻结。
旧 Expected、全部规范/Registry、Native Oracle、F1 lowering 修复不改。

五组精确历史使用各自源树/Registry/Expected，任务运行前后校验 9542 Git blobs。
版本/CaseIds：abc91d5(512)、51239a0(51)、4d3bb59(173)、875e6e4(889)、
e623aab(262)。Cargo pass-record 另行统计，不与 CaseIds 相加混用。
现行 254 注册回归重新检查所有 shape/rule/admission/DAG-subgraph/Portable 回执。
新任务采用自己的 5 路径实现变更授权和 2113 路径冻结；旧 C2G2 以全文件冻结
实现代码的历史审计器保持原样，绝不篡改它的旧 Expected 或掩盖旧阶段基线。

Native 53/53、F1 26 lowering +36 native inputs、40 mutations 重新独立核验，
并检查原 51 份完整 Actual 保持不变。全部 Cargo/fmt/Clippy/spec-check 重跑。
Native Analyzer/direct checker/private constructor/alternate/current-only、
默认 D0 ResourceLimit 与显式有界 D4 UnsupportedSemantics 的原拒绝门禁保持。
没有放宽生产默认限制或新增受支持/激活的 Edition 状态。

最终证据提交树在真实 E:/Projects/agent 跑全部实际 gen-*.py 清单连续两轮。
要求每脚本退出 0，全部非忽略路径/raw SHA baseline=round1=round2=final，
增删改均 0。完整逐文件图与日志留 target，不以镜像、时间戳或单一目录摘要代替。
如失败，保留失败记录并从同一最终基线重跑完整双轮，不拼接成功片段。

## Findings

- C2R1-FIELD-01：OPEN/BLOCKING，正式 Fact 根字段至少 1250 实例缺口及嵌套缺口。
- C1-GAP-01、C2-GAP-01：FORMAL BINDING REGISTERED；本次没有完成 Raw Producer
  Verified for authorized scope，不把已注册等同于完整生产事实链。
- G01 的注册模型不重开；完整 Capture/Normalizer 未闭合。G02 OPEN/P1，G03 OPEN/P2。
- F01/F02 CLOSED；F03/F04 OPEN/P2；B01–B07 BLOCKING；R3-NG-B01 的原 F1 修复保持关闭。

Outcome D — Conformance Incomplete。这里的 1250 是实际已知的必需根字段未闭合
计数下界，不是新的正式模型缺口，不是完整树中唯一缺口，也不是“允许忽略”的清单。
剩余工作仍需完整 FactKey、引用/primary、Prerequisite/Consumer、TypedContradiction、
正式 ObservationSite、完整 Compiler/Context 独立投影和完整字段门禁。

Raw Capture：PARTIAL / Locally Non-authoritative；Total R3/T3 INCOMPLETE；
Native Full Seed、Standalone、剩余 Producers INCOMPLETE；T4 NOT ELIGIBLE；
Normalizer/Complete G02 Scope NotImplemented；Positive RootExport NotAuthorized。
本次交付后停止，不自动进入其它 T3、T4–T8、G02/G03、Publication 或 Runtime。
