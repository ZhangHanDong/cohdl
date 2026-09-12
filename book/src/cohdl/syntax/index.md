# CoHDL 语法总览：每个关键字管什么

> **定位**：系统列出 CoHDL **当前已实现**的语法，说明每一项在编译器里的作用，以及它对应的 PCB 概念。前置：[连接模型](../connections.md)。基线：CoHDL 0.7.0，main `0e3d770`；本组章节的运行示例在仓库 `book/examples/syntax-tour/`，已用该版本 `check`、`fmt --check` 和 `build` 通过。提案中的语法（循环、常量、表达式）不在本组章节里，见[语言设计研究](../programming-proposal.md)。

一个 `.cohdl` 文件里有九种顶层声明，外加 `use` 导入。前六种描述“世界上有什么器件、怎么分类、买哪一款、焊盘长什么样”，它们本身不产生任何元件。元件只从 `inst` 产生：写在 `design` 里的直接算数，写在 `fn` 或 `subdesign` 里的在被调用、被实例化时展开，最后都由顶层 `design` 汇总装配。

| 关键字 | 做什么 | 对应的 PCB 概念 | 规范 | 章节 |
| --- | --- | --- | --- | --- |
| `trait` | 声明一类元件必须有的引脚角色、规格字段和位号前缀 | 元件类别（电阻、电容、二极管、IC） | RFC-003、005 | [单位与器件](units-devices.md) |
| `device` | 声明一个器件的物理引脚、引脚角色、规格和封装变体 | 数据手册里的引脚表 | RFC-002、007、008 | [单位与器件](units-devices.md) |
| `impl` | 断言某个器件满足某个 trait，名字不同时给出映射 | 库作者的分类判断 | RFC-003 | [单位与器件](units-devices.md) |
| `part` | 把一个完全具体的器件绑定到厂商和 MPN，并指定封装 | BOM 行、AVL | 临时语法 §2、RFC-017 | [物料与封装](parts-footprints.md) |
| `pad` | 一个可复用的焊盘：形状、尺寸、铜层、工艺、钻孔 | 焊盘、阻焊、钢网开口 | RFC-018 | [物料与封装](parts-footprints.md) |
| `footprint` | 引用焊盘并定位，加上外框、丝印、定位孔、开窗 | 封装、丝印、NPTH | RFC-018、022、023、025、031 | [物料与封装](parts-footprints.md) |
| `fn` | 一段可参数化、调用即展开的电路片段 | 反复出现的小电路 | RFC-006、007、028 | [复用](composition.md) |
| `subdesign` | 带端口、带默认布局、可整体摆放的电路单元 | 功能模块、子板级复用 | RFC-032 | [复用](composition.md) |
| `design` | 顶层：实例、网络、不接、布局事实 | 一块板的原理图与放置 | RFC-002、004、024 | [设计与网络](design-nets.md) |

`design`、`fn`、`subdesign` 的**体内**再用四个语句：`inst` 声明实例，`net` 声明连接，`nc` 声明显式不接，`layout { … }` 声明放置和布局约束。属性 `#[...]` 挂在声明前面，给工具附加事实；写对了不改变电气判定，写错单位或引用会和其他语句一样报错。

想先看全貌，[语言项与 PCB 事实的完整对照](mapping.md)把每一项、它建模的电路事实、检查时机和边界放在一张表里，也包括 M2 提案和语言有意不管的部分。

## 阅读顺序

按编译器判定的顺序读最省力，这也是[诊断指南](../diagnostics.md)里错误码的分层：

1. **名字和单位**：`use`、模块路径、11 种单位类型。名字找不到是 E2xx，单位不对是 E1xx。名字来自哪个包、版本怎么锁，见[包与注册表](packages.md)，E11xx 和 E12xx 在那里。
2. **器件和类别**：`device`、`trait`、`impl`、泛型、变体。类别不满足是 E4xx。
3. **连接义务**：`inst`、`net`、`nc`、数组与扇出。必需引脚没交代是 E701。
4. **四条残余 DRC**：超压、极性、单驱动、多驱动，D001 到 D004。
5. **物料与封装**：`part`、`pad`、`footprint`。`build` 时没绑定物料是 E801，焊盘与引脚不一致是 E805 到 E807。
6. **布局事实**：`layout`、`place`、约束、物理属性。它们的语法、引用和单位仍会被检查（E10xx、E110），但合法的布局语句从不改变前五步的电气结论、位号和网表字节，只进板文件和 CSV。

## 这组章节不做什么

- 不重复[连接模型](../connections.md)对网络的解释，也不重复 [subdesign 专章](../subdesign.md)的两路 RC 实验；这里只给语法和边界，链接过去。
- 介绍当前已实现的语法，并分别标明 Accepted RFC、provisional 临时语法和已记录的实现偏离；“已实现”不等于“已接受”。CoHDL 的 `for`、`const`、`n + 1` 等 M2 提案写法另列在[语言设计研究](../programming-proposal.md)。
- 不把“编译通过”写成“电路正确”。每一节末尾都说明编译器在这一步**没有**检查什么。

## 运行示例

从仓库根目录：

```sh
cargo run -- check book/examples/syntax-tour
cargo run -- fmt book/examples/syntax-tour --check
cargo run -- build book/examples/syntax-tour --emit kicad_pcb
```

完整文件和构建产物的逐项解释见[最后一节](tour.md)。
