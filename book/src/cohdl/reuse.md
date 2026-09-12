# 模块、fn、subdesign 与数组

> **定位**：区分文件组织、复用定义和实际电路实例。前置：[连接模型](connections.md)；配套实践：[第 3 课](../course/03-functions.md)。原例基线 CoHDL 0.6.0 / `9ac0e3c`；subdesign 新增内容按 main `ab44257` 验证。

手表的电源、显示和传感器由不同功能区负责，这是关注分离。同一个按键或去耦电路出现多次，则是复用。两者可以一起使用，但不是一个问题。

## 文件模块组织名字

现有模块系统使用文件树、命名空间、`use` 导入与 `pub` 可见性。把电源定义放入独立文件，有助于隔离实现；导入这个文件不会自动增加 PCB 元件。

Explorer 可以按功能显示分组，这种分组不改变电路连接。KiCad 页面和显示区域也不能替代接口语义。

## fn 展开真实电路

下面直接包含仓库 `lib/passive/src/circuits.cohdl`，它属于 passive 包，是库文件片段的完整展示；不是一个可单独 check 的顶层设计。

```cohdl
{{#include ../../../lib/passive/src/circuits.cohdl}}
```

调用 `decoupling_100n` 时，`vdd` 和 `gnd` 绑定到实际 pin；内部实例 `c` 成为该次调用的一颗真实电容。调用两次会有两颗电容，而不是两条指向同一颗电容的引用。

这个 helper 固定了容值、额定值和封装。用它之前仍要核对目标器件的要求。`#[bypass]` 保存去耦关联和物理提示，不证明电容实际放在了正确距离和回流路径上。

## 数组表达多个实例

当前语法采用 `inst name: [Device; N]`，通过 `name[0]` 等下标引用元素。N 和索引当前是字面量，裸数组名不能当作一个普通实例使用。每个元素都是真实实例，应分别满足连接义务。

数组声明本身没有生成相邻连接。当前范围/list fan-out 只用于 net 成员，不能把它当成一般的循环或算术索引。后续 `for` 的目标与边界见 [M2](milestones.md)。

## subdesign 保留端口和可放置的层级

main 已在 RFC-032 中实现 `subdesign`。它把真实内部实例封装在显式端口之后，支持嵌套、泛型、数组实例和跨包导入；外层能整体摆放，也能用 `place power.reg` 覆盖一个内部器件的位置。完整可运行的两路 RC 例子及边界见 [subdesign 专章](subdesign.md)。

`fn` 继续用于参数绑定与内联展开，但没有可供源代码寻址的调用结果对象。其展开路径使用全设计调用计数；subdesign 的路径则包含显式使用点名称。相同源码输出确定，不等于重命名、移动层级或插入调用后旧位号必定不变；修改后仍应查看 `design.lock`。

已有 `pub fn` 也能跨包调用，不能把这个能力说成 subdesign 独有；新概念的直接价值是端口边界、具名层级与可复用布局。

对手表的实际作用是：先保持通用电源定义独立，用 subdesign 端口由顶层组合，再记录参数、契约和工具显示仍有哪些缺口。[第 4 课](../course/04-first-board.md)把它落实到小板，[第 5 课](../course/05-watch-prototype.md)再扩展到手表。

来源：[FnDef 与 Stmt](https://github.com/conol-ai/cohdl/blob/9ac0e3c/src/ast.rs#L878)、[函数展开与绑定](https://github.com/conol-ai/cohdl/blob/9ac0e3c/src/check/expand.rs)、[数组验收](https://github.com/conol-ai/cohdl/blob/9ac0e3c/tests/inst_array.rs)。变化发生后更新 [版本记录](../appendix/sources.md)。
