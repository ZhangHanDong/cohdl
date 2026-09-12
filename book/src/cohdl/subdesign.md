# subdesign：把一组电路作为一个单元复用

> **2026-09-10 main 核对：** 最新拉取的 main `0e3d770` 包含 Accepted RFC-032 subdesign，规范及实现相对 `ab44257` 未变，24 项专项测试再次通过。此前因同编号旧 harness 提案而将其排除为依据，是文档错误，已撤回。下文保留 2026-09-08 实验基线；M2 已在英文工作稿中补入组合规则，仍为 Proposed，见[当前导读](programming-proposal.md)。

> **定位**：用两路电阻、电容电路认识端口、层级和整组布局。基线为 2026-09-08 拉取的 main `ab44257`；本章例子已实际 check/build。此前 Sonde 实验仍以旧基线 `9ac0e3c` 为准。

`subdesign` 让你写一次小电路，再在大板上实例化多次。每次都有自己的真实元件，对外通过声明的端口连接；还可以给这组元件安排默认相对位置，在大板上整体摆放。

它对应之前 M1 的两个需求：电源、主控、显示各自封装；相同电路反复使用。它描述的是同一块 PCB 内的逻辑结构，不会自动制造出一块可插拔的小板。

## 先看一个能编译的定义

我们从已经见过的 R、C 开始。下面的 `R_100R_F_0603` 和 `C_100p_50V_C0G_0603` 是 passive 包中的具体 part：100ohm 电阻、100pF 电容，均为 0603 贴片封装。完整文件的导入与依赖已配好，位于仓库 `book/examples/subdesign/`。

```cohdl
{{#include ../../examples/subdesign/src/main.cohdl:definition}}
```

先沿着连接读一遍：`IN → 电阻 r → OUT`，再从 `OUT → 电容 c → GND`。这是被动 RC 低通电路的连接形状；本次只验证结构，未接实际驱动和负载，也未测量滤波效果。100ohm/100pF 用于衔接此前 RC 教学，不是已选定的手表参数。

`ports` 是这组电路的**对外逻辑连接点**。例如 `OUT` 在内部接到 `r.B` 和 `c.A`，在外部可以接到另一器件的引脚。`OUT` 自己没有封装、焊盘或料号；若需要接线端子，应另外放一个真实接头。

`required` 要求这个端口最终连到子设计之外。当前端口类型只有 `Pin`；写 `IN: Pin` 不会自动声明它是输入方向，也没有声明“只能接 3.3V”。输入、输出、驱动冲突等已有检查仍依赖内部真实器件的 pin 角色及其他已建模事实。

## 写一次，得到两套真实元件

```cohdl
{{#include ../../examples/subdesign/src/main.cohdl:assembly}}
```

`subdesign channels: [RcChannel; 2]` 生成两个独立的子设计，用 `channels[0]`、`channels[1]` 寻址。每个里面都有一颗 r、一颗 c。顶层接头的 P1/P2 接第一路输入/输出，P3/P4 接第二路，P5 接共用地，P6 显式不连接。

本次实际输出为：

| 实例路径 | PCB 位号 | 坐标，mm |
| --- | --- | --- |
| `RcPair::channels_0::r` | R1 | (10, 15) |
| `RcPair::channels_0::c` | C1 | (13, 15) |
| `RcPair::channels_1::r` | R2 | (20, 15) |
| `RcPair::channels_1::c` | C2 | (24, 17) |
| `RcPair::connector` | J1 | (10, 6) |

注意第一路电容：内部位置是 `(3, 0)`，整组放到 `(10, 15)`，它最终就在 `(13, 15)`。第二路电容原本会在 `(23, 15)`，但顶层 `place channels[1].c` 覆盖为 `(24, 17)`，第一路不受影响。整体旋转、翻到背面和多层嵌套也有上游专项测试。

最终是 **5 个元件、5 条网络**。BOM 按相同物料合并后为 3 行：两颗电阻、两颗电容、一个接头。两个 `RcChannel` 容器都没有自己的位号和 BOM 行；`design.lock` 只记录表中的真实器件。

默认布局需要整组锚点才会落到板坐标。“锚点”就是这组电路的局部原点在板上的位置，例如 `place channels[0] at (10mm, 15mm)`。只在定义里写相对位置、未给该使用点建立锚点，不会自动把它放到板上。定义也可以只放置部分器件；“有默认布局”不等于已经完成全组布局，更不等于已经布线。

## 与 module、fn 如何分工

| 机制 | 解决的问题 | 对本课程的作用 |
| --- | --- | --- |
| 文件模块、`use`、`pub` | 定义放在哪里、名字如何导入、哪些对其他包公开 | 把电源定义放入 `power.cohdl`，导入本身不产生器件 |
| `fn` | 调用时绑定参数，展开一段可复用电路或接线 | 保留 Sonde 的 `buffered_path`、简单去耦 helper |
| `subdesign` | 给电路建立显式端口、命名层级和可复用默认布局 | 手表 USB、电源、显示供电等由顶层组合 |

此前 Sonde 的 `buffered_path` 接收顶层已有的器件和 pin，复用三路接线；本章的 `RcChannel` 自己拥有内部 r/c，顶层通过端口接线，布局时可以引用内部器件。`fn` 本来也可以在内部创建元件；真正的区别在于 `subdesign` 为外层保留了可直接引用的名字和布局入口。

这里要校正 RFC-032 的一个论据：原文把 `fn` 说成只能同包复用，但当前仓库的 `pub fn` 已可跨包解析调用。本次在临时变体的 RC 顶层加入 `passive::bulk_10u(connector.P1, connector.P5)`，实际 check 通过；这次调用没有保留在上面的五元件例子中。因此跨包复用不是 `subdesign` 独有；**显式端口、具名层级、整体布局与内部放置覆盖**才是这次最清楚的新增组合能力。[现有库函数](https://github.com/conol-ai/cohdl/blob/ab44257/lib/passive/src/circuits.cohdl)

`pub subdesign` 同样接入现有模块、包版本和 hash 锁机制；专项测试验证了跨包导入，以及非 `pub` 定义触发 E209。本次未实际向 registry 发布包。

## 编译器保护什么边界

外部可以连接 `channels[0].OUT`，可以放置 `channels[0].c`，但不能用 `net` 直接连接 `channels[0].r.B`。物理位置允许调整，电气连接通过端口约定。

实际执行的反例和结果：

| 改动 | 结果 | 含义 |
| --- | --- | --- |
| 删除第一路 OUT 的外部连接 | E1302 | required 端口只连内部不够 |
| 把端口类型从 `Pin` 改为 `Voltage` | E1303 | 当前端口不是任意数据类型 |
| 外部用 `net` 连接 `channels[0].r.B` | E010、E1301、E1302 | 电气越界被拒绝；此写法还产生解析及后续诊断 |
| 在外部 net 中增加 `channels[0].r` | E1301 | 此写法能解析，但 r 是内部器件名，不是公开端口 |
| 对端口写 `nc: channels[0].OUT` | E1306 | nc 面向器件 pin；optional 端口直接不接即可 |

普通内部器件的 part 绑定、pin 义务和 residual DRC 仍然执行。`subdesign` 不会消除 Sonde 完整模型的 E801 物料缺口。

另有几个实际边界：数组使用点不能附带一份批量端口连接块，应逐元素用 `net` 连接；`fn` 内不能实例化 `subdesign`（E1307），但子设计内部可以调用 fn；用于物理提示的 `#[bypass]` 目标必须解析到适用的真实器件 pin，不能直接用逻辑端口代替。详细规则见[错误码](https://github.com/conol-ai/cohdl/blob/ab44257/docs/error-codes.md#e13xx--typed-logical-composition-rfc-032-subdesign)。

## 实现走到了哪里

<details>
<summary>展开阅读编译器结构、审计修复和 IR 层级边界</summary>

本次 main 包含 [实现 PR #35](https://github.com/conol-ai/cohdl/pull/35) 和[修复 PR #36](https://github.com/conol-ai/cohdl/pull/36)。后者补了数组名称冲突可能丢失元件、逻辑端口被误当物理实例、未使用定义的检查、LSP 跳转和嵌套布局覆盖等回归问题，所以本课使用修复后的 `ab44257`。

| 源码入口 | 负责什么 |
| --- | --- |
| [AST 与解析](https://github.com/conol-ai/cohdl/blob/ab44257/src/ast.rs) | `SubdesignDef`、端口、使用点和布局路径 |
| [subdesigns.rs](https://github.com/conol-ai/cohdl/blob/ab44257/src/check/subdesigns.rs) | 定义检查与递归包含检测 |
| [expand.rs](https://github.com/conol-ai/cohdl/blob/ab44257/src/check/expand.rs) | 实例化、端口并网、层级路径、布局变换与覆盖 |
| [专项测试](https://github.com/conol-ai/cohdl/blob/ab44257/tests/subdesign.rs) | 连接、数组、布局、封装、跨包和审计回归 |

实现中端口先参与网络合并，然后被去掉，真实器件保留 `RcPair::channels_0::r` 这样的路径。要区分规范目标和数据结构：当前公共 `DesignIr` 仍是平面的真实实例/网络表，层级保存在实例路径里；展开器的 `SubNode` 表没有作为独立层级树输出。不能据 RFC 的“保留层级”直接推定 Explorer 已经有可折叠的子设计 UI；本次未验证 Explorer GUI。

</details>

验证结果为：24 项 subdesign 专项测试通过；完整 Rust 测试 635 通过、1 忽略，忽略项是仓库尚不存在的 `robot-dog-mainboard` 示例。三份上游板级示例 `rpi-pico2`、`sf32-miniboard`、`joint-motor-controller` 均实际 check 通过。本章 RC 例子还核对了每条网络的全部端点、BOM 器件总数、所有位置，以及重复构建的产物字节一致性。[本次记录与证据](../learning/2026-09-08-subdesign.md)

下一步可做[十路 RC 的布局覆盖与扩容实操](../course/03-rc-workflow.md)。该课使用 main `0e3d770` 并记录二进制和源码身份；下面保留两路实验的历史运行说明。

## 怎样运行本章例子

本机原目录仍在有课程改动的文档分支，本次把 main 单独检出到临时 worktree。两边版本号均为 0.6.0，因此必须看 commit，不能仅凭 `--version` 判断是否支持新语法。

```sh
export COHDL_MAIN="/private/tmp/cohdl-main-subdesign-ab44257"
export COHDL_BOOK_REPO="/Users/zhangalex/Work/Projects/cohdl"
cargo build --manifest-path "$COHDL_MAIN/Cargo.toml" --locked --offline
python3 "$COHDL_BOOK_REPO/book/examples/subdesign/verify.py" \
  --compiler "$COHDL_MAIN/target/debug/cohdl"
```

这是本次本机复现路径；临时目录以后可能被清理。其他读者应使用包含 RFC-032 的完整仓库及其 `lib/`，并把 `--compiler` 指向该仓库构建的程序。不要用旧文档分支的二进制运行本章。

## 放回手表和语言里程碑

M1 已有实现基础，课程下一步可以直接验证“共享定义、独立实例、显式端口、布局覆盖”。手表先用固定器件搭一个可解释的电源或 USB 子设计，再由主板连接；现有 [USB 前端](https://github.com/conol-ai/cohdl/blob/ab44257/examples/rpi-pico2/src/usb_frontend.cohdl)与[显示供电](https://github.com/conol-ai/cohdl/blob/ab44257/examples/sf32-miniboard/src/display_power.cohdl)可作为阅读入口，具体方案仍按手表条件选定。

M2 仍缺有界循环和计算索引；数组不会自动生成十路输出或 LED 链的全部接线。M3 可以用带单位的泛型传参，但 `DcDcConverter<12V, 5V, 500mA>` 不会凭名字就完成拓扑选择、阻容计算或负载验证。M4 的器件条件与缺失事实处理也没有因组合功能而自动完成。更新后的分工见[语言里程碑](milestones.md)。

本课先让学习者复述一件事：为什么两个 RcChannel 最后是四颗 RC 元件，为什么改变第二路电容位置不会改变第一路。学习者尚未独立完成这一步；教材和编译验证不代替学习掌握或硬件测量。
