# Multi-AI Research — RSS Feed Discovery (2026-04-27)

## 原始研究问题

发掘新的高质量 RSS 源扩展 rss-scout `feeds.toml`（当前 157 个源），目标 AI coding agents / agent architecture / LLM tooling / AI research / AI industry analysis（中英文均可）。

约束：
1. 必须官方 RSS（不要 RSSHub 镜像，rsshub.app 当前 403）
2. 必须 curl 拿到合法 XML（不是 HTML/404/auth 墙）
3. 每周至少更新一次

## Phase 1 任务分解

| 通道 | 类型 | 范围 | 状态 |
|------|------|------|------|
| Internal A | sub-agent | 英文分析博客 + 实战 newsletter（curl-verify） | ✅ 完成 |
| Internal B | sub-agent | OSS 社区 + GitHub releases + 漏掉的 vendor blogs（curl-verify） | ✅ 完成 |
| Internal C | sub-agent | 亚太 AI（日韩中，curl-verify） | ✅ 完成 |
| Grok | background bash | 实时社区/X 信号 | ✅ 完成 |
| Gemini | background bash | Google 生态/结构化 | ✅ 完成 |
| ChatGPT | background bash | 强推理 | ❌ BLOCKED (send button not found) |

降级到双外部模式（Grok + Gemini），仍满足 skill 的 v1 双外部共识标准。

## Phase 5 交叉验证矩阵

| 类别 | 数量 | 说明 |
|------|------|------|
| Strong consensus（Internal verified + ≥1 外部确认） | 5 | Hamel Husain / OpenHands / HF Blog / r/MachineLearning / Preferred Networks EN |
| Internal verified-200 only（curl 即 ground truth） | 29 | 见下方 final list |
| External-only verified（外部推荐 + 我现场 curl 通过） | 7 | Vicki Boykis / Charles Frye / Paco Nathan / Gary Marcus / DagsHub / Rex Coleman / DS+AI Section |
| Conflict（外部推荐但 Internal 测过失败） | 13 | Mistral / Cohere / RunwayML / Stability / Perplexity / Groq / DeepSeek / Plandex / Sweep / 极客公园 / 智谱 / The Rundown AI / Block xyz blog |
| Hallucination（外部独家推荐但 curl 现场失败） | 7 | Sayak Paul / LLM Extractum / Naver AI Lab gh.io / Block xyz / Analytics India Mag / AIGenerative / aaif-goose（Grok 误记） |

**外部 AI URL 准确率**：Gemini ~30%，Grok ~28%。两家都频繁猜 vendor blog 路径——印证 Internal B 结论："modern AI vendor blogs (Mistral/Cohere/Stability/Runway/Perplexity/Groq/LM Studio/AssemblyAI/ElevenLabs/xAI/DeepSeek) 全部是 Next.js/SPA 不暴露 RSS"。

## Phase 6 Tiered Action Items

### 🟢 Tier 1: Strong consensus（5 个，最高置信度）

Internal curl-verified + 外部 AI 至少一家独立确认。

| # | name | url | bucket | post_freq |
|---|------|-----|--------|-----------|
| 1 | Hamel Husain | https://hamel.dev/index.xml | analysis blog | weekly |
| 2 | GH:All-Hands-AI/OpenHands | https://github.com/All-Hands-AI/OpenHands/releases.atom | community | weekly |
| 3 | HuggingFace Blog | https://huggingface.co/blog/feed.xml | analysis | 3-5/week |
| 4 | Reddit:r/MachineLearning | https://www.reddit.com/r/MachineLearning/.rss | community | daily |
| 5 | Preferred Networks (EN) | https://tech.preferred.jp/en/feed/ | apac/jp | monthly |

### 🟢 Tier 2: Internal verified-200 only（29 个，curl 即事实）

#### 英文分析博客 / 个人研究者（10 个）

| # | name | url | post_freq | rationale |
|---|------|-----|-----------|-----------|
| 6 | Jason Wei | https://www.jasonwei.net/blog?format=rss | 2/月 | Ex-OpenAI / Anthropic researcher, CoT/scaling-laws |
| 7 | Yoav Artzi | https://www.yoavartzi.com/feed.xml | weekly | Cornell NLP, agent eval / language grounding |
| 8 | Sasha Rush | https://srush.github.io/feed.xml | bi-weekly | Cornell/HF, Annotated Transformer, LLM internals |
| 9 | Andriy Burkov (TheLMBook) | https://www.thelmbook.com/feed | weekly | "100-Page LLM Book" 作者，agent/eval methodology |
| 10 | Horace He (Thonking) | https://www.thonking.ai/feed | bi-weekly | PyTorch core, GPU/perf, harness engineering |
| 11 | Neel Nanda | https://www.neelnanda.io/blog?format=rss | weekly | DeepMind interp lead, mechanistic interp / safety |
| 12 | Finbarr Timbers (Artificial Fintelligence) | https://www.artfintel.com/feed | weekly | Ex-DeepMind, RL/agents/training infra |
| 13 | LessWrong (curated) | https://www.lesswrong.com/feed.xml?view=curated | 2-4/week | AI safety / Daniel Kokotajlo / alignment 主阵地（curated 限定） |

#### 实战 newsletter / podcast（2 个）

| # | name | url | post_freq | rationale |
|---|------|-----|-----------|-----------|
| 14 | Cognitive Revolution (Nathan Labenz) | https://feeds.megaphone.fm/cognitiverevolution | 3-4/week | Long-form practitioner interviews on agents/eval/frontier labs |
| 15 | Big Technology (Alex Kantrowitz) | https://www.bigtechnology.com/feed | 3-4/week | 业界访谈 + 分析，frontier-lab access |

#### Reddit / 开源社区（3 个）

| # | name | url | post_freq | rationale |
|---|------|-----|-----------|-----------|
| 16 | Reddit:r/OpenAI | https://www.reddit.com/r/OpenAI/.rss | high | OpenAI 产品/新闻最快社区 |
| 17 | Reddit:r/AI_Agents | https://www.reddit.com/r/AI_Agents/.rss | high | Practitioner discussion on agent frameworks |
| 18 | Reddit:r/ArtificialInteligence | https://www.reddit.com/r/ArtificialInteligence/.rss | high | 综合 AI news subreddit |

#### GitHub releases for agent frameworks（7 个）

| # | name | url | post_freq | rationale |
|---|------|-----|-----------|-----------|
| 19 | GH:block/goose | https://github.com/block/goose/releases.atom | weekly+ | Block 开源 agent CLI，5 releases Apr 17-23 |
| 20 | GH:sst/opencode | https://github.com/sst/opencode/releases.atom | daily | 开源 coding agent，每日发版 |
| 21 | GH:danny-avila/LibreChat | https://github.com/danny-avila/LibreChat/releases.atom | weekly | 多模型 chat UI，4 月活跃 |
| 22 | GH:microsoft/semantic-kernel | https://github.com/microsoft/semantic-kernel/releases.atom | weekly | MS agent SDK |
| 23 | GH:mem0ai/mem0 | https://github.com/mem0ai/mem0/releases.atom | multi/week | Agent memory layer，5 days 5 releases |
| 24 | GH:ToolJet/ToolJet | https://github.com/ToolJet/ToolJet/releases.atom | daily | Low-code AI app builder |
| 25 | GH:Significant-Gravitas/AutoGPT | https://github.com/Significant-Gravitas/AutoGPT/releases.atom | weekly | 基础 AutoGPT 平台 |

#### 亚太 AI（9 个）

| # | name | url | language | post_freq | rationale |
|---|------|-----|----------|-----------|-----------|
| 26 | Preferred Networks (JA) | https://tech.preferred.jp/ja/feed/ | ja | weekly | 日本顶级 DL lab（Chainer/MN-Core/PLaMo） |
| 27 | Sakana AI Blog | https://sakana.ai/feed.xml | en | 2/月 | 东京 frontier lab（David Ha / Llion Jones） |
| 28 | ABEJA Tech Blog | https://tech-blog.abeja.asia/feed | ja | 2-3/week | 日本应用 AI 实战（LLM ops / RAG / fine-tune） |
| 29 | AI Shift Tech Blog | https://www.ai-shift.co.jp/techblog/feed | ja | weekly | Cyberagent 子公司，voicebot/LLM agent ⚠️ latest 2026-03-05 |
| 30 | CyberAgent Developers | https://developers.cyberagent.co.jp/blog/feed/ | ja | daily | AI Lab / CALM LLM / recsys |
| 31 | LY Corporation (LINE/Yahoo) | https://engineering.linecorp.com/ja/feed/index.html | ja | 3-4/week | HyperCLOVA-X 相邻 ML/LLM 平台 |
| 32 | Naver D2 | https://d2.naver.com/d2.atom | ko | weekly | Naver Clova / HyperCLOVA / search-LLM 团队（Clova 独立 blog 已 404） |
| 33 | Kakao Tech | https://tech.kakao.com/feed/ | ko | weekly | Kakao Brain (Karlo, KoGPT) 主阵地 |
| 34 | AI Singapore | https://aisingapore.org/feed/ | en | ⚠️ stale | 唯一 SEA 渠道但 last post 2025-09-02 |

### 🟢 Tier 3: External-verified（7 个，外部推荐 + curl 现场验证）

| # | name | url | source | post_freq |
|---|------|-----|--------|-----------|
| 35 | Vicki Boykis | https://vickiboykis.com/index.xml | Gemini | 3/月 |
| 36 | Charles Frye | https://charlesfrye.github.io/feed.xml | Gemini | 2/月 |
| 37 | Paco Nathan | https://medium.com/feed/@pacoid | Gemini | weekly |
| 38 | Gary Marcus | https://garymarcus.substack.com/feed | Grok | weekly |
| 39 | DagsHub Blog | https://dagshub.com/blog/feed | Grok | weekly |
| 40 | Rex Coleman (AI Security) | https://rexcoleman.dev/index.xml | Grok | weekly |
| 41 | DS+AI Section | https://rssdsaisection.substack.com/feed | Grok | weekly |

### 🚫 不采纳（13 个，外部推荐但 Internal 已测失败）

外部 AI 自信地推荐但 URL 不工作。SPA 没 RSS / 死路径 / 已 stale：

- Mistral / Cohere / RunwayML / Stability / Perplexity / Groq / DeepSeek（vendor 全是 SPA）
- Plandex（last release 2025-07）/ Sweep（dead since 2023）
- 极客公园（SSL fail）/ 智谱（SPA）/ The Rundown AI（404）/ Block xyz（404）

### ⚠️ 潜在幻觉（7 个，单家推荐 + 现场验证失败）

- Sayak Paul (sayak.dev/feed.xml — 404, Gemini)
- LLM Extractum (SSL fail, Gemini)
- Naver AI Lab github.io (404, Gemini)
- Analytics India Magazine (HTML, Grok)
- AIGenerative (empty reply, Gemini)
- Block xyz blog (404, Gemini)
- aaif-goose (Grok 把 block/goose 误记为 aaif-goose)

## Phase 7 Metadata

- **耗时**：~10 分钟（6 个并行任务）
- **任务数**：3 sub-agent + 3 background bash = 6
- **外部 AI 可用**：2/3（ChatGPT BLOCKED）
- **总验证 verified-200**：41 个
- **建议加入数**：30-34 个（cap 由 skill 设置）
- **新增源占现有比例**：30/157 = +19%
