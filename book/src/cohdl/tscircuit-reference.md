# 从 tscircuit 看 CoHDL 应有的可编程能力

> **定位**：比较参数计算、重复构造、组件组合和检查的职责，为 CoHDL 可编程能力模型提供参考。前置：[语言设计原则](language-principles.md)。本文是源码观察与设计判断；CoHDL 的规范依据以最新 main 为准，Accepted RFC-032 subdesign 已恢复为 M2 的依据。

## 先给结论

tscircuit 值得参考的核心是：用普通程序处理参数和数据，再把结果构造成可组合的电路对象，最后由电路工具处理连接、几何与检查。CoHDL 需要先定义对应的能力模型，再决定哪些表达式、循环或组合语法能够实现它。

这个判断不要求 CoHDL 使用 TypeScript、React 或 JSX，也不使 tscircuit 的 subcircuit 规则自动成为 CoHDL 规范。借鉴机制之前，仍须明确输入、对象身份、连接权限、诊断和可复现性。

## 本地仓库是什么

用户提供的 `/Users/zhangalex/Work/CoHDL/tscircuit` 位于提交 `7afedf434c3fdb7528191db84d9571bc9bbcce67`，包版本 `0.0.2478`。它主要是聚合入口：`index.ts` 导出 core、eval 与 Circuit JSON 类型，`cli.mjs` 委托 CLI 包；入口测试分别演示对象式构建和字符串代码求值。[入口源码](https://github.com/tscircuit/tscircuit/blob/7afedf434c3fdb7528191db84d9571bc9bbcce67/index.ts)、[对象式构建测试](https://github.com/tscircuit/tscircuit/blob/7afedf434c3fdb7528191db84d9571bc9bbcce67/tests/smoke1.test.tsx)、[字符串求值测试](https://github.com/tscircuit/tscircuit/blob/7afedf434c3fdb7528191db84d9571bc9bbcce67/tests/smoke2.test.tsx)

本地没有安装 node_modules。为核对真实实现，本次在临时目录取得 core 提交 `e1f5a0edc482897d67969e694c8fbe3e18d4f339`，其声明版本 `0.0.1875` 与本地 bun.lock 中的 core 版本一致。这是明确固定的源码阅读基线；未验证该源码与 npm 发布包逐字一致，也未安装整个依赖树或执行上游测试。

## 程序怎样变成电路

```mermaid
flowchart LR
    A[参数与普通程序计算] --> B[React 元素树]
    B --> C[电路类实例及父子关系]
    C --> D[分阶段构建与检查]
    D --> E[Circuit JSON 中的设计与错误记录]
    E --> F[原理图 PCB 和其他使用方]
```

`IsolatedCircuit.add` 接收 React 元素或已有电路对象。React 元素进入自定义 reconciler；`createInstance` 从 catalogue 查找 resistor、capacitor 等对应类并构造实例，添加子元素时建立父子关系。用户函数与数组运算先产生这些元素，后续阶段处理实际电路。[add 与 render](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/IsolatedCircuit.ts#L117)、[React 到实例的转换](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/fiber/create-instance-from-react-element.ts#L60)

阶段表包含 source 对象、端口匹配、连接检查、原理图、PCB 放置与布线、物理检查等步骤。`renderUntilSettled` 等待异步工作结束，`getCircuitJson` 返回数据库条目；这两个 API 本身不能被当作“所有电气和制造检查通过”的证据。[阶段表](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/components/base-components/Renderable.ts#L13)、[输出接口](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/IsolatedCircuit.ts#L204)

这里可以区分三种东西：普通数据值、电路构造结果，以及构造后的检查结果。对 CoHDL，这是比先选 `for` 拼写更靠前的设计问题。

## 重复构造不只等于重复连线

RP2040 测试项目用同一份元件名称列表创建六颗去耦电容。以下为源码原样节选，未在本次运行：

```tsx
    {["C12", "C14", "C8", "C13", "C15", "C19"].map((cName) => (
      <capacitor
        name={cName}
        capacitance="100nF"
        schOrientation="vertical"
        footprint="0603"
        connections={{
          pin2: "net.GND",
        }}
      />
    ))}
```

列表决定要构造哪些实例；map 中同时声明元件属性和一端接地。该文件前面的 RP2040 connections 把六个 IOVDD 分别连到相应电容 pin1 与 V3_3，完整连接需要结合两处阅读。它不是只在六个预先声明的器件上重复调用接线操作。[完整 RP2040 电路](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/projects/seveibar__rp2040-zero/lib/RP2040Circuit.tsx#L1)

另一份测试辅助文件把一段电路封装成 `KicadSubcircuit` 函数组件，传入 name 与其他 props；再用 `Array.from` 和 map 生成多个使用点。参数、重复和嵌套组合在同一种宿主语言中完成。[函数组件与重复实例](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/subcircuits/subcircuit-caching-benchmark-fixtures.tsx#L10)

**对 CoHDL 的设计判断：** 可编程需求应覆盖“根据参数构造实例及其连接”，不能只用 LED 相邻索引说明全部能力。当前 [M2 工作稿](programming-rfc-zh.md)已按 2026-09-12 的用户决定选择 A（操作循环，保留已有 fn 构造能力），B（额外允许直接迭代局部声明）延后；见[范围记录](../learning/2026-09-12-m2-scope-a.md)。上面的例子传入现成名称列表，并未使用字符串插值。它展示了重复构造和名称引用如何配合，不能单独证明 B 的私有局部对象能够满足循环外的编辑任务，也不能据此否定局部构造在其作用域内的价值。CoHDL 的持久身份和访问契约仍须独立论证。

## 组合边界必须单独判断

tscircuit 的 `selectOne` 在当前作用域查找失败后，可以把显式路径拆成子电路名与剩余路径，再递归进入子电路查找；Trace 的端口解析实际调用这个接口。因此，`.subcircuit1 .R1 .pin1` 这种路径具有访问内部引脚的实现通路，其名称隔离并不等于“只允许通过公开端口访问内部电路”。[选择器递归实现](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/components/base-components/PrimitiveComponent/PrimitiveComponent.ts#L1303)、[Trace 端口解析](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/components/primitive-components/Trace/Trace__findConnectedPorts.ts#L40)

对应测试使用了跨层完整路径，但正例只断言 render 不抛异常，没有断言最终连接关系；这里把它作为使用意图的例子，结论依据上面的实现路径。本次没有执行该测试。[作用域测试](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/subcircuits/subcircuit1-isolated-refdes.test.tsx)

因此，对 CoHDL 要分别讨论：复用定义、命名作用域、外部可连接的接口、布局分组。它们可以组合，但不能看到一个叫 subcircuit 的构造，就假定四者具有我们想要的同一边界。CoHDL 的 RFC-032 已规定端口、层级与布局边界；借鉴此项目时应先与它核对，M2 的新增组合行为另作明确论证。

## 检查模型能借鉴什么

电路检查并没有被 TypeScript 的类型系统全部代替。未知元素会在 reconciler 查 catalogue 时报错；无法匹配的引脚选择器可以产生 Circuit JSON 错误记录。另一个测试直接调用 PCB 走线重叠检查，再把错误写回数据库以供展示。[未知元素测试](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/fiber/unsupported-component-error.test.tsx)、[物理检查测试](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/features/drc-error-detection.test.tsx)

部分无效输入会变成占位对象或警告，以保留可显示的结果。例如非法 pin label 会被过滤并记录 warning。这可以帮助交互预览；它不能直接成为 CoHDL 成功构建的规则。[错误占位路径](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/lib/fiber/create-instance-from-react-element.ts#L78)、[pin label 测试](https://github.com/tscircuit/core/blob/e1f5a0edc482897d67969e694c8fbe3e18d4f339/tests/components/normal-components/chip-invalid-pin.test.tsx)

**对 CoHDL 的设计判断：** 保留可供编辑器查看的部分模型，与允许制造输出成功，需要分别定义。还要明确源表达式怎样对应每个生成实例和诊断；TSX 能执行、JSON 能生成，均不等于电路满足已建模义务。我们也没有从这些样例证明 tscircuit 具有与 CoHDL 相同的单位静态检查、字节确定性或 design.lock 身份契约。

09-11 的进一步源码核对：[重名路径](https://github.com/tscircuit/core/blob/c298605b2779793876f20f1139a2ebd024a1d7f3/lib/components/base-components/NormalComponent/NormalComponent.ts#L236)会写入错误并把冲突元件标记为移除，而另一些[渲染路径](https://github.com/tscircuit/core/blob/c298605b2779793876f20f1139a2ebd024a1d7f3/lib/components/base-components/Renderable.ts#L527)直接抛出异常。因此“所有错误都能继续渲染”和“有错误记录就无法字节确定”都不能从该机制推出。CoHDL 要守住的是成功判定：可以显示失败状态下的部分模型，但成功构建不能悄悄漏掉必需元件。见[本轮复核](../learning/2026-09-11-tscircuit-rfc-review.md)。

## 怎样影响这次 RFC

| 需要先定义的能力 | tscircuit 提供的参考 | CoHDL 要独立决定的契约 |
| --- | --- | --- |
| 参数与值计算 | 函数 props、普通数组与表达式 | 哪些值域、单位运算、默认值和输入来源可用 |
| 电路构造 | 程序返回元素，元素构造真实对象 | 构造与普通求值的区别；直接生成与 helper 展开的关系 |
| 重复与配置 | map、函数组合及宿主语言能力 | 有限性、条件选择范围、空分支检查；不能默认接受整个宿主语言 |
| 组合与连接 | 组件树、命名作用域、选择器 | 哪些对象可访问；连接授权与布局范围分别定义 |
| 身份与可检验性 | 名称、对象记录、错误记录和测试样例 | 编辑后的身份稳定、源码来源、检查阶段和成功判定 |

同日重写稿已用“十颗 LED 级联”“每路包含阻容的 N 通道电路”“复用定义内再组合重复电路”共同检验能力模型，并将类型、值、组装检查连接到详细语义。参数化电源配方还需要器件事实与适用条件；tscircuit 的通用程序能力本身不证明已经解决 CoHDL M3/M4 的需求。

## 其他开源参考放在哪里

按与当前问题的贴合程度选择研究对象，不以星数推断设计质量：

| 项目 | 可参考的机制 | 本次证据范围 |
| --- | --- | --- |
| [Polymorphic Blocks](https://github.com/BerkeleyHCI/PolymorphicBlocks) | Python 电路生成器、抽象器件与 refinement；文档包含根据负载选择电感/电容的转换器生成器，适合研究 M2 怎样承接 M3/M4 | 官方 README；研究性质，未运行 |
| [atopile](https://github.com/atopile/atopile) | 声明式模块/接口、带单位与容差的约束、参数选料，适合比较“求一个具体值”与“约束一个可行范围” | 官方 README；未运行，不推断其语法支持与 CoHDL 相同的循环 |
| [SKiDL](https://github.com/devbisme/skidl) | Python 计算、参数化子电路、网表和 ERC，适合研究通用宿主语言与电路库的职责分界 | [官方文档](https://devbisme.github.io/skidl/)；未运行 |

这些项目是设计参考，不是 CoHDL 的规范依据。对 tscircuit，当前最值得进一步比较的是“普通计算 → 电路构造 → 检查”的分工及其组合成本。

## 版本与学习状态

本次阅读于 2026-09-10，入口包和 core 的提交分别固定如上。读取了源码与上游测试文本，没有执行上游测试、下载物料或产生 PCB/制造结果；没有修改用户的 tscircuit 工作树。可编程模型与语法仍是讨论材料，未记录学习者已掌握。实际记录见[本次学习记录](../learning/2026-09-10-programmability-references.md)。

2026-09-11 补查了 core v0.0.1875 标签提交 `c298605b2779793876f20f1139a2ebd024a1d7f3`。它与此前固定提交不同，但本轮涉及的六个源码/测试文件逐字节一致；该比较不证明整个仓库或 npm 发布包等价。本轮仍为源码阅读，未运行上游测试。入口包与 core 是不同包，不能用 0.0.2478 与 0.0.1875 的数字差推断后者落后了多少版。
