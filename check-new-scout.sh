#!/usr/bin/env bash
# check-new-scout.sh — SessionStart hook，检测未读的 scout 报告
# 输出到 stderr 让 Claude Code 看到提示
set -euo pipefail

SCOUT_DIR="${RSS_SCOUT_DIR:-$HOME/.rss-scout}"
OUTPUT_DIR="$SCOUT_DIR/output"
READ_MARKER="$SCOUT_DIR/.last-read"

# 找到最新报告
latest=$(ls -t "$OUTPUT_DIR"/scout-*.md 2>/dev/null | head -1)
[ -z "$latest" ] && exit 0

# 检查是否已读
if [ -f "$READ_MARKER" ]; then
  last_read=$(cat "$READ_MARKER")
  [ "$latest" = "$last_read" ] && exit 0
fi

# 提取统计
stats=$(grep "^\*\*统计\*\*" "$latest" 2>/dev/null || echo "")
new_count=$(echo "$stats" | grep -oE '新发现 [0-9]+' | grep -oE '[0-9]+' || echo "0")

# 只在有新发现时提示
[ "$new_count" = "0" ] && exit 0

date_part=$(basename "$latest" .md | sed 's/scout-//')
echo "📡 Knowledge Scout: ${date_part} 发现 ${new_count} 条新内容，运行 /knowledge-scout 查看分析" >&2

# 标记为已读
echo "$latest" > "$READ_MARKER"
