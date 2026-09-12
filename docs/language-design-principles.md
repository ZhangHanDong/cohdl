# CoHDL 语言设计原则与哲学

**状态：评审指南草案，2026-09-09；2026-09-10 校正依据。** 本文从既有 Constitution、原则约束、Accepted RFC 和已记录实验中提炼判断方法，供语言演进与 RFC 评审使用。它不增加 Accepted 语义，不取代语言规范，也不接受 M2 的具体方案。P01–P10 只是本文的引用编号。

**2026-09-10 main 核对：** 已拉取 `0e3d7705ab393a5c64c20836540ad1bc2e49898e`；[RFC-032 subdesign][subdesign] 是 Accepted，且已写入 note 10，应作为 M2 依据。此前将其与本分支同编号的旧 harness 提案混淆并排除，是文档错误，现予以撤回。已关闭的 harness 提案继续不提供语言规范依据。

适用对象是 cohdl.dev 的语言、编译器及其公共接口。文档权威关系继续遵循项目指引与[设计快照说明][snapshot]；若本指南与有效规范冲突，应修订指南，并按既有流程处理规范与实现的差异。以下“既有依据”指有来源的原则，“应用判断”指本次归纳，“提议”指尚需接受的设计。

## 设计哲学：让表达能力建立在可检查的契约上

**CoHDL 用少量可组合的概念表达电路，让编译器检查已建模的约束，让 AI 和人能够预测、审查和修复设计。** 这句话是对[产品宪章][constitution]的归纳，重点有三层。

第一，语言交付可检查的设计。`.cohdl` 源码、显式依赖、锁定信息和引用的构建资源共同构成可审查的输入；画布是投影，AI 的解释是辅助。硬件设计中的连通性、器件身份与物料不能只存在于一次聊天或 GUI 会话里。

第二，严格性必须支持组合。单位、trait 和引脚义务约束错误组合，泛型、fn 和数组扩大复用能力。增加一项限制时，要说明它保护什么，以及它怎样让更复杂的设计仍可检查；增加表达能力时，要说明它怎样保留这些保护。严格性与表达能力要一起论证。[既有依据：宪章与设计回归][governance]

第三，抽象压缩描述，同时保留物理责任。十路复用仍包含十路真实器件；逻辑容器没有封装，但容器里的元件不能从 BOM 消失。可计算的坐标只是已声明的放置事实，仍需后续物理检查。这也是语言、EDA 工具、制造和实测之间的责任边界。

## 发生冲突时的取舍顺序

沿用[宪章的优先级阶梯][constitution]，不重新排序：

1. 正确性与机器可检验性（gradeability）。
2. AI 生成的规则性与局部性。
3. 人的可审查性与信任。
4. 可组合性。
5. 对真实 EDA 生态的忠实输出。
6. 人工书写便利。
7. 功能覆盖广度。

“AI-native”因此不等于“模型容易吐出一段文本”。容易生成但无法可靠检查的写法，不能凭简短胜出；正确性成立后，再比较生成成本、审查成本与复用能力。借鉴 Rust 的严格性或 EDA 的成熟经验，都要落到具体硬件问题上，不能直接移植其全部概念。

## 十条原则：依据、约束和评审问题

### P01 可检查的承诺必须与检查一起交付

**既有依据：** [原则到约束][mapping]、[RFC 流程][rfc-process]。凡声明某种设计性质受到保障，就要有对应的编译阶段、条件与失败信号；不能先允许语法，随后长期依赖约定维持正确性。

**应用判断：** 判断一个功能完成，要沿着“源码 → 解析/解析符号 → 类型与展开 → 连接义务/残余 DRC → 适用输出”查看完整路径。具体阶段以规范为准；只跑通 parser 不代表支持可检查的循环。

**评审问题：** 它承诺检查什么？哪个失败用例能证明检查确实执行？若没有相应数据或机制，文档如何限制承诺？

### P02 在最早具备充分信息的阶段检查结构错误

**既有依据：** [单位类型 RFC-001][units]、[引脚义务 RFC-002][pins]、[DRC 分类 RFC-004][drc]。能归入类型、trait、连接义务的错误，优先在那里拒绝；残余 DRC 负责规范明确留下的整体关系。

“最早”不等于“声明一出现就检查完”。required pin 的完整义务在最终 design 组装、fn 展开后检查，否则调用者本来负责的连线会被误报。`optional` pin 可以按其声明规则不被提及；`nc` 只表明明确不连接，不证明这种决定符合实际电路用途。

**评审问题：** 检查需要哪些信息，何时齐备？删掉或后移旧检查时，替代机制是否覆盖同一错误集合？涉及数字本身不能成为把检查归入 DRC 的理由。

### P03 扩展共同语义，控制新增概念的永久成本

**既有依据：** [规则性、稳定概念原则][mapping]、[概念模型][concepts]。概念要有独立职责；新增机制应优先复用既有解析、绑定与检查规则。

**应用判断：** module 管名称与可见性，fn 管可复用电路展开，subdesign 管显式端口、具名层级与默认布局，数组管可寻址的实例族。M2 应与这些既有契约组合，不能把循环标签悄悄变成第二种 subdesign。fn 纯计算化须明确修改既有构造能力并论证迁移。

“共同语义”不要求所有代码挤进一个函数，也不自动禁止已经接受的语法糖。例如数组 fan-out 与显式成员列表可以经同一检查形成同一网络；新拼写仍须通过规则性和概念成本评审。相似写法也不必含义相同：单网 fan-out 不能冒充多网数据级联。

**评审问题：** 新概念承担了什么旧概念无法承担的责任？能否解释其与最近邻概念的区别？五年后仍值得维护吗？

### P04 语义由显式、可固定的输入决定

**既有依据：** [局部性原则][mapping]、[精确依赖与内容哈希 RFC-029][dependencies]、[位号锁 RFC-005][identity]。设计不应依赖未声明的 GUI 状态、当前时间或隐含联网结果。

**应用判断：** 做可复现性验证时，应固定编译器版本/语义与选项、源码及模块路径、项目清单与依赖锁、完整的精确依赖内容、适用的输入 `design.lock`，以及所有被引用的构建资源内容。例如 [RFC-020 的板框 DXF][outline]在 build 时读取；只改 DXF，也能改变几何输出或触发 E1006。项目资源的版本记录属于复现输入管理，不能把依赖锁的内容哈希保证自动扩大到它。这里是把既有契约的输入说明完整，并非发明新的锁机制。

未来选型可以由代理或库流程提出候选，再把器件与依据固定到设计输入。市场库存变化不能使一次离线 check 自行换料。是否引入新的外部事实类型及其来源格式，属于后续提案。

**评审问题：** 同一份设计为何会在另一台机器或另一天改变结果？所有影响语义的输入是否可见、可固定？

### P05 连接、默认与例外都要有可追溯的来源

**既有依据：** [显式性原则][mapping]、[泛型 RFC-007][generics]。连接来自源码中明确的 net；通过 fn 复用的接线仍遵循同一连接规则，默认来自可阅读的声明。

显式性允许复用定义中的可见默认，不要求每个调用者抄写一遍参数。它要求读者能沿定义和实参解释结果。RFC-032 已明确允许 place 沿 subdesign 路径访问内部实例并覆盖默认布局；电气访问仍经过端口。新增循环须说明怎样沿用这一例外，不得将其扩大为任意内部数据访问。

**评审问题：** 这条连接、这个默认值或访问权限由哪一处声明授权？隐藏在 helper 或布局块中后，是否仍遵守同一边界？

### P06 身份稳定与重复构建确定性分别验证

**既有依据：** [持久身份原则][mapping]、[RFC-005][identity]。位号沿层级路径持久保存，移除项留下 tombstone，分配不能碰撞。

重复构建确定性检查“相同输入是否得到相同输出”；身份稳定检查“编辑之后，仍是那个器件的对象是否保留身份”。一个按出现顺序命名的生成器可以完全确定，却在插入或交换代码时把旧身份分给另一用途。对不同输入，不能拿第一次的字节一致性结果证明第二种保证。

**应用判断：** RFC 应明确哪些编辑保留路径、哪些会改变路径，并测试插入、重排、删除和扩容。当前 fn 调用存在顺序身份限制；M2 提议的具名循环只针对循环站点，不声称解决任意 helper 重排。`design.lock` 没变也不能代替连接、物料与布局差异审查。

**评审问题：** 身份由名字、结构还是出现顺序决定？代码重排后，旧位号会不会对应到另一种用途？

### P07 抽象不能消除真实元件与输出责任

**既有依据：** [fn RFC-006][functions]、[实例数组 RFC-024][arrays]、[忠实输出原则][mapping]。真实实例必须进入后续适用的检查和制造投影；容器不能被伪装成器件，也不能使内部器件被跳过。

**应用判断：** 对手写和抽象表达的两个等价设计，比较真实实例、连接分区、NC、物料和有效放置；不要只比较源码长度或匿名网名。不同输出格式按各自合同验证：无法表示的信息须显式失败或明确披露，不能声称所有格式携带完全相同的信息。

`check` 可以验证尚未绑定采购 part 的模型；构建所需的物料证据仍须在 build 阶段满足。简化接口样例通过 check，不足以证明 BOM 或制造文件正确。

**评审问题：** 每个真实对象最终去了哪里？哪个目标格式保留或不承载哪些数据？是否有静默漏项？

### P08 诊断、格式化和工具接口属于语言契约

**既有依据：** [工具原则][mapping]、[fmt RFC-009][fmt]、[JSON 诊断 RFC-010][json]、[错误注册表 RFC-011][errors]。AI 的修复循环依赖这些接口，不能把它们当作语法实现后的附带工作。

**应用判断：** 新语法应具有规范化形式和可定位的诊断。涉及展开时，需要说明错误怎样返回源码及必要的调用上下文；这一点在 M2 中仍需实现验收。LSP 与其他消费者应理解同一个已检查的设计，不另行推断一套电气语义。

**评审问题：** 用户或代理能否从错误找到要修改的源码？格式化是否保持语义？CLI 与编辑器会不会给出相互矛盾的判断？

### P09 每项保证都说明依据、适用条件和未覆盖部分

**既有依据：** [宪章的模拟/布局边界][constitution]、[intent 的零判定影响 RFC-012][intent]。编译器验证已定义的性质；注释里的愿望不会自动变成约束。

**应用判断：** 区分“语言模型通过检查”“符合需求”“物料可构建”“板子完成物理检查”“实物在给定条件下工作”。Sonde 的既有实验表明，错误地交换通道仍可能通过语言检查，但独立参考网表对照能够发现差异。这是补充验证各司其职的例子。

对未来 M3/M4，必须明确所需器件事实及缺失事实的处理，不能让“没有数据”被解释成“条件满足”。采用何种新类型、错误或结果状态，要单独走 RFC；本文不引入新语法或新的全局检查阶段。

**评审问题：** 通过这个检查，究竟可以说什么？依据来自哪里？要证明剩余结论，还缺哪一种证据？

### P10 演进必须显式修改契约并留下理由

**既有依据：** [RFC 生命周期][rfc-process]、[演进治理与设计回归][governance]。重大功能说明目标、原则、概念、检查和兼容性影响；Accepted RFC 与语言规范同步，公开接口变化明确说明迁移和弃用安排。

原则可以在有依据的治理过程中修订，不能借“特殊情况”悄悄绕开，也不应因一句历史状态永远禁止演进。布局约束入口的开放就是有记录的范围修订；描述约束与编写自动布线求解器仍是不同责任。

**评审问题：** 哪条旧约束被修订，哪条仍成立？谁作出决定，理由在哪里？已有实现测试通过之外，七维一致性问题是否都得到回答？

## 三个案例：让原则实际影响取舍

| 案例与状态 | 原则怎样发挥作用 | 不能据此宣布什么 |
| --- | --- | --- |
| RFC-032 subdesign 已接受；2026-09-10 已核对 main | P03/P05/P10：沿用端口、层级与放置契约，显式处理 M2 组合 | 不能把代码或测试存在当作规范已接受；也不能据此说 fn 无法跨包，pub fn 跨包调用已有实验 |
| M2，现有 RFC 草案，尚未接受 | P02/P03/P06/P08：共用表达式规则，检查具体索引；提议具名循环保护站点身份，并要求诊断与工具配套 | 具名循环、Int 参数、工作量上限仍是候选方案；符合某项原则不等于唯一合法语法 |
| M3/M4，后续目标 | P01/P04/P09：选型公式、器件事实和约束检查各自有依据；新保证必须能实际检验 | 会算电阻值不等于稳压器已选对；写了 assert 风格愿望不等于编译器已经执行它 |

案例证据分别见 `book/src/cohdl/subdesign.md`、`docs/proposals/rfc-draft-bounded-compile-time-programming.md` 和 `book/src/cohdl/sonde-case.md`；对应原始验证材料位于 `book/examples/subdesign/`、`docs/proposals/fixtures/m2-programmability/`、`book/examples/sonde/`。本文复用它们的已记录结果，没有重新运行编译器或硬件测试。RFC-032 的规范效力来自 main 的接受记录与 note 10；历史实验用于说明实现行为，不能代替规范或最新验证。

## RFC 作者可使用的评审卡

这是对既有[一致性矩阵][matrix]和设计回归的辅助整理，不增加审批角色或第二套 RFC 流程。

| 需要写清的内容 | 回答应具体到什么程度 |
| --- | --- |
| 真实问题与替代方案 | 给出目前可复现的设计需求；说明现有 fn、subdesign、数组或库方法哪里不足 |
| 目标与取舍 | 引用优先级和适用原则，明确获得什么、支付什么概念成本 |
| 语义边界 | 哪些输入、类型、作用域与展开规则决定结果；哪些操作不在范围内 |
| 检查契约 | 依据哪些事实、在什么条件下检查何种性质；何时检查、缺失事实如何处理；给出成功/失败例及未覆盖部分 |
| 身份与输出 | 保留哪些路径；如何验证实际元件、连接、物料和放置没有遗漏 |
| 兼容性与工具 | 哪些旧行为、错误或输出受影响；fmt、JSON、LSP 等如何跟进 |
| 一致性影响 | 填写 Concepts / Grammar / Oracle / Diagnostics / Netlist / Compat / Trust，逐项回答 High/Crit |
| 接受与交付 | 需要更新哪些规范/RFC/决策；实施时哪些检查与文档必须一起完成 |

对 M2 的直接应用：先定义参数计算、真实电路构造和分阶段检查，再以 LED、阻容通道、嵌套复用检验“数量变化后类型、引脚义务和拓扑仍可检查”，最后评审表达式与循环拼写；明确修订 RFC-007 的参数限制和 RFC-024 的字面量限制；沿用 RFC-032 的端口与布局规则，并补齐循环与 subdesign 的组合；用手工展开例验证图等价，用真实 part 例另验构建与锁。空循环的静态检查范围、具名循环成本及展开上限都需在接受时明确，不能靠本文提前定案。

## 维护方式与版本基线

新增条目先寻找既有来源。若只是更清楚的解释，补充反例和引用；若改变优先级、增加硬约束或放宽已有保证，走现有 RFC/目标变更流程，并在有效规范中记录。不要让 P01–P10 逐步变成与 note 10 竞争的“第二规范”。

本次基于文档分支 `9ac0e3c` 与已取得的 main `ab44257`；以下引用固定到 `ab44257`，不声称它是 2026-09-09 的最新远端提交。快照中的“未实现”“尚无兼容性影响”等历史状态不作为当前结论。M2 的独立草案仍为 Proposed。

[snapshot]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/README.md
[constitution]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/01-product-constitution.md
[concepts]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/02-conceptual-model.md
[mapping]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/04-principle-constraint-mapping.md
[matrix]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/05-coherence-matrix.md
[rfc-process]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/06-rfc-process.md
[governance]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/08-evolution-governance.md
[units]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-001-units-as-types.md
[pins]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-002-pin-connection-obligation.md
[drc]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-004-drc-reclassification.md
[identity]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-005-designator-allocation.md
[functions]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-006-nested-fn-calls.md
[generics]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-007-generics-over-specs.md
[fmt]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-009-fmt.md
[json]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-010-check-json.md
[errors]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-011-error-registry.md
[intent]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-012-intent-annotations.md
[outline]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-020-board-outline-dxf.md
[arrays]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-024-instance-arrays.md
[dependencies]: https://github.com/conol-ai/cohdl/blob/ab44257/docs/design/rfc-029-package-dependency-versioning.md

[subdesign]: https://github.com/conol-ai/cohdl/blob/0e3d7705ab393a5c64c20836540ad1bc2e49898e/docs/design/rfc-032-subdesign.md
