#!/usr/bin/env python3
"""从 JSONL 中过滤条目：去重 + 关键词匹配，输出 TSV"""
import json
import re
import sys


def main():
    if len(sys.argv) < 4:
        print("Usage: filter_entries.py <entries.jsonl> <seen.txt> <keywords_regex> [skip_filter]", file=sys.stderr)
        sys.exit(1)

    entries_file = sys.argv[1]
    seen_file = sys.argv[2]
    keywords_re = re.compile(sys.argv[3], re.IGNORECASE)
    skip_filter = sys.argv[4] == "1" if len(sys.argv) > 4 else False

    # 加载已见 URL
    seen = set()
    try:
        with open(seen_file) as f:
            seen = {line.strip() for line in f if line.strip()}
    except FileNotFoundError:
        pass

    with open(entries_file) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue

            title = entry.get("title", "")
            link = entry.get("link", "")
            date = entry.get("date", "")
            desc = entry.get("desc", "")

            if not title or not link:
                continue

            # 去重
            if link in seen:
                continue

            # 关键词过滤
            if not skip_filter:
                match_text = f"{title} {desc}"
                if not keywords_re.search(match_text):
                    continue

            # 清理 desc 中的 HTML 标签
            clean_desc = re.sub(r"<[^>]+>", "", desc)[:150]

            # 输出 JSON
            print(json.dumps({
                "title": title,
                "link": link,
                "date": date,
                "desc": clean_desc,
            }, ensure_ascii=False))


if __name__ == "__main__":
    main()
