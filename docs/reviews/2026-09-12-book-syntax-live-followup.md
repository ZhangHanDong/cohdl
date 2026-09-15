# Claude Live 14:34 修订后的增量审查

> 本文保留增量审查时的状态。用户随后授权的教材修订及验证，见[修复记录](../../book/src/learning/2026-09-12-book-syntax-repair.md)。

本次读到的最新实质答复是 `2026-09-12T06:34:46.785Z`（北京时间 **14:34:46**），晚于[首轮审查](2026-09-12-book-syntax-live-review.md)中的 13:23/14:08 答复。七篇语法章节均在 14:33 左右更新。Live connector 仍返回 `Transport closed`，内容来自同一项目的本地 Claude 会话，只读取得。

结论：上一轮的核心能力边界错误已大部分修正，不能继续把旧意见当作尚未处理。补入焊盘语法时产生了新的几何解释错误，仍需修改后再关闭审查。本轮只记录意见，没有改动章节或 compiler。

## 本轮发现

### R1 · P2：annulus 的 size 被解释成宽、高

位置：[物料与封装第 31 行](../../book/src/cohdl/syntax/parts-footprints.md)，原句为“矩形、椭圆和环形是 `(宽, 高)`”。

当前语义是：矩形和椭圆用 `(width, height)`，**环形用 `(outer_diameter, inner_diameter)`**，并要求外径 > 内径 > 0。环形不是椭圆，也不是以两个方向的尺寸界定铜面。

实际检查：`shape: annulus size: (0.8mm, 0.4mm)` 通过；`size: (0.8mm, 0.8mm)` 得到 E805，明确提示外径必须大于内径。即使宽高写法碰巧满足大小关系而通过，读者预期的铜形状也会错误。

依据：`docs/provisional-syntax.md:279`、`src/resolve.rs:1009`；完整输入和诊断见[本轮探针](evidence/2026-09-12-book-syntax-live-followup.json)。

### R2 · P2：courtyard 不能继续写“形状同 pad”

位置：同章第 50 行。新增 annulus 后，pad 与 courtyard 的形状集合已经不同。annulus 仅用于电气焊盘；courtyard 仍是 rect/circle/oval。

实际把 `courtyard` 的 shape 设为 annulus，check 返回 E806：`annulus is only valid for electrical pads, not courtyard`。应直接列出 courtyard 的闭合词汇；同样不要把 annulus 推广给 mount_hole 或 window。依据：`src/parse.rs:1095`。

### R3 · P2：paste: circle(...) 不限于比铜小的开口

位置：同章第 33 行，把三种 paste 写法概括成“比铜小或分段的开口”。

`paste: (w, h)` 的矩形开口有铜包络范围限制；**`paste: circle(d)` 可以比铜包络小，也可以大**。本轮 0.3mm 圆铜焊盘配 `paste: circle(0.4mm)` 的检查通过。应分别介绍三种形态的尺寸含义，不能从矩形 aperture 的规则推广到圆形。

依据：`docs/provisional-syntax.md:273`、`src/resolve.rs:1243`。这只确认语言允许的几何表达，不是对某个制造方案的推荐。

## 仍需收紧的表述

- **Accepted 与已实现分开**：`syntax/index.md:35` 仍说“不写任何未接受的语法”，但新章明确介绍 provisional §10/11。应改为“介绍当前已实现的语法，并分别标明 Accepted、provisional 或记录在案的偏离；M2 提案另列”。
- **电流规格仍有单位检查**：`units-devices.md:64` 的“写一个电流规格不会自动产生任何检查”，应限定成“不会自动产生工作电流求解或电流上限检查”。字段和泛型的单位检查依然存在。
- **稳定路径仍需限定直接具名实例**：`design-nets.md:65` 的“subdesign 内部实例不受影响”不应包含其内部 fn 生成的对象。fn 调用序号仍由整个 design 的计数器产生（`src/check/expand.rs:201`、`:1863`）；这里应写“示例中 subdesign 直接声明的具名实例”。本轮是源码核对，没有另跑嵌套 fn 重排实验。
- **示例注释漏改一处**：`book/examples/syntax-tour/src/led.cohdl:2` 仍写“只有 design 里的 inst 才会”，与总览已修正的 fn/subdesign 展开说明不一致。

## 上轮意见的处理状态

| 上轮意见 | 本次核对 |
| --- | --- |
| fn 可跨包复用 | 已修正，正文和选择表都不再用跨包能力区分 fn/subdesign |
| 布局/属性自身可能使 check 失败 | 已修正，明确 E1007/E110 与电气判定的区别 |
| 位号依赖身份路径 | 已加入路径/锁条件与 fn 调用重排例子；上述 subdesign 范围用语再收紧即可 |
| annulus 已实现 | 状态已修正，但新尺寸说明产生 R1，关联表格产生 R2 |
| E807 比较去重后的编号集合 | 已修正，包括 tour 和封装示例的对应注释 |
| required/nc 不是物理可用性证明 | 已修正；任意电流规格不会自动产生电流 DRC 的方向也正确 |
| DRC 不只读取网络注解 | 已修正，明确 D003/D004 读取连通图和角色 |
| 九种声明/四种路径等计数 | 已修正 |
| 当前 IPC 已有固定名义 Stackup | Claude 已在 14:34 答复中明确承认并纠正；不是新增的 stackup 语法 |

## 最新语言特性判断

Claude 本次对叠层缺口、数组参数缺陷及 M2 优先级的判断，与已核实事实和用户决定一致：

- 两面元件放置是现有能力；一般可配置、可绑定真实厂商资料的叠层语言模型仍是未来需求。输出端已有固定名义双层叠层，不代表完整制造验收。
- 数组元素作为 fn 实例参数的 E202 是 RFC-024 的实现缺陷；Pin 参数成功不能覆盖该位置。独立追踪该缺陷，不把它变成 M2 新语法。
- M2 仍 Proposed，A 已选，B 延后；语法比较与声明验证器/兼容性审计在附录。库里只有 2 个非泛型 fn、0 个 subdesign 定义，只证明语料覆盖不足，不证明新验证器正确或全库语义审计已通过。
- 对“CoHDL 不管的三件事”的新页面，Claude 明确说尚未写，并同意保持 M2 优先。本轮没有发现另一个已落盘的叠层/走线新章节。

## 验证边界

- 比较原课程副本与当前四个 `.cohdl` 文件：仅 main/led 的整行注释变化，filter/mech 未变；manifest、依赖锁和 design.lock 未变。因此不重复上一轮 11 次实验，也没有重跑编译器全套测试。
- 本轮新增四个针对新增文字的最小检查：两个通过，两个按规则失败；使用同一 `0e3d770` / 0.7.0 二进制，并核对二进制 SHA-256 与上轮相同。原始输入、命令、诊断、最新 Live 原文和七篇章节的哈希都存入[证据文件](evidence/2026-09-12-book-syntax-live-followup.json)。
- `mdbook build book` 与 `python3 book/tools/check_book.py` 通过：71 章、74 个 HTML 页面、3655 个本地链接/资源/锚点；`git diff --check` 通过。没有改变编译器、教学源码、M2 RFC、学习者掌握状态或硬件结果。

## 用户授权修复后的状态

用户随后要求修复。R1–R3 已在物料与封装章按形状和 paste 形态分别说明；四项剩余表述已修正，并同步复用章、示例注释和 README 的声明计数。修订后示例在临时副本中重新通过 check、fmt --check、build --emit kicad_pcb，保持 11 实例/8 网络；新命令和源码哈希见[修复证据](evidence/2026-09-12-book-syntax-repair.json)。

维护记录、SUMMARY 和进度已同步；Book 构建及链接检查通过 72 章、75 个 HTML 页面、3708 个本地链接/资源/锚点。本轮所列教材修订项已处理；RFC-024 的数组实例参数缺陷仍是独立的编译器问题，本次没有修改它。

## 上游缺陷跟踪

用户随后要求将数组参数缺陷先报告上游。已创建并回读确认 [conol-ai/cohdl #41](https://github.com/conol-ai/cohdl/issues/41)：`RFC-024: array elements fail as fn instance arguments with E202`，状态为 open。Issue 包含无需依赖库的单文件复现、标量/Pin 参数成功对照、两种实例参数写法的失败结果和回归覆盖建议。

报告时上游 main 为 `f41f9c2cf75e88d1dca687d6282eca21ac0b309b`；已核对与 `0e3d770` 的差异，参数解析函数及原数组调用测试未变。复现实际执行于 0.7.0 / `0e3d770`，没有把源码核对冒充为对新 main 的编译测试。该 issue 作为 RFC-024 实现缺陷独立跟踪，M2 范围不变。
