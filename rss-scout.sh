#!/usr/bin/env bash
# rss-scout.sh — 零 API key 的 AI 开发领域知识发现
# 依赖: curl, python3 (macOS 自带)
# 用法: bash rss-scout.sh [--dry-run]
set -euo pipefail

DATA_DIR="${RSS_SCOUT_DIR:-$HOME/.rss-scout}"
SEEN_FILE="$DATA_DIR/seen.txt"
OUTPUT_DIR="$DATA_DIR/output"
MAX_ITEMS=30
DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PARSER="$SCRIPT_DIR/parse_feed.py"
FILTER="$SCRIPT_DIR/filter_entries.py"

KEYWORDS="agentic|vibe.?cod|ai.?cod|coding.?agent|claude.?code|cursor.?ai|copilot|llm.?tool|ai.?engineer|context.?engineer|mcp|harness|code.?gen|ai.?pair|prompt.?engineer|llm.?agent|rag|fine.?tun|tool.?use|function.?call|agentic.?ide|windsurf|swe.?bench|swe.?agent|code.?review|cline|devin|guardrail|alignment|red.?team|augment.?code|rust.?lang|cargo|tokio"

# ── 源定义 (名称|URL|skip_filter) ─────────────────────
# 共 113 源 (更新于 2026-03-24)
FEEDS=(
  # ── HN 服务端过滤 (20) ──
  "HN:agentic-coding|https://hnrss.org/newest?q=agentic+coding&points=5&count=30|1"
  "HN:vibe-coding|https://hnrss.org/newest?q=vibe+coding&points=5&count=20|1"
  "HN:AI-coding-agent|https://hnrss.org/newest?q=AI+coding+agent&points=10&count=20|1"
  "HN:claude-code|https://hnrss.org/newest?q=claude+code&points=5&count=20|1"
  "HN:context-engineering|https://hnrss.org/newest?q=context+engineering&points=5&count=20|1"
  "HN:spec-driven|https://hnrss.org/newest?q=spec+driven+development&points=5&count=20|1"
  "HN:harness-engineering|https://hnrss.org/newest?q=harness+engineering&points=5&count=20|1"
  "HN:mcp-protocol|https://hnrss.org/newest?q=model+context+protocol&points=5&count=20|1"
  "HN:cursor-ai|https://hnrss.org/newest?q=cursor+AI&points=5&count=20|1"
  "HN:copilot-agent|https://hnrss.org/newest?q=copilot+agent&points=5&count=20|1"
  "HN:llm-safety|https://hnrss.org/newest?q=llm+safety&points=5&count=20|1"
  "HN:devin|https://hnrss.org/newest?q=devin+cognition&points=5&count=20|1"
  "HN:openai-codex|https://hnrss.org/newest?q=openai+codex&points=5&count=20|1"
  "HN:anthropic|https://hnrss.org/newest?q=anthropic&points=10&count=20|1"
  "HN:ai-agents|https://hnrss.org/newest?q=AI+agents&points=10&count=20|1"
  "HN:llm-eval|https://hnrss.org/newest?q=LLM+evaluation&points=5&count=20|1"
  "HN:ai-safety|https://hnrss.org/newest?q=AI+safety&points=10&count=20|1"
  "HN:rag-retrieval|https://hnrss.org/newest?q=RAG+retrieval&points=5&count=20|1"
  "HN:windsurf|https://hnrss.org/newest?q=windsurf+codeium&points=5&count=20|1"
  "HN:augment-code|https://hnrss.org/newest?q=augment+code+AI&points=5&count=20|1"
  # ── 核心博客 (14) ──
  "SimonWillison|https://simonwillison.net/atom/everything/|0"
  "MartinFowler|https://martinfowler.com/feed.atom|0"
  "LilianWeng|https://lilianweng.github.io/lil-log/feed.xml|0"
  "JayAlammar|https://jalammar.github.io/feed.xml|0"
  "ChipHuyen|https://huyenchip.com/feed|0"
  "AddyOsmani|https://addyo.substack.com/feed|0"
  "LatentSpace|https://www.latent.space/feed|0"
  "Karpathy|https://karpathy.github.io/feed.xml|0"
  "EugeneYan|https://eugeneyan.com/rss/|0"
  "Swyx|https://www.swyx.io/rss.xml|0"
  "OneUsefulThing|https://www.oneusefulthing.org/feed|0"
  "SebastianRaschka|https://magazine.sebastianraschka.com/feed|0"
  "BenEvans|https://www.ben-evans.com/benedictevans?format=rss|0"
  "Aider|https://aider.chat/feed.xml|0"
  # ── Newsletters / Substacks (14) ──
  "TheSequence|https://thesequence.substack.com/feed|0"
  "ImportAI|https://importai.substack.com/feed|0"
  "JackClark|https://jack-clark.net/feed/|0"
  "Interconnects|https://www.interconnects.ai/feed|0"
  "SemiAnalysis|https://www.semianalysis.com/feed|0"
  "ZviMowshowitz|https://thezvi.substack.com/feed|0"
  "AstralCodex|https://www.astralcodexten.com/feed|0"
  "ChinaAI|https://chinai.substack.com/feed|0"
  "Stratechery|https://stratechery.com/feed/|0"
  "NotBoring|https://www.notboring.co/feed|0"
  "BensBites|https://www.bensbites.com/feed|0"
  "PragmaticEngineer|https://newsletter.pragmaticengineer.com/feed|0"
  "AlignmentForum|https://www.alignmentforum.org/feed.xml|0"
  "LexFridman|https://lexfridman.com/feed/podcast/|0"
  # ── AI 新闻聚合 (4) ──
  "LastWeekInAI|https://lastweekin.ai/feed|0"
  "TransformerNews|https://www.transformernews.ai/feed|0"
  "AISafetyNewsletter|https://newsletter.safe.ai/feed|0"
  "PracticalAI|https://changelog.com/practicalai/feed|0"
  # ── 科技媒体 (6) ──
  "TechCrunch:AI|https://techcrunch.com/category/artificial-intelligence/feed/|0"
  "TheVerge:AI|https://www.theverge.com/rss/ai-artificial-intelligence/index.xml|0"
  "ArsTechnica|https://feeds.arstechnica.com/arstechnica/technology-lab|0"
  "MITTechReview|https://www.technologyreview.com/feed/|0"
  "VentureBeat:AI|https://venturebeat.com/category/ai/feed/|0"
  "InfoQ:AI|https://feed.infoq.com/ai-ml-data-eng|0"
  # ── 社区 (2) ──
  "Lobsters:ai|https://lobste.rs/t/ai.rss|0"
  "LessWrong|https://www.lesswrong.com/feed.xml|0"
  # ── 学术 (4) ──
  "arXiv:cs.SE|https://rss.arxiv.org/rss/cs.SE|0"
  "arXiv:cs.AI|https://export.arxiv.org/rss/cs.AI|0"
  "arXiv:cs.CL|https://export.arxiv.org/rss/cs.CL|0"
  "arXiv:cs.LG|https://export.arxiv.org/rss/cs.LG|0"
  # ── 行业官方 (17) ──
  "OpenAI|https://openai.com/blog/rss.xml|0"
  "AnthropicEng|https://raw.githubusercontent.com/conoro/anthropic-engineering-rss-feed/main/anthropic_engineering_rss.xml|0"
  "AnthropicNews|https://raw.githubusercontent.com/taobojlen/anthropic-rss-feed/main/anthropic_news_rss.xml|0"
  "LangChain|https://blog.langchain.com/rss|0"
  "HuggingFace|https://huggingface.co/blog/feed.xml|0"
  "GoogleResearch|https://research.google/blog/rss/|0"
  "GoogleDevBlog|https://developers.googleblog.com/feeds/posts/default|0"
  "DeepMind|https://deepmind.google/blog/rss.xml|0"
  "MetaEngineering|https://engineering.fb.com/feed/|0"
  "MSResearch|https://www.microsoft.com/en-us/research/feed/|0"
  "BAIR|https://bair.berkeley.edu/blog/feed.xml|0"
  "NvidiaDev|https://developer.nvidia.com/blog/feed|0"
  "Replit|https://blog.replit.com/feed.xml|0"
  "Vercel|https://vercel.com/atom|0"
  "Together|https://www.together.ai/blog/rss.xml|0"
  "CrewAI|https://blog.crewai.com/feed|0"
  "Sourcegraph|https://sourcegraph.com/blog/rss.xml|0"
  # ── 竞品 (2) ──
  "AugmentCode|https://www.augmentcode.com/blog/rss.xml|0"
  "Kagi|https://blog.kagi.com/rss.xml|0"
  # ── GitHub Releases (19) ──
  "GH:claude-code|https://github.com/anthropics/claude-code/releases.atom|1"
  "GH:openai-codex|https://github.com/openai/codex/releases.atom|1"
  "GH:aider|https://github.com/paul-gauthier/aider/releases.atom|1"
  "GH:cursor|https://github.com/getcursor/cursor/releases.atom|1"
  "GH:continue|https://github.com/continuedev/continue/releases.atom|1"
  "GH:copilot|https://github.com/microsoft/vscode-copilot-release/releases.atom|1"
  "GH:langchain|https://github.com/langchain-ai/langchain/releases.atom|1"
  "GH:llamaindex|https://github.com/run-llama/llama_index/releases.atom|1"
  "GH:dspy|https://github.com/stanfordnlp/dspy/releases.atom|1"
  "GH:ollama|https://github.com/ollama/ollama/releases.atom|1"
  "GH:crewai|https://github.com/crewAIInc/crewAI/releases.atom|1"
  "GH:autogen|https://github.com/microsoft/autogen/releases.atom|1"
  "GH:open-interpreter|https://github.com/OpenInterpreter/open-interpreter/releases.atom|1"
  "GH:swebench|https://github.com/princeton-nlp/SWE-bench/releases.atom|1"
  "GH:cline|https://github.com/cline/cline/releases.atom|1"
  "GH:zed|https://github.com/zed-industries/zed/releases.atom|1"
  "GH:tabnine|https://github.com/codota/tabnine-vscode/releases.atom|1"
  "GH:amazon-q|https://github.com/aws/amazon-q-developer-cli/releases.atom|1"
  "GH:bolt|https://github.com/stackblitz/bolt.new/releases.atom|1"
  # ── GitHub Blog (2) ──
  "GitHubBlog|https://github.blog/feed/|0"
  "GitHubChangelog|https://github.blog/changelog/feed/|0"
  # ── Rust 生态 (2) ──
  "ThisWeekInRust|https://this-week-in-rust.org/atom.xml|1"
  "RustBlog|https://blog.rust-lang.org/feed.xml|1"
  # ── YouTube AI (4) ──
  "YT:Fireship|https://www.youtube.com/feeds/videos.xml?channel_id=UCsBjURrPoezykLs9EqgamOA|0"
  "YT:YannicKilcher|https://www.youtube.com/feeds/videos.xml?channel_id=UCZHmQk67mSJgfCCTn7xBfew|0"
  "YT:TwoMinutePapers|https://www.youtube.com/feeds/videos.xml?channel_id=UCbfYPyITQ-7l4upoX8nvctg|0"
  "YT:AIExplained|https://www.youtube.com/feeds/videos.xml?channel_id=UCNJ1Ymd5yFuUPtn21xtRbbw|0"
  # ── MLOps / Infra (3) ──
  "MLOps|https://mlops.community/feed/|0"
  "ByteByteGo|https://blog.bytebytego.com/feed|0"
  "TheNewStack|https://thenewstack.io/feed/|0"
)

# ── 初始化 ────────────────────────────────────────────
mkdir -p "$DATA_DIR" "$OUTPUT_DIR"
touch "$SEEN_FILE"

TODAY=$(date +%Y-%m-%d)
REPORT="$OUTPUT_DIR/scout-${TODAY}.md"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

NEW_COUNT=0
TOTAL_COUNT=0

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" >&2; }

# ── 报告头 ────────────────────────────────────────────
{
  echo "# Knowledge Scout — $TODAY"
  echo ""
  echo "> 关键词: \`${KEYWORDS//|/ | }\`"
  echo "> 模式: $($DRY_RUN && echo 'dry-run' || echo 'production')"
  echo ""
} > "$REPORT"

# ── 主循环 ────────────────────────────────────────────
for feed_spec in "${FEEDS[@]}"; do
  IFS='|' read -r name url skip_filter <<< "$feed_spec"
  log "采集: $name"

  safe_name=$(printf '%s' "$name" | tr '/:' '__')
  local_file="$TEMP_DIR/${safe_name}.xml"

  # 下载
  if ! curl -sL --max-time 20 --retry 1 \
       -H 'User-Agent: rss-scout/1.0 (knowledge-discovery)' \
       "$url" -o "$local_file" 2>/dev/null; then
    log "  SKIP (下载失败): $name"
    continue
  fi

  [ ! -s "$local_file" ] && { log "  SKIP (空响应): $name"; continue; }

  # 解析 XML → JSONL
  raw_file="$TEMP_DIR/${safe_name}_raw.jsonl"
  if ! python3 "$PARSER" "$local_file" "$MAX_ITEMS" > "$raw_file" 2>/dev/null; then
    log "  SKIP (解析失败): $name"
    continue
  fi

  raw_count=$(wc -l < "$raw_file" | tr -d ' ')
  log "  解析 $raw_count 条"

  # 过滤：去重 + 关键词 + 归一化 (--compact 回写 seen.txt)
  filtered_file="$TEMP_DIR/${safe_name}_filtered.jsonl"
  compact_flag=""
  $DRY_RUN || compact_flag="--compact"
  python3 "$FILTER" "$raw_file" "$SEEN_FILE" "$KEYWORDS" "$skip_filter" "$name" $compact_flag > "$filtered_file" 2>/dev/null

  match=$(wc -l < "$filtered_file" | tr -d ' ')
  TOTAL_COUNT=$((TOTAL_COUNT + raw_count))

  [ "$match" -eq 0 ] && continue

  NEW_COUNT=$((NEW_COUNT + match))

  # 生成报告段落
  {
    echo "## $name ($match 条新发现)"
    echo ""
    # 用 python3 批量生成 markdown + 提取 link 列表
    python3 -c "
import json, sys
for line in open('$filtered_file'):
    line = line.strip()
    if not line: continue
    e = json.loads(line)
    print(f'- **{e[\"title\"]}**')
    print(f'  {e[\"link\"]}')
    if e.get('desc'):
        print(f'  _{e[\"desc\"]}..._')
    if e.get('date'):
        print(f'  \`{e[\"date\"]}\`')
" 2>/dev/null

    # seen.txt 已由 filter_entries.py --compact 回写，无需额外追加
    echo ""
  } >> "$REPORT"
done

# ── 报告尾 ────────────────────────────────────────────
{
  echo "---"
  echo ""
  echo "**统计**: 扫描 ${TOTAL_COUNT} 条 / 新发现 ${NEW_COUNT} 条 / 去重库 $(wc -l < "$SEEN_FILE" | tr -d ' ') 条"
  echo ""
  echo "_生成时间: $(date '+%Y-%m-%d %H:%M:%S')_"
} >> "$REPORT"

log "完成: ${NEW_COUNT} 新 / ${TOTAL_COUNT} 总"
log "报告: $REPORT"
echo "$REPORT"
