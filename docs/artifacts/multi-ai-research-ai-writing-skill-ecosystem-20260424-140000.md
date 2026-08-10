# Multi-AI Research: AI 辅助写作 Skill / 工具 / 工作流 生态扫描

**会话时间**：2026-04-24 14:00
**研究问题**：用 AI 写长文/blog/thread/newsletter 的 skill 生态有什么？2026 年真在用、能产出高质量内容的有哪些？
**和 v1 报告关系**：v1（scout-to-article-20260424-103600.md）是基于「我的素材+现有 skill」出方案；本报告是「业界生态扫描」，互补不重复

**调研模式**：3-way（grok ✅ + gemini ✅ + 内部 Explore agent ✅，chatgpt ❌ chatgpt.com send button DOM bug）

---

## Phase 5: 交叉验证矩阵

### A. 开源 Skill / Repo 发现

| Skill / Repo | grok | gemini | 内部 agent | Tier | URL |
|---|---|---|---|---|---|
| **AgriciDaniel/claude-blog**（22 子 skill / 12 内容模板 / 5 级质量评分 / SEO 优化） | ✅ | (未提) | ✅ 详细 | 🟢 **双重确认** | https://github.com/AgriciDaniel/claude-blog |
| **ComposioHQ/awesome-claude-skills 的 content-research-writer**（研究 + 引文 + 逐节反馈） | ✅ 给 URL | (未提) | ✅ 给 URL | 🟢 **双重确认** | https://github.com/ComposioHQ/awesome-claude-skills/blob/master/content-research-writer/ |
| **simonw/tools**（blog-to-newsletter 等真实工具集） | ✅ | ✅ | ✅ | 🟢 **三重确认** | https://github.com/simonw/tools |
| **Karpathy LLM Wiki gist**（增量维护知识库 → 整合新源 / 检测矛盾） | ✅ | ✅ 描述 | ✅ 验证 URL | 🟢 **三重确认** | https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f |
| **alirezarezvani/claude-skills**（232+ skills，含 content-creator/blog-writing 系列） | ✅ 具体仓库 + 数量 | (未提) | (未列) | 🟡 单家强信号 | https://github.com/alirezarezvani/claude-skills |
| **K-Dense-AI/claude-scientific-writer**（学术论文/clinical/grant，集成 Perplexity） | (未提) | (未提) | ✅ | 🟡 单家发现 | https://github.com/K-Dense-AI/claude-scientific-writer |
| **haowjy/creative-writing-skills**（17 agent + 13 skill，创意写作） | (未提) | (未提) | ✅ | 🟡 单家（与你需求弱相关 — 创意非分析） | https://github.com/haowjy/creative-writing-skills |
| **anthropics/skills (skill-creator)**（自定制写作 agent） | ✅ | (未提) | (未提) | 🟡 单家但官方权威 | https://github.com/anthropics/skills |
| **swyx ai-notes** | ✅ "raw notes → 素材库" | (未提) | 🔴 **验证后说"是知识库不是执行 workflow"** | 🟡 **数据冲突** | https://github.com/swyxio/ai-notes |

### B. 商业 AI 写作产品

| 产品 | grok | gemini | Tier | 你的相关性 |
|---|---|---|---|---|
| **Lex.page** | ✅ "最活跃，长文/营销文首选，Dan Martell 等每天用" | ✅ "顶级长文编辑器，协作式预测，停顿时给精准下文" | 🟢 **强共识** | ⭐⭐⭐⭐⭐ 直接试用 |
| **NotebookLM** | ✅ "研究-写作必备，2026 仍爆火" | ✅ "Source-Grounded，所有内容必带引文" | 🟢 **强共识** | ⭐⭐⭐⭐ scout 14 篇直接喂 |
| **Cursor 写作模式** | ✅ "越来越多 thread/blog 用 Cursor + Claude 写 markdown" | ✅ "像管理代码库一样管理文章" | 🟢 **强共识** | ⭐⭐⭐⭐ 你已有 Cursor |
| **Sudowrite** | ✅ "创意/长文/小说仍活跃" | ✅ "Story Engine 世界观一致性" | 🟢 **强共识** | ⭐ 偏小说，分析文不适用 |
| **Granola** | 🟡 "活跃度较低" | ✅ "职场/会议笔记 → 周报" | 🟡 部分 | ⭐⭐ 仅会议场景 |
| **Reflect / Mem.ai** | 🟡 "提及少" | ✅ "数字孪生写作" | 🔴 **冲突** | ⭐⭐ 需自己试 |
| **Lex.studio** | 🚫 "几乎无讨论" | (未提) | 🚫 排除 | — |

### C. AI 写作 Agent 模式（方法论）

| 模式 | grok | gemini | Tier |
|---|---|---|---|
| **Critique-rewrite loop**（初稿 → AI 批判 → 重写，迭代 2-3 轮） | ✅ "最常用" | ✅ "Iterative Refinement 论文成主流" | 🟢 **强共识** |
| **Multi-draft + judge**（生成 N 个版本 → AI judge 选优） | ✅ "Cursor agent 常用" | ✅ "Writer-Critic 循环" | 🟢 **强共识** |
| **Reader-as-critic agent**（模拟目标读者提问 → 逐一回答重写） | ✅ | ✅ "Reader-as-Critic 论文" | 🟢 **强共识** |
| **Knowledge-graph / Obsidian + LLM**（双向链接 + 每日笔记 → 长文） | ✅ "Karpathy LLM Wiki + Obsidian 高产标配" | ✅ "RAG-for-writing 风格迁移" | 🟢 **强共识** |
| **拒绝一键生成 / 原子化写作**（人工大纲 → AI 填素材 → 人工重构 → AI 润色） | ✅ Karpathy/Simon 都手写 | ✅ 明确 "Common Pattern" | 🟢 **超强共识** |

### D. 反 AI Slop 内置技术

| 技术 | 来源 | Tier |
|---|---|---|
| **Cliché Detector**（Lex & Writer.com 实时高亮 AI 标志修辞） | gemini | 🟡 单家详细 |
| **黑名单词库**（Jasper 2026 版从底层切断特定词） | gemini | 🟡 单家详细 |
| **Few-shot Personalization**（上传 10 篇旧文 → fine-tune → 对齐 sentence length distribution） | gemini | ⚪ 单家未验证 |
| **Heavy Editing**（thought-leader 共识：人工重写不可省） | grok + gemini | 🟢 强共识 |

### E. Thought-leader 真实工具链共识

| 人 | grok 描述 | gemini 描述 | Tier |
|---|---|---|---|
| **Simon Willison** | Datasette + LLM CLI（files-to-prompt + blog-to-newsletter） | "LLM as over-confident junior"，自研工具喂模型生成大量初稿 + 多次 llm CLI 转换 + 极繁重人工重写 | 🟢 **强共识 + 互补描述** |
| **Karpathy** | LLM Wiki "idea file" gist | 用 Python 脚本调 API 处理"分段合成"，拒绝单一 UI | 🟢 **强共识** |
| **swyx** | ai-notes 仓库 raw → newsletter/blog | (未提细节) | 🟡 但内部 agent 说该 repo 是知识库非 workflow |
| **Mitchell Hashimoto** | https://mitchellh.com/writing/my-ai-adoption-journey "先 agent 调研 → 自己写" | (未提) | 🟡 单家但 URL 可验证 |

---

## Phase 6: 改动清单（Tiered）

### 🟢 极高置信度必做（多源强共识）

**1. ★ 直接 clone 这 3 个仓库挖一遍（最高 ROI 起点）**

```bash
# 双重确认 + 22 子 skill 最完整
git clone https://github.com/AgriciDaniel/claude-blog ~/code/external/claude-blog

# 双重确认 + 研究→初稿专用
git clone https://github.com/ComposioHQ/awesome-claude-skills ~/code/external/awesome-claude-skills

# 三重确认 + Simon Willison 真实在用
git clone https://github.com/simonw/tools ~/code/external/simonw-tools
```

挖什么：
- claude-blog 的 12 内容模板 + 5 级质量评分 → 评估能否抄来作为你的 `scout-to-article` 的"模板库"
- content-research-writer 的 prompt 设计 → 抄选题/反馈环
- simonw/tools 看 blog-to-newsletter 工具的 HTML+JS 实现思路 → 你的 scout 也能做"日报 → newsletter HTML"

**2. ★ 试用 Lex.page**（双源强共识，无需开发）

- 注册 → 把今天的 Bridge 1「验证才是瓶颈」素材丢进去 → 体验"协作式预测"
- 评估它在"写技术分析文"上的实际手感
- 如果手感对，可作为日常长文 GUI 编辑器

**3. ★ NotebookLM 喂 14 篇 scout 精读**（双源强共识）

- 把今天 14 个深抓的链接全喂进去（NotebookLM 支持 URL 直接吃）
- 让它生成大纲 / 综述 / 引文版
- 评估能否替代 / 加速你的 Bridge Notes 整理环节

**4. ★ 采用「Reader-as-Critic」迭代写作模式**（双源强共识）

- 写完初稿后，让 Claude 用"目标读者"角色提 5 个最尖锐的问题
- 逐个改写
- 替代单纯 humanizer 的"反 AI 味"路径

---

### 🟡 高置信度建议（单家强信号 + 来源可信）

**5. 评估 alirezarezvani/claude-skills 232 skill 集合**
- grok 单家但给了具体仓库 + 数量，可信
- 风险：232 个里大部分可能低质量，需筛选
- 建议：浏览 README + 挑 1-2 个 content-creator 类试用

**6. 抄 Karpathy LLM Wiki 模式做 scout-to-article 的中间层**
- 三源都验证存在
- gist 里的设计：增量维护 markdown wiki + 自动整合新源 + 检测矛盾
- 你 scout 每天都在产生新源，正好用得上

---

### ⚪ 待验证（单家发现）

**7. K-Dense-AI/claude-scientific-writer**
- 内部 agent 单独发现
- 学术写作垂类，但你写的是"分析文"，可借鉴的可能是其"集成 Perplexity 实时搜索"做事实核查的设计

**8. 反 AI slop 商业内置技术（Lex Cliché Detector / Jasper 黑名单）**
- gemini 单家详细
- 价值：可启发你的 humanizer 升级方向（加 cliché list + sentence length 检测）

---

### 🚫 不做

**9. 不试 Lex.studio / Reflect 主写作场景**
- grok 明确说"几乎无讨论 / 活跃度低"
- gemini 提到 Reflect 但与 grok 冲突 → 信号弱

**10. 不试 Sudowrite 写技术分析文**
- 双源都说它定位创意/小说
- 与你的"AI coding agent 分析"风格不匹配

---

## 直接可执行的最小组合（今天就能动）

按"15 分钟试用 → 看是否值得深入"原则：

| 步骤 | 时间 | 工具/资源 | 预期产出 |
|---|---|---|---|
| 1. clone AgriciDaniel/claude-blog 看 SKILL.md 设计 | 10 min | git + Read | 决定是否抄来作为 scout-to-article 模板库 |
| 2. clone ComposioHQ/awesome-claude-skills 看 content-research-writer | 10 min | git + Read | 决定是否抄 prompt 设计 |
| 3. NotebookLM 喂今天 scout 14 个 URL | 5 min | notebooklm.google.com | 看自动综述质量 vs 你手写 Bridge Notes |
| 4. Lex.page 注册 + 写 Bridge 1 100 字开头 | 10 min | lex.page | 评估编辑体验 |
| 5. 决定哪个最有用，把核心抄到本地 scout-to-article skill | 30 min | Edit | 升级版 SKILL.md |
| **总耗时** | **~1 小时** | | 一份"已验证有用"的工具组合 |

---

## Metadata

- 总耗时：~3 分钟（重派外部 + 内部 + Phase 5/6）
- 任务数：4 派发 → 3 完成（grok / gemini / 内部 agent）+ 1 失败（chatgpt 站点 send button bug，重试无救）
- 仲裁模式：3-way（仍能交叉，单家发现降低权重）
- 关键发现冲突：swyx ai-notes 仓库定位（grok 说"素材库可用"，内部 agent 实地验证后说"是知识库非 workflow"）→ **采信内部 agent 实地验证**
- 待解 chatgpt.com 集成 bug：chatgptcli 找不到 send button，可能 chatgpt 改版/反爬，需 chatgptcli 升级或换其他 chatgpt 接入方式
