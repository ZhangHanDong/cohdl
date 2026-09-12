# 2026-09-08：拉取 main，验证 subdesign

用户要求拉取 main 新增的 subdesign 并解读。当前工作目录在 `docs/proof-gated-agent-harness-rfc`，有尚未提交的 Book 等改动，HEAD 为 `9ac0e3c`。

已执行 `git fetch origin main`，远端跟踪分支更新到 `ab44257`，再以 detached worktree 检出到 `/private/tmp/cohdl-main-subdesign-ab44257`。没有在原文档分支执行合并、覆盖已有改动或安装替换全局编译器。main worktree 在阅读和验证后保持干净。

阅读 RFC-032、规范、实现、合规记录和 PR #35/#36 对应提交；课程详解在 [subdesign 章节](../cohdl/subdesign.md)。Accepted 的 RFC-032 是 typed logical composition；本分支旧 `rfc-032-proof-gated-agent-harness.md` 只是历史提案，编号相同不代表被接受。

## 本次实际执行

- `cargo test --test subdesign --offline --locked`：24 项通过。
- `cargo test --offline --locked`：首次在沙箱内因本地模拟注册表监听端口被拒而停止；获得沙箱外执行许可后重新运行，exit 0，635 通过、1 忽略。忽略项为尚未入库的 robot-dog-mainboard 示例。[完整成功日志](evidence/2026-09-08-subdesign-cargo-test.txt)
- main 的 `rpi-pico2`、`sf32-miniboard`、`joint-motor-controller` 各自 `check --json`：exit 0、pass、无诊断。这是编译检查，没有执行硬件测试。
- 新建独立的 `book/examples/subdesign/`，并用新 main 二进制运行 `verify.py`：check/build 成功；5 个真实元件、5 条网络，BOM 合并为 3 行；5 个器件位置和全部网表端点符合预期；网表/BOM/layout/封装/锁文件重复构建字节一致。
- 五组失败用例实际被拒绝：漏接 required 端口 E1302；电气越界 E010/E1301/E1302；把内部器件名用作端口 E1301（语法合法，单独验证边界检查）；非 Pin 端口 E1303；nc 端口 E1306。额外正例 `passive::bulk_10u` 跨包调用 check 通过。[原始 RC 结果](evidence/2026-09-08-subdesign-results.json)

## 留下的判断

`subdesign` 已可承担 M1 的端口组合、独立实例、层级布局实验，课程不再把它只列为未来提案。M2 循环、M3 参数配方/选料、M4 条件契约仍需各自补齐证据和能力。

两个规范解读应以源码校正：`pub fn` 已支持跨包调用；当前 IR 保留的是子器件层级路径，没有单独输出 SubNode 层级树。Explorer 可视交互未验证。端口类型目前只有 Pin，required 连接义务不等同于电压范围或方向类型。

本次没有改动 Sonde 原始模型、KiCad 工程或硬件，没有证明 RC 响应或可投产。学习者未复述或独立操作，新例子只记为已完成教材与编译验证。
