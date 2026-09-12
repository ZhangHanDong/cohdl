# 读懂诊断：从报错到验证边界

> **定位**：用一条报错找到电路中的具体问题，修复后重新核对意图。前置：[连接模型](connections.md)。例子来自[十路 RC 实操](../course/03-rc-workflow.md)，基线为 CoHDL 0.7.0 / `0e3d770`，不是未来 M2 语法。

## 先读位置与对象，再读修复建议

删除 RC 定义里电容 B 到 GND 的连接后，实际诊断包括：

```text
E701: required pin `RcWorkflow::channels_0::c.B` is unresolved:
add it to a `net` or explicitly mark it `nc`
```

`channels_0` 是第一路，`c.B` 是电容的真实引脚。十次复用会生成十套引脚义务；共享定义的一个错误会影响多路。JSON 的 `primary` 指向本地实例声明，`secondary` 指向 passive 包里把 B 声明为 required 的位置。诊断在回答“哪个实例欠了哪项义务”。

帮助文字列出了合法处置方式；电路意图决定采用哪一种。本例应恢复接地。把 B 标成 nc 可能让结构检查通过，但没有恢复原来的 RC 连接。

## 三类问题，用实际例子区分

| 问题 | 本课实际错误码 | 下一步 |
| --- | --- | --- |
| 值或单位不适合这个位置 | 坐标写 `10V` 得到 E1007 | 阅读“期望 Length、实际 Voltage”，改回带 mm 的长度 |
| 引用不存在 | 十路中的 `channels[10]` 得到 E202 | 确认从 0 开始的实际范围 0–9 |
| 连接义务未完成 | E701 物理引脚；E1302 必需端口 | 确认内部连接与外部接线分别完成 |

单位错误并不一律是 E110。通用单位错误、泛型实参错误与布局字段可能走不同诊断入口；[错误码注册表](https://github.com/conol-ai/cohdl/blob/0e3d770/docs/error-codes.md)给出细分含义。错误码稳定地标识问题，消息与位置提供本次上下文。删除一个外部连接同时得到 E1302/E701 是两个具体义务，不表示要随意补两条网。

## 一次只修一个原因

```mermaid
flowchart LR
    A[恢复已知正确源码] --> B[单独修改]
    B --> C[读取 code 和位置及消息]
    C --> D[按电路意图修复]
    D --> E[重跑 check]
    E --> F[核对网络和需要的物理检查]
```

在 RC 课配置好变量后执行：

```sh
"$COHDL_COMPILER" check "$RC_LAB" --json
```

查看 `verdict`、`diagnostics[].severity/code/message/primary/secondary/help`；单独有 warning 不一定导致失败。先修能解释后续错误的根因，再检查剩余诊断。不能把错误码数字大小当作编译器执行顺序。

## 没有错误，还知道什么

本课故意短接两个输出，check 仍通过，但网络分区从 21 变成 20。这是“通道独立”的意图对照失败，普通引脚类型并没有表达这项约束。

CoHDL 现有四条残余 DRC 也有各自前提：D001 根据已声明的网电压与器件额定电压检查超压，D002 检查建模的极性关系，D003 警告只有一个驱动引脚的孤立网，D004 检查多个驱动端冲突。它们不是完整物理电路验证，也不等同于 KiCad 的间距与走线 DRC。[实现与前提](https://github.com/conol-ai/cohdl/blob/0e3d770/src/drc.rs)

源码入口是 [diag.rs](https://github.com/conol-ai/cohdl/blob/0e3d770/src/diag.rs)、[expand.rs](https://github.com/conol-ai/cohdl/blob/0e3d770/src/check/expand.rs)与错误码注册表。想做语言开发，先把本课某个最小失败源映射到实际检查分支，再读相应测试；不要为了消除报错而削弱义务。

**版本与验证：** 上表与 E701 来源位置均取自[本次真实实验](../learning/2026-09-10-book-review-repair.md)。编译成功之后的文件、板与测量关系继续读[从 check 到实际 PCB](pipeline.md)。
