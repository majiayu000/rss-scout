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

KEYWORDS="agentic|vibe.?cod|ai.?cod|coding.?agent|claude.?code|cursor.?ai|copilot|llm.?tool|ai.?engineer|context.?engineer|mcp|harness|code.?gen|ai.?pair|prompt.?engineer|llm.?agent|rag|fine.?tun"

# ── 源定义 (名称|URL|skip_filter) ─────────────────────
FEEDS=(
  # ── HN 服务端过滤 ──
  "HN:agentic-coding|https://hnrss.org/newest?q=agentic+coding&points=5&count=30|1"
  "HN:vibe-coding|https://hnrss.org/newest?q=vibe+coding&points=5&count=20|1"
  "HN:AI-coding-agent|https://hnrss.org/newest?q=AI+coding+agent&points=10&count=20|1"
  "HN:claude-code|https://hnrss.org/newest?q=claude+code&points=5&count=20|1"
  "HN:context-engineering|https://hnrss.org/newest?q=context+engineering&points=5&count=20|1"
  # ── 核心博客 ──
  "SimonWillison|https://simonwillison.net/atom/everything/|0"
  "MartinFowler|https://martinfowler.com/feed.atom|0"
  "LilianWeng|https://lilianweng.github.io/lil-log/feed.xml|0"
  "JayAlammar|https://jalammar.github.io/feed.xml|0"
  "ChipHuyen|https://huyenchip.com/feed|0"
  "AddyOsmani|https://addyo.substack.com/feed|0"
  "LatentSpace|https://www.latent.space/feed|0"
  # ── 社区 ──
  "Lobsters:ai|https://lobste.rs/t/ai.rss|0"
  "LessWrong|https://www.lesswrong.com/feed.xml|0"
  # ── 学术 ──
  "arXiv:cs.SE|https://rss.arxiv.org/rss/cs.SE|0"
  "arXiv:cs.AI|https://export.arxiv.org/rss/cs.AI|0"
  "arXiv:cs.CL|https://export.arxiv.org/rss/cs.CL|0"
  "arXiv:cs.LG|https://export.arxiv.org/rss/cs.LG|0"
  # ── 行业 ──
  "OpenAI|https://openai.com/blog/rss.xml|0"
  "LangChain|https://blog.langchain.com/rss|0"
  "HuggingFace|https://huggingface.co/blog/feed.xml|0"
  "GoogleResearch|https://research.google/blog/rss/|0"
  "BAIR|https://bair.berkeley.edu/blog/feed.xml|0"
  "NvidiaDev|https://developer.nvidia.com/blog/feed|0"
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

  # 过滤：去重 + 关键词
  filtered_file="$TEMP_DIR/${safe_name}_filtered.jsonl"
  python3 "$FILTER" "$raw_file" "$SEEN_FILE" "$KEYWORDS" "$skip_filter" > "$filtered_file" 2>/dev/null

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

    # 标记已见
    if ! $DRY_RUN; then
      python3 -c "
import json
for line in open('$filtered_file'):
    e = json.loads(line.strip())
    print(e['link'])
" >> "$SEEN_FILE" 2>/dev/null
    fi
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
