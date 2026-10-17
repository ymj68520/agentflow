# C2G1 — Document Occurrence Identity / design only

起点：1ee68d327e3c3ea8aa65305ab3e59e9352368b7e。当前正式内容全部冻结。
本组材料是未注册的设计，不是现行规范、类型分配、Producer 或 G02 实现。

唯一推荐 **Option C：显式受检的声明式 Document Occurrence Binding**。
来源必须由调用者在验证之前实际提供；只有相等内容的两份内存对象时不能建立它。
本设计仅建立 **Declared Provenance**，不建立 External Authentication。
设计完备不代表任意匿名输入都能成功：拒绝匿名输入正是必要的充分性边界。

阅读顺序：existing-binding-audit → document-identity-model →
owner-occurrence-binding → source-admission-trust-boundary → physical-native-mapping →
scope-dag-impact → origin-factkey-impact → binding-options/recommended-decision →
normative-amendment-draft → edition-migration → independent-vectors/oracle-design →
readiness/gate-report/final-report。文件均使用本文件相同 phase 前缀。

六个独立结论：

1. 完整内容相等是内容身份相等，不能推出声明式 occurrence 相等。
2. occurrence 来自真实提交的来源域、Owner 声明及容器中真实条目，不来自分配次数。
3. 不同 Owner/条目的 binding 是不同 typed values；相同内容子树仍规范去重。
4. 同一完整输入 Scope 内，同一 occurrence/成员/谓词被重复观察，FactKey 不变。
5. 旧 content-only 数据没有保存 occurrence，不能无证据补造；旧版仍按旧版解释。
6. 声明来源身份不赋予 Scope、Publisher、Runtime 或 RootExport 任何新增权限。

目标裁决：Outcome A — Formal Amendment Design Accepted，FORMAL REGISTRATION PENDING。
现行模型仍不能表达 C2 反例的独立来源；不能称 Existing Binding Verified。
48 个新设计向量全部 Not Executed。新 Raw Capture/Full-field Oracle/Normalizer 均未实现。
R3-C2 INCOMPLETE；T4 NOT ELIGIBLE；Positive RootExport NotAuthorized。
