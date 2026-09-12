# 课程编译器准备：独立工作树

十路 RC 课使用 CoHDL 0.7.0、源码提交 `0e3d7705ab393a5c64c20836540ad1bc2e49898e`。本页把编译器准备集中在一次操作中；之后回到课程，专注电路的连接和修改。

需要 Git、Rust/Cargo 和 Python 3.11 或更新版本。RC 验证器使用 Python 标准库 `tomllib`，因此其要求高于 Book 链接检查的 Python 3.9。首次构建可能需要联网下载 Cargo.lock 中的依赖。

## 第一次准备

在包含本书与 RC fixture 的 CoHDL 仓库根目录执行。所有路径都从你的仓库位置推导；同级的 `cohdl-compiler-0e3d770` 将保存课程编译器源码。Git worktree 共用提交对象，但有独立文件和 HEAD，因此当前分支的未提交修改可以保留。

```sh
export COHDL_BOOK_REPO="$PWD"
export COHDL_MAIN="$COHDL_BOOK_REPO/../cohdl-compiler-0e3d770"
export COHDL_COMPILER="$COHDL_MAIN/target/debug/cohdl"
git -C "$COHDL_BOOK_REPO" worktree add --detach "$COHDL_MAIN" 0e3d7705ab393a5c64c20836540ad1bc2e49898e
git -C "$COHDL_MAIN" rev-parse HEAD
cargo build --manifest-path "$COHDL_MAIN/Cargo.toml" --target-dir "$COHDL_MAIN/target" --locked
"$COHDL_COMPILER" --version
```

逐条确认成功后再继续：HEAD 应等于上面的完整提交，版本应为 `cohdl 0.7.0`。版本号用于识别版本，完整提交用于固定本课实际源码；不以工作树目录名推断版本，也不假定本地已有同名 release tag。

如果 worktree 命令提示找不到提交，先在课程仓库运行 `git fetch origin main`，再重试。若目标目录已存在，不要覆盖它：确认它是否是之前创建的课程工作树，再按下一节复用；无关目录应保留，并为 `COHDL_MAIN` 选择另一个未占用路径，同时更新 `COHDL_COMPILER`。

任何命令失败都先停止后续步骤，保留报错供排查。`--locked` 防止构建静默改写 Rust 依赖锁；这里允许首次下载依赖，不要求读者预先具备离线缓存。

## 下次打开终端

回到同一本书的仓库根目录，重新执行上面三条 export，再执行 `rev-parse HEAD`、build 和 `--version`；已经创建的工作树不必再 add。只重新赋值不会切换当前分支。

保持这个终端打开，返回[十路 RC 实操](../course/03-rc-workflow.md#在临时副本里运行)。课程中的 `RC_LAB` 是可重复创建的练习目录；编译器工作树可以跨课保留。

## 记录证据时

本课验证命令把 `COHDL_COMPILER` 与 `COHDL_MAIN` 一并交给验证器，记录可执行文件哈希、源码提交、工作树改动及输入哈希。必须先从该工作树构建；仅给一个源码路径不能证明任意二进制由它产生。

历史报告保存产生时的信息。缺少来源字段的早期报告不会事后补写；当前 RC 基线引用 `book-results-2026-09-10.json`，它含已核对的编译器来源。新的运行结果保存在自己的 `RC_LAB/verification.json`。

本页和 RC 课程的命令已于 2026-09-10 在新的测试副本中顺序运行，见[本次记录](../learning/2026-09-10-book-portable-setup.md)。本机已有依赖缓存，首次联网获取依赖不在这次验证范围内。
