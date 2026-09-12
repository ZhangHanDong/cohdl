# 连接模型：从实例到焊盘

> **定位**：把 PCB 术语映射到 CoHDL。前置：[第 1 课](../course/01-read-board.md)，或已有基本认图经验。当前能力；基线 CoHDL 0.6.0 / `9ac0e3c`。

一个设计先说明有哪些元件、它们的哪些引脚应该相连，再把逻辑引脚映射到实际封装的焊盘。画在纸上的线与板上的铜几何，不必是一一对应的形状。

| CoHDL 对象 | 表达什么 | 与 PCB 的关系 |
| --- | --- | --- |
| `device` | 引脚、规格和类型上的器件模型 | 还需要具体 part 和封装才能制造 |
| `part` | device 的具体实现与采购信息 | 把型号、规格、封装联系起来 |
| `inst` | 设计里的一次实际使用 | 构建时对应一个实际元件与位号 |
| `pin` | 器件逻辑端点及其物理编号 | 可以映射到一个或多个 pad |
| `net` | 应相连的端点集合 | 需要铜、过孔、铜区等实现 |
| `nc` | 显式声明一个 pin 不连接 | 可解决 required pin 的显式处理义务，不证明断开在电气上合理 |

一条逻辑 pin 可能对应多个物理焊盘，同一编号还可能有散热焊盘/过孔等重复实体。因此不能总按“一个名字等于一个图形”比较电路和 PCB。

## 一个可执行的最小例子

下面是书中 `book/examples/connections.cohdl` 的完整内容，构建书时直接包含该文件，避免正文与例子漂移。

```cohdl
{{#include ../../examples/connections.cohdl}}
```

它声明两组端点：`input` 和 `output`。`SIGNAL` 连接各自的 A，`RETURN` 连接各自的 B。这里的 ContactPair 是教学模型，没有选定真实连接器、part 或 footprint，适合检查连接语义，不用于制造。

从 CoHDL 仓库根目录运行：

```sh
cargo run -- check book/examples/connections.cohdl --no-std
```

本例不引用标准库，故显式使用 `--no-std`。正常结果为退出码 0。将文件复制到练习目录，删除 `net RETURN` 那一行后重新检查，会暴露两处 required pin 未处理；诊断应指向 `input.B` 与 `output.B`，代码为 E701。恢复这条连接，再运行检查。

当前 RFC-002 要求 required pin 明确出现在 net 或 nc 中，不能同时出现，也不能两者都没有。`nc` 在语法上能够解决这项义务，但不自动证明真实器件允许断开这个 pin；本练习的原始意图是连接 RETURN，所以修复时恢复该网。器件工作条件另见 [电源与契约](../course/06-power-contracts.md)。

这里的 `input` 是实例名；`A` 是逻辑 pin 名；数字 `1` 是该模型的物理 pin 编号。reference designator 如 J1/R1 则由构建与 `design.lock` 管理，不能把所有名字混为同一身份。

## 单位也是类型

在需要 Voltage 的位置，`3.3V` 与裸数字 `3.3` 不是同一种值；Resistance 使用 `ohm` 等规定字面量。编译器不会把缺单位的数字猜成设计者想要的单位。由此可以在生成 PCB 前发现一类结构错误。

但 `net VDD [3.3V]` 是源代码提供的电压事实，并不是编译器求解完整电路得到的测量结果。有关电源、负载和条件，见 [第 6 课](../course/06-power-contracts.md)。

## 回到实际 PCB

第 1 课中，你追踪 R1 pin 1 到 D1 pin 2。CoHDL 同样需要保留这种端点关系；原生 PCB emitter 再把 net 写到对应 pad 上。完整铜连接仍需实际布局布线与检查，见 [构建到 PCB](pipeline.md)。

练习完成后，将报错与修复写进 [记录](../learning/template.md)。语言依据：[语言规范](https://github.com/conol-ai/cohdl/blob/9ac0e3c/docs/design/10-language-specification.md)、[pin 展开](https://github.com/conol-ai/cohdl/blob/9ac0e3c/src/check/expand.rs)、[错误代码](https://github.com/conol-ai/cohdl/blob/9ac0e3c/docs/error-codes.md)。
