# 语言设计与 RFC 记录

这里保存设计讨论和评审发生时的状态。阅读当前方案请先看[M2 中文导读](../cohdl/programming-proposal.md)及[英文工作稿](../cohdl/programming-rfc-en.md)；历史记录中的推荐不自动成为当前决定，翻译和评审也不代表提案已接受。

## 产品边界与语言原则

- [2026-09-08：翻译与解读设计总览](2026-09-08-design-philosophy.md)
- [2026-09-08：产品与基础设施的边界](2026-09-08-product-boundary.md)
- [2026-09-09：沉淀语言设计原则与哲学](2026-09-09-language-principles.md)

## M2 提案与比较材料

- [2026-09-08：起草 M2 RFC](2026-09-08-m2-rfc.md)
- [2026-09-09：按设计原则再审 M2 RFC](2026-09-09-m2-review.md)
- [2026-09-09：M2 RFC 中文全文](2026-09-09-m2-chinese.md)
- [2026-09-10：校正 RFC 依据，研究 tscircuit](2026-09-10-programmability-references.md)
- [2026-09-10：重写 M2，并连接类型系统](2026-09-10-m2-rewrite.md)

## 依据核对与后续修订

- [2026-09-10：核对最新 main，恢复 RFC-032 依据](2026-09-10-main-rfc032.md)
- [2026-09-10：根据 Live 评审修订英文 RFC](2026-09-10-m2-english-review.md)
- [2026-09-10：M2 中英文 RFC 全文同步](2026-09-10-m2-bilingual-sync.md)
- [2026-09-11：补读最新 RFC 评审，明确减号规则](2026-09-11-m2-minus-lexing.md)
- [2026-09-11：用 tscircuit 源码复核 M2 取舍](2026-09-11-tscircuit-rfc-review.md)
- [2026-09-12：M2 选定 A，归档 B 实施计划](2026-09-12-m2-scope-a.md)
- [2026-09-12：比较循环标签与区间，建立审计清单](2026-09-12-m2-syntax-options.md)
- [2026-09-12：聚焦 RFC，将后续工作收录为附录](2026-09-12-m2-rfc-appendices.md)

M2 已选 A，B 延后；当前仍为 Proposed，直接推进的是 RFC 正文评审。标签/区间比较和声明验证器保留为附录中的后续工作。已有 subdesign 能力的实际实验见[工作台与电路实操记录](workbench-circuits.md)。
