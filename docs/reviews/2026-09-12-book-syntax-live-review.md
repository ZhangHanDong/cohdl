# Book 语法新增章节与 Claude Live 最新特性判断复核

> 本文保留首轮审查时的状态。Claude 在 14:34 答复并修改七篇章节后的处理情况与新增发现，见[后续增量审查](2026-09-12-book-syntax-live-followup.md)。

审查结论：七篇章节的组织和可编译配套工程可以保留，但当前文字有多处会误导能力边界的事实错误，尚不宜作为已核准的完整语言参考。以下是审查意见，没有据此修改章节、语言规范或 M2 范围。

## 来源与验证基线

- Claude 本地会话：`/Users/zhangalex/.claude/projects/-Users-zhangalex-Work-Projects-cohdl/d353a61e-4076-4f7e-a2e8-dc70afcb5b67.jsonl`。
- 新章节交付答复：`2026-09-12T05:23:43.935Z`，北京时间 **13:23:43**。
- 正反面、走线、叠层答复：`2026-09-12T06:08:38.717Z`，北京时间 **14:08:38**。本轮已读到这一条，没有停在旧 Book 评审。
- `mempal_session_peek` 返回 `Transport closed`，本次用只读本地会话回退，没有向 Claude 发消息。
- 审查对象：`book/src/cohdl/syntax/` 七篇 Markdown、`book/examples/syntax-tour/` 及上述两条答复。
- 实现基线：main 快照 `0e3d7705ab393a5c64c20836540ad1bc2e49898e`、CoHDL 0.7.0；从干净的独立工作树执行 `cargo build --offline` 后测试。当前文档分支的 `src/`、Cargo.toml、Cargo.lock 与该快照无差异。本次未联网更新 main，不把这个快照称为已经核实的远程最新提交。
- 机器证据：[现有编译器探针、源码哈希、命令和原始诊断](evidence/2026-09-12-book-syntax-live-probes.json)。全部实验使用临时副本，没有修改教学源码或实现声明验证器。

## 需要修正的发现

### F1 · P2：不能用“不能跨包发布”区分 fn 与 subdesign

位置：`book/src/cohdl/syntax/composition.md:37`、`:81`。

该页把“不跨包发布”列为 fn 的选择条件。当前 `lib/passive/src/circuits.cohdl:8`、`:18` 就有两个 `pub fn`。在课程副本加入 `passive::bulk_10u(j.P4, j.P5)`，check 和 build 都通过，证明确实能够复用依赖包里的电路函数。旧 Book 的 `cohdl/subdesign.md:59` 已明确纠正过同一问题，新章节把它重新引入了。

应以**显式端口、保留的具名层级、整体放置与内部覆盖**区分 subdesign；包的发布与跨包可见性由模块/包系统管理。相邻的“这四种机制都不生成接线”（`:85`）也要限定：fn/subdesign 体内的 net 会随调用或实例化展开；尚缺的是按索引计算、自动构造菊花链等 M2 能力。

### F2 · P2：布局与属性并非“永远不改变 check 结论”

位置：`layout-facts.md:3`、`design-nets.md:40`，以及 `index.md:19` 和示例源码的相同注释。

实际反例：将 place 目标改为不存在的实例，check 返回 fail/E1007；将 `#[high_current(500mA)]` 改为 `500mV`，返回 fail/E110，并有解析恢复产生的 E010。RFC-013 本身要求对约束词汇、引用、数量进行验证；RFC-027 的结构化属性也有单位与引用检查。

应写成：**合法的布局/物理约束不改写电气连接，不新增那四条以外的电气 residual DRC；约束本身仍须通过语法、名称、单位和结构检查。** CLI 的总 verdict 可以因此失败。`#[designator]` 还明确影响位号及带位号的产物，不能与 `#[intent]` 一律概括成无影响的元数据。

### F3 · P2：源码重排不影响位号的保证过宽

位置：`design-nets.md:65`；`composition.md:74` 的稳定性说明也需保留条件。

同一副本先调用 `drive_led` 再调用 `passive::bulk_10u`，构建后只交换两条调用的顺序，保留原 design.lock 再构建：

| 物理对象 | 重排前 | 重排后 |
| --- | --- | --- |
| drive_led 内的电阻 | `SyntaxTour::__fn0_drive_led::r` → R1 | `SyntaxTour::__fn1_drive_led::r` → R5 |
| bulk_10u 内的电容 | `SyntaxTour::__fn1_bulk_10u::c` → C3 | `SyntaxTour::__fn0_bulk_10u::c` → C4 |

旧路径进入 tombstones，顶层具名实例和本例 subdesign 内具名 R/C 的位号保持不变。原因是 `src/check/expand.rs:1863` 的 fn 调用序号进入身份路径。应保证“路径保持相同且沿用设计锁时的位号稳定”，不能把该保证推广到所有语句重排。这与 M2 为生成对象定义身份的动机直接相关。

### F4 · P2：annulus 已实现；现有焊盘语法被漏掉并判成未实现

位置：`parts-footprints.md:31`、`:33`，以及 Claude 13:23 答复中的“AST 里有这个变体，parser 不认”。

`src/parse.rs:1278` 直接接纳 `PadShape::from_name` 的结果；`docs/provisional-syntax.md:279` 已描述 annulus，`lib/@contrib/mic/src/t3902.cohdl:33` 有实际用例。把课程中的焊盘改为 `shape: annulus`、`size: (0.8mm, 0.4mm)`，check 无诊断通过。

该章也漏了已实现的 `chamfer`、`paste: circle(...)`、`paste: segmented_annulus(...)`。可以为入门裁剪内容，但不能将裁剪解释成未实现，也不应同时宣称列全了当前语法。需要明确“已实现的 provisional 工艺语法”与“Accepted RFC 语法”的状态差别。

### F5 · P2：物理焊盘不是与器件引脚逐个一一对应

位置：`parts-footprints.md:49`、`tour.md` 末段以及 `src/led.cohdl` 的对应注释。

当前 E807 比较的是**去重后的电气焊盘编号集合**与器件物理引脚编号集合（`src/check/footprints.rs:74`、`:119`）。同一编号可以有多个实体，用于散热焊盘、热过孔或不同面的铜焊盘。课程副本给 LED 封装增加第二个 `pad 1` 后仍可 build。

因此应解释编号集合一致和一号多实体的关系；“焊盘数和引脚数一致”不是通用的构建保证。该实验只验证语言规则，不认可修改后封装的物理合理性。

### F6 · P2：required/optional 的物理解释超出了编译器保证

位置：`units-devices.md:64`；`design-nets.md:36`。

把 required/optional 直接翻成手册中的“必须接/可悬空”，会遮蔽 required 也可由 nc 解决的正式语义。它们应先解释为**必须显式交代连接状态/允许不提**；nc 表达作者的明确决定，编译器没有核实这项决定是否满足器件工作条件。

同段“除非写进 spec”也不能暗示任意引脚电压/电流上限都会自动被检查。当前 `src/drc.rs:22` 只对特定的器件级 `voltage_rating: Voltage` 做 D001 比较；写一个电流规格不会自动建立电流求解或上限断言。

### F7 · P2：用“缺少 Stackup 段”解释当前 IPC 输出缺口不成立

位置：Claude **14:08** 答复的叠层判断，尚未作为新章节落入 Book。

当前 `src/emit/ipc2581.rs:1809` 的 FAB_LAYERS 和 `:1826` 的 emit_stackup 已输出固定的两铜层、名义 1.6mm 叠层。课程副本 `build --emit ipc2581` 的 XML 实际包含一个 Stackup 和九条 StackupLayer（含丝印、钢网、阻焊工艺行；**不是九层铜板**）。`docs/compliance-report.md:1629` 已说明它不是制造商的真实叠层。

正确的缺口是：**语言尚无作者可配置、可引用厂商资料、可描述一般多层板的 stackup 模型；输出端已经有固定的名义叠层。** 新 RFC 的动机应建立在这个差异上。当前实现也不能因为有 Stackup 就被称为可制造的完整交付。

### F8 · P3：计数和概述需做一轮内部一致性修订

- `index.md:5` 写八种顶层声明，表里实际列了九种；AST 另有 use 导入项。应先说清统计的是声明还是所有顶层项。
- 同段“只有 design 里的 inst”遗漏 fn/subdesign 体内随展开产生的实例；应说明最终由顶层 design 汇总装配。
- `design-nets.md:26` 的“DRC 只读注解”应限于从哪里取得网络电压/地标记；D003/D004 实际还依赖连通图和引脚电气角色（`src/drc.rs:92`、`:116`）。
- `tour.md` 写“三种路径形态”，随后列了四种。
- `parts-footprints.md:33` 把槽孔限制成只能用于 oval，但 provisional §9 的规则和当前实现只明确禁止 circle 等非法组合；应按实际形状规则解释，不再凭示例扩成封闭词汇。

## Claude 正确发现的问题

数组元素作为 fn 的**实例参数**失败，确实是现有实现与 RFC-024 契约不一致：课程副本 `drive_led(leds[0], ...)` 实际得到 E202，外加未绑定调用产生的三条 E701。`src/check/expand.rs:2153` 附近的 resolve_instance_arg 只按 base 名查找，没有处理索引。`tests/inst_array.rs:156` 的既有测试只覆盖 `sw[0].A` 这种 **Pin 参数**，不足以证明实例参数位置合规。

应保留这条发现，但标为已有特性的实现缺陷，不能变成“CoHDL 语言禁止数组元素参数”的设计原则，也不能把修复包装成 M2 新增能力。本轮没有修改编译器或增加验证器原型。

## 对最新特性优先级判断的意见

| Claude 的方向 | 审查意见 |
| --- | --- |
| 正反面已支持 | 成立，指外侧元件放置的 side top/bottom；不能等同于一般多层叠层或任意 padstack |
| 路由求解留给伙伴，语言记录要求 | 符合当前 Constitution；这是产品边界选择，不是“几何求解必然无法确定”这类数学结论 |
| 返回的布线板需要验收 | 成立；应明确验证对象、与已检查 IR 的对应关系和工具边界。已关闭的 harness 提案可作历史研究材料，不自动成为当前规范或已获授权的新任务 |
| 叠层是一个真实缺口 | 成立，但缺的是一般可配置语言模型，不能遗漏已有的固定 IPC 输出 |
| stackup: profile_ref，厂商包提供 profile | 可作为未来候选，不是现有语法，也未证明示例厂商包已存在。须另议 profile 种类、锁定身份、单位、引用与结构校验；不能直接承诺零 check 影响 |
| 叠层应成为下一项语言工作 | 现有证据不足以重排用户优先级。本会话当前仍聚焦 M2 RFC；语法比较、声明验证器已移到附录作为后续工作，stackup 可另记未来需求 |

建议先修正上述事实与教学边界，再讨论叠层方案。七篇章节的结构、源码 include 和“小工程读产物”的方式可继续使用。

## 本轮实际验证

- 11 次现有编译器调用：8 次成功，3 次预期失败；逐次命令、退出码和完整诊断见机器证据。
- 原课程副本 check、fmt --check、build --emit ipc2581 通过。网表 11 实例/8 网络，BOM 5 行，layout 10 个放置，与教材计数一致。
- 确认跨包 fn、annulus、重复焊盘号可用；确认错误布局目标/属性单位会失败；复现数组实例参数缺陷与 fn 重排位号变化。
- `mdbook build book` 通过；`python3 book/tools/check_book.py` 通过 **71 章、74 个 HTML 页面、3655 个本地链接/资源/锚点**；语法章节 HTML 中无未展开的 include；`git diff --check` 通过。
- 本轮没有进行 KiCad GUI 导入、布局布线、IPC schema 全量验证、制造或板上测量，也没有新增学习者理解验收。Book 构建通过不能替代本报告的语义审查。
