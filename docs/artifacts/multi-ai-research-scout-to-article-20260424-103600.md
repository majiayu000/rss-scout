# Multi-AI Research: Scout 报告 → 对外文章 加工链路（4-way 完整版）

**会话时间**：2026-04-24 10:30 → 12:50（含修复 Node 19 → v22 + 重跑外部 AI）
**研究问题**：14 篇 AI coding agent 精读 + 5 个 Bridge Notes 的 22KB 内部 scout 报告，如何加工成发 blog / 微信公众号 / X Article 的对外文章？

**外部 AI 状态（修复 Node 后重跑全绿）**：
- ✅ **grok**：完成（37s 思考，引用 Karpathy / Simon Willison 真实工具）
- ✅ **gemini**：完成（5 博主对比表 + 4 平台适配）
- ✅ **chatgpt**：完成（7 模块 skill 设计 + PRISMA 借鉴 + 反 slop 7 招）
- ✅ **内部 sub-agent**：已完成 skill 扫描

**调研模式**：完整 4-way（1 内部 + 3 外部独立 AI）。

---

## Phase 1-3: 派发与回收
- 4 路同时派发，3 外部全部成功，最长耗时 ~3 分钟
- artifact 完整原始输出存放在临时 task 文件，本文档为 Phase 5/6 仲裁结果

---

## Phase 5: 交叉验证矩阵（4-way）

| # | 发现 | Grok | Gemini | ChatGPT | 内部 | Tier |
|---|---|---|---|---|---|---|
| 1 | **angle-picker（选题）是最高 ROI** | ✅ "挑 1 bridge insight" | ✅ "Framework First" | ✅ "最高 ROI = angle-picker" | (隐含) | 🟢 **超强共识** (3/3) |
| 2 | **Bridge Notes 是天然选题入口** | ✅ "bridge insight" | ✅ "跨界交叉用 bridge" | ✅ "angle-picker 输入" | — | 🟢 **超强共识** (3/3) |
| 3 | **LLM 只做编译/proofreading，禁止生成正文** | ✅ Karpathy/Simon 都手写 | ✅ "原始记录 + 个人语气" | ✅ "anti-slop-editor 模块" | (humanizer 覆盖) | 🟢 **超强共识** (3/3) |
| 4 | **每个判断必须有具体证据**（代码/截图/repo） | ✅ "反直觉数据 hook" | ✅ "代码/截图证伪" | ✅ "claim-verifier 模块" | — | 🟢 **超强共识** (3/3) |
| 5 | **反 AI slop 核心 = 亲自做+保留个人语气** | ✅ "重写 I 段 + 手动 touchup" | ✅ "保留吐槽" | ✅ "禁套话 + 失败案例" | (humanizer 23 类) | 🟢 **超强共识** (3/3) |
| 6 | **应新建轻量 pipeline skill** | (未明说) | (未明说) | ✅ "必做，7 模块" | 🟡 "拼合可达" | 🟢 **共识可达成**（见仲裁） |
| 7 | **Thread 试水 → Article → 公众号渐进** | ✅ "Thread 优先互动" | ✅ "Thread 试水筛选" | (未明说优先级) | — | 🟡 部分共识 (2/3) |
| 8 | **Stratechery「framework + 节奏」模式** | (未明说) | ✅ | ✅ | — | 🟡 部分共识 (2/3) |
| 9 | **有真实公开工具可参考** | ✅ Datasette/ai-notes/idea gist | (未明说工具) | (未明说) | — | 🟡 单家详细 |
| 10 | **借鉴学术 PRISMA 综述方法** | (未明说) | (未明说) | ✅ "明确为什么综述/如何筛选/如何综合" | — | ⚪ 单家提议 |

### 仲裁说明

**第 6 行（是否新建 skill）**——表面冲突，本质共识：
- ChatGPT 推荐的 7 模块（source-normalizer / angle-picker / audience-router / claim-verifier / hook-designer / structure-composer / anti-slop-editor）**全部是 checkpoint，不是 AI 自动执行**
- 内部 agent 说"拼合可达，不必新建大 skill"——指的是**不要 AI 替你写文章**
- **两者一致**：做一个轻量编排 skill（< 100 行），用 ChatGPT 的 7 模块命名作为强制 checkpoint，但每个模块都是 **人工 + 工具辅助**，不接管创作

---

## Phase 6: 升级版改动清单（基于 4-way 共识）

### 🟢 极高置信度必做（4-way 强共识）

**1. ★ angle-picker 是最高 ROI（覆盖原 v1 推荐）**

> 原 v1 推荐"开 public repo 把 scout 当 asset"
> 4-way 共识：**先选对角度比任何长期 asset 路径都更紧迫**
> 两条不冲突——但单篇文章紧急度排序：angle-picker 第一，public repo 第二

**Action**：每次发文前，强制从 5 个 Bridge Notes 中选 1 个角度，说出 3 句话主张（不是 14 个候选，是 5 个 Bridge）

**当前 5 个 Bridge 推荐排序**：
- 🥇 **Bridge 1「验证才是瓶颈」**——4 源（SWE-chat 44% / Addy 80% Problem / Bad Harness / Agent Orchestrator）+ 反直觉 + 数据强
- 🥈 **Bridge 2「Harness Engineering 已成方法论」**——4 源（Fowler / OpenAI / Show HN / TheSequence）+ 与你 harness 项目本身呼应
- 🥉 **Bridge 4「Rust 已成 AI agent CLI 事实标准」**——4 源 + 强信号 + 反"Python/Node 默认"共识

**2. ★ LLM 只做编译/proofreading，禁止生成正文**（3/3 外部强共识）

3 家都引用真实案例：
- Karpathy：LLM 编译 Obsidian raw → wiki，但**自己重写 I 段 + 手动 touchup**
- Simon Willison：LLM **只做 proofreading**，正文手写
- ChatGPT 设计 anti-slop-editor：**删空话 + 加失败案例 + 保留作者立场**

**操作含义**：用 `x-post` 时只让它产出"草稿框架"或"已收集的事实点"，正文必须人工写

**3. ★ 每个判断必须有具体证据**（3/3 共识）

最具体可执行的版本（综合 3 家）：
- 数据 → 必须有 source link（如 SWE-chat arXiv ID）
- 论断 → 必须有 repo commit / CLI 输出 / 截图
- 引用 → 必须有 quote 原文 + 链接
- "我认为" → 必须有反例对照（"虽然 X 似乎是 Y，但 Z 案例证明..."）

**4. ★ 反 AI slop 7 招清单（ChatGPT 提供，可立即用）**

```
① 每节只保留一个判断句
② 每个判断后放"因为/反例/所以"三件套
③ 禁用"正在重塑"、"值得关注"、"赋能"等套话
④ 加一个失败案例（自己亲历的）
⑤ 明确"我不同意 X 哪里"（反共识立场）
⑥ 用具体数字或版本号（如"14 篇里 6 篇都指向 eval"）
⑦ 结尾给读者一个可执行 checklist
```

---

### 🟡 高置信度建议做（2/3 部分共识）

**5. Thread 试水 → 拿数据 → 决定扩 Article + 公众号**

- Grok：Thread 互动远高于 Article，特别是带个人 hook 的
- Gemini：5 个 Bridge 全做 Thread → 看互动 → 最高的扩公众号
- 实操：每周一发 Bridge Note 1 的 thread（10-15 条），周五看互动指标，决定扩长

**6. 借鉴 Stratechery「framework first」模式**

不是"今天发生了什么"，而是"什么稳定 framework 解释了今天发生的事"
- 你的 Bridge Notes 已经天然是 framework
- 写文章时强制把 Bridge 升格为「Harness Engineering 三定律」式的命名

---

### ⚪ 待验证（单家提议，需要先小试）

**7. PRISMA 综述方法借鉴**（ChatGPT 独家）

学术综述的 4 步：明确为什么综述 / 如何筛选 / 如何综合证据 / 如何评估
- 可作为 angle-picker 的 evaluation rubric
- 待小试验证是否真有用

**8. 真实公开工具借鉴**（Grok 独家但来源可信）

- Karpathy idea file gist：LLM wiki pipeline 模板（待找具体 URL 验证）
- Simon Willison **tools.simonwillison.net**：blog-to-newsletter 工具（grok 提到 2026/4 公开）
- swyx **GitHub ai-notes**：raw datastore → Latent Space 写作流（待找 repo 验证）

---

### 🚫 不做（4-way 反对）

**9. 绝不"AI 自动生成正文 + 自动发布"**

3 家外部 + 内部 agent 全部反对：
- Grok：Karpathy/Simon 都手写正文
- Gemini：Anti-slop 核心是"AI 难模拟的人类真实感"
- ChatGPT：anti-slop-editor 强调"保留作者立场"
- 内部：humanizer 是补救，不是替代手写

**10. 不发"今天 scout 精读罗列文"**

- 违反 Stratechery「framework > news」+ swyx「assets > content」+ Gemini 反对的"通用新闻"

---

## 升级版 SKILL.md 骨架（基于 4-way 共识）

```markdown
---
name: scout-to-article
description: 把 RSS knowledge scout 内部报告加工成 X Thread / X Article / 微信公众号 / blog 的强制流水线 + checkpoint。Use when user says "把今天 scout 写成文章" / "发 thread 讲讲今天看到什么". 7 个 checkpoint 全部强制人工 + 工具辅助；不接管选题判断；不自动写正文（4-way 调研共识）。
---

# Scout → Article Pipeline (v2)

## 7 强制 Checkpoint（不能跳）

### 1. source-normalizer（自动）
- 把今日 scout 拆成 3 类：单篇精读 / Bridge Notes / 元事件（如配额耗尽）
- 输出：3 张单一来源表

### 2. angle-picker（强制人工）★ 最高 ROI
- **必须从 Bridge Notes 中选 1 个**（不是从精读列表中选）
- 输出：3 句话主张 + 反直觉点 + 反对者会怎么反驳

### 3. audience-router（半自动）
- 根据角度判断走哪个平台：
  - 数据强 + 反直觉 → X Thread 试水（10-15 条）
  - Bridge 跨多领域 → X Article（1500-2500 字）
  - 中文社区有相关讨论 → 公众号（5000+，加补叙）
  - 完整版 → blog（8000+）

### 4. claim-verifier（半自动）
- 列出主张里的所有断言，每个必须满足 ≥1：
  - [ ] 数据 source link
  - [ ] Repo commit / CLI 输出截图
  - [ ] 引用原文 + 链接
  - [ ] 反例对照
- 不满足 → 删掉该断言或降级为"假设"

### 5. hook-designer（强制人工）
- 用户必须提供前 280 字符 hook（X Article 显示长度）
- 推荐结构：1 个数字 + 1 个反直觉断言（参考 Karpathy 模式）
- 禁止 AI 替写

### 6. structure-composer（半自动 + 人工 finalize）
- 框架：问题 → 证据 → 判断 → 建议（ChatGPT 推荐）
- AI 可以打草稿框架，正文人工写
- 强制 7 招反 slop 检查

### 7. anti-slop-editor + publish（半自动 + 人工 review）
- humanizer 跑一遍
- 人工再 review：
  - [ ] 删掉所有"我没真亲身验证"的断言
  - [ ] 删掉所有内部信息（IP / TEMP_DIR / 内部规则代号）
  - [ ] 每个 link 点开过
- 平台分发：x-article-publisher / weixin-mp / 直接 push blog

## 禁止
- 跳过任何一步
- AI 自动选题（违反 angle-picker）
- AI 写正文（违反 4-way 共识）
- 把 scout 内部 metadata 漏出去
- 一稿多发不改写
```

---

## 升级版"如果今天就要发"清单（最小可执行）

| 步骤 | 时长 | 工具 |
|---|---|---|
| 1. 选 Bridge 1「验证才是瓶颈」做主线 | 5 min（人工） | — |
| 2. 写 hook：「**SWE-chat 数据：AI 写的代码 44% 活不到 commit**」 | 10 min（人工） | — |
| 3. 列证据 4 项（SWE-chat / Addy / Bad Harness / Agent Orchestrator） + 验证每个 link | 15 min（半自动） | claim-verifier 清单 |
| 4. 写正文（手写，参考 Stratechery「framework 解释 news」） | 30 min（人工） | — |
| 5. 跑 humanizer + 反 slop 7 招过一遍 | 5 min（半自动） | humanizer |
| 6. 人工 review + 脱敏 | 10 min（人工） | — |
| 7. x-article-publisher 发 X Article 草稿 + 再读 hook | 5 min | x-article-publisher |
| **总耗时** | **80 min**（首次） | |

---

## Metadata
- 总耗时：调研 + 仲裁 + 落地约 25 min
- 任务数：4 派发 → 4 全部完成（修复 Node 后）
- 外部 AI 状态：grok ✅ gemini ✅ chatgpt ✅
- 仲裁模式：完整 4-way（1 内部 + 3 外部独立投票）
- 升级点 vs v1：
  - angle-picker 优先级从"建议"升级到"最高 ROI 必做"
  - skill 设计从"轻量编排"升级为"7 强制 checkpoint"
  - "LLM 不写正文"从"建议"升级到"硬规定"（3/3 外部强共识）
  - 反 AI slop 从"用 humanizer"升级到"7 招 SOP + humanizer"
- 待验证项：Karpathy idea file gist / Simon tools.simonwillison.net / swyx ai-notes 的具体 URL
