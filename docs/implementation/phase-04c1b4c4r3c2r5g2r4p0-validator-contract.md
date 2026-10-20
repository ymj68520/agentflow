# 离线验证器合同与使用边界

实现仅覆盖Static Manifest Validation；Real Environment Qualification、Real Executor Attestation、Real Historical Gate Execution均Not Executed / Not Authorized。phase-local JSON Schema不是AFIDL/Model8/Registry修订。候选所有真实未知字段为null，qualification_status=Not Verified；不填入本地主机或假想环境。

在仓库根目录，可在原保护器下对本地静态准备清单运行：

```powershell
python -X utf8 tools/run-production-input-graph-safe.py r5g2r4p0-manual-readback python -X utf8 docs/implementation/phase-04c1b4c4r3c2r5g2r4p0-offline-manifest-validator.py docs/implementation/phase-04c1b4c4r3c2r5g2r4p0-preparation-manifest.json
```

名称必须唯一，不覆盖旧log。返回0只代表Static PASS；不代表真实资格、执行、信任或正式权限。无效静态清单返回2。代码没有环境接入、复制、构建、测试二进制执行或密钥生成路径。小型identity接口只在明确本地root内流式核对长度/SHA256，拒绝越界/链接；它不认证来源或执行者。

主验证器使用本机已有jsonschema的Draft202012Validator及仅本地引用registry，拒绝外部schema提取；增加整数token严格类型、重复键、浮点/NaN、int64、4MiB输入、32层/50000节点等边界。闭合Schema覆盖root/candidate/gate/ref/profile命令/产物/输出/时序/资源/签名；未知字段或双侧遗漏不因两侧相同而通过。Schema结构校验与来源/语义/授权并非一回事。[JSON Schema验证规范](https://json-schema.org/draft/2020-12/json-schema-validation)、[jsonschema验证接口](https://python-jsonschema.readthedocs.io/en/stable/validate/)为实现依据。

validation-contract.json从已封存R3独立输入提取，含完整原头/profile/命令/归属hash和独立分母要求。Preparation模式不带实际输出；Synthetic模式使用每Gate两个明确标记的测试CaseId/伪产物，不冒充历史512等真实分母。CaseIds/Expected/Actual/符号产物各自对固定契约验证，不能从两侧交集或Actual生成Expected。实际全量CaseId/cfg-expanded Rust/doc-test roster仍需未来资格认证；静态TOML census只证明声明/source身份。真实执行输入不得绕过P0拒绝状态，P1/P2须另行批准并认证相应验证协议。

独立Oracle不import/call主验证器，也不读取其Actual；以单独Schema子集解释器和冻结合同逐项比对。static-test-vectors.json的Expected事先独立编写；自动遗漏矩阵的“必填缺失→拒绝”来自冻结Schema要求，而不是主验证器输出。P0只测该精确Schema子集，Oracle不宣称通用Draft实现。

Ed25519离线public verification调用已有cryptography库，签名payload为域分隔R5G2R4-EVIDENCE-v1加规范UTF-8 JSON，按键排序、无重复/浮点；仅排除signature自身bytes/digest，algorithm/key_id仍被签。有效签名只是密码学有效，不是生产信任。没有真实批准根/执行者/Trust Policy时，始终Cryptographic Trust=Not Established、Executor Authority=Not Established、Environment=NOT QUALIFIED。[cryptography Ed25519接口](https://cryptography.io/en/latest/hazmat/primitives/asymmetric/ed25519/)和[RFC8032公开验证向量](https://www.rfc-editor.org/rfc/rfc8032)用于离线接口自检；只保存公开key/signature，不保存私钥、不生成临时密钥，不把公开向量登记为生产根。

ExecutionId重放检查依据清单之外的只读历史ID集合；本轮只有明确命名的合成旧ID测试，不能宣称已实现真实分布式持久反重放服务。真实批准根、撤销、持久新鲜性/nonce及独立执行者必须由P1可信流程提供，提交者不能自授信。Failed101/cancelled-null和Interrupted79分别保留；Not Executed/PASS不可互换。所有环境证据都不产生Native/Portable/RootExport正向权限。
