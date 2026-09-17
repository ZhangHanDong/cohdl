# 维护循环(内环 master 每轮唤醒执行)

本仓库当前战役:实现 RFC-033(`docs/design/rfc-033-parameterized-circuit.md`)。
唯一事实来源是实施计划 `docs/superpowers/plans/2026-09-15-rfc-033-parameterized-circuit.md`;
黑板条目只引用「计划任务号 + 验收断言」,不复制计划内容,有分歧以计划为准、以 RFC 为上。

每次维护唤醒依次执行,全部完成才结束本轮:

1. 读 `.octos/OUTER_LOOP_REVIEW.md`(外环审查黑板)。
2. 若存在**未 ACK 的条目**(无 `ACK(` 定式行):取编号最小的一条,按其内容执行到完成,
   然后在该条目下补一行 v1 定式 ACK:`ACK(done|wontdo|blocked): <说明>`。
   done 必须附:commit hash、逐字跑过的验证命令与结果、R2 声明
   (`verified` / `partially-verified: <跑了什么>` / `unverified: <原因>`)。
3. 无未 ACK 条目:检查在途分支与测试基线,如实记录状态后结束本轮。

工作纪律(本仓库专用):

- 分支:所有工作落在主工作树的 `feat/rfc-033-parameterized-circuit`,基于 origin/main。
  不切分支、不开新分支、不 rebase。**只 commit,不 push**(推送权在外环,复验后代推)。
- 每个任务先写失败的测试再实现(计划里给了测试代码),测试文件名按计划。
- 验证命令(逐字,每条 ACK 必附结果):
  `CARGO_INCREMENTAL=0 cargo test --all-targets`
  `cargo run -- fmt lib examples book/examples --check`
  `cargo build --manifest-path explorer/extractor/Cargo.toml`
  任务涉及 docs/error-codes.md 时:`cargo test --test error_registry`。
- 编译一律 `CARGO_INCREMENTAL=0`;并发编译全机 ≤2;测试可加 `--test-threads=8`。
- 只 `git add` 自己改的文件,禁止 `git add -A`。不要碰 `book/`、`.superpowers/`、
  `.octos/`(黑板除外)、`docs/reviews/`;来源不明的脏文件保留并在 ACK 里报告。
- 硬约束:编译器零外部依赖;输出相邻处不得出现 HashMap 迭代;现有 fixture 字节不变;
  每个诊断有稳定码和精确 span,新码必须登记到 `docs/error-codes.md`。
- 同一目标反复试错超过 30 分钟,写 `ACK(blocked)` 说明卡点并等外环图纸,不要硬磨。
- 提交信息末尾加:`Co-Authored-By: octoscode inner loop <noreply@octos.local>`。
