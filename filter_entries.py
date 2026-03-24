#!/usr/bin/env python3
"""从 JSONL 中过滤条目：去重 + 关键词匹配，输出 JSONL

seen.txt 格式: normalized_url|ISO-timestamp (每行)
支持: 自动过期 (>90天)、URL 归一化、跨源去重

arXiv 源使用增强过滤：复合关键词 + 负面排除 + 分类校验
"""
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from urllib.parse import parse_qs, urlencode, urlparse, urlunparse


# ── 去重配置 ────────────────────────────────────────────
SEEN_EXPIRE_DAYS = 90

# URL 归一化时要移除的追踪参数
STRIP_PARAMS = {
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "ref", "source", "via", "fbclid", "gclid", "mc_cid", "mc_eid",
}


# ── arXiv 增强过滤配置 ──────────────────────────────────

ARXIV_DOMAIN_ANCHORS = re.compile(
    r"(?i)(?:^|\W)(?:"
    r"large[\s\-]?language[\s\-]?model\w*|\bllm\w*\b|transformer\w*|\bgpt\w*\b|claude"
    r"|language[\s\-]?model\w*|neural\w*|deep[\s\-]?learn\w*|machine[\s\-]?learn\w*"
    r"|software[\s\-]?engineer\w*|code[\s\-]?generat\w*|program[\s\-]?synth\w*"
    r"|natural[\s\-]?language|\bnlp\b|chatbot\w*|conversational"
    r"|autonomous[\s\-]?agent|multi[\s\-]?agent|ai[\s\-]?agent"
    r"|reinforcement[\s\-]?learn\w*|\brlhf\b|reward[\s\-]?model"
    r"|diffusion[\s\-]?model|generative[\s\-]?ai|foundation[\s\-]?model"
    r"|benchmark\w*|evaluat\w+[\s\-]?(?:model|llm|agent)"
    r"|instruction[\s\-]?(?:tun|follow)\w*|in[\s\-]?context[\s\-]?learn\w*"
    r")"
)

ARXIV_TECH_SIGNALS = re.compile(
    r"(?i)(?:^|\W)(?:"
    r"retriev\w*[\s\-]+augment\w*[\s\-]+generat\w*"
    r"|rag[\s\-]?(?:system|pipeline|framework|retriev|chunk|vector|survey|code|approach)\w*"
    r"|ai[\s\-]?(?:align|safe)\w*"
    r"|align\w*[\s\-]+(?:llm|model|language|rlhf|preference|human)\w*"
    r"|red[\s\-]?team\w*|jailbreak\w*|guardrail\w*|safety[\s\-]?filter\w*"
    r"|agentic\w*|vibe[\s\-]?cod\w*|ai[\s\-]?cod\w*|coding[\s\-]?agent\w*|code[\s\-]?agent\w*"
    r"|claude[\s\-]?code|cursor[\s\-]?ai|copilot[\s\-]?(?:agent|chat|x)\w*"
    r"|context[\s\-]?engineer\w*|prompt[\s\-]?engineer\w*"
    r"|\bmcp\b|model[\s\-]?context[\s\-]?protocol"
    r"|tool[\s\-]?use|function[\s\-]?call\w*|tool[\s\-]?augment\w*"
    r"|fine[\s\-]?tun\w*|\blora\b|\bqlora\b|adapter\w*"
    r"|swe[\s\-]?bench|swe[\s\-]?agent|code[\s\-]?review"
    r"|chain[\s\-]?of[\s\-]?thought|\bcot\b|reason\w+[\s\-]?model"
    r"|tokeniz\w*|embed\w+[\s\-]?model|vector[\s\-]?(?:db|store|search)"
    r"|ai[\s\-]?pair|code[\s\-]?complet\w*|code[\s\-]?assist\w*"
    r"|\brust\b|\bcargo\b|\btokio\b|\bwasm\b"
    r")"
)

ARXIV_NEGATIVE = re.compile(
    r"(?i)(?:^|\W)(?:"
    r"medical|clinical|biomedic\w*|patholog\w*|radiology|diagnosis"
    r"|patient\w*|disease\w*|drug\w*|pharma\w*|health[\s\-]?care"
    r"|legal|jurisprud\w*|courtroom|litigation"
    r"|biolog\w*|genomic\w*|protein\w*|molecul\w*|\bdna\b|\brna\b|gene[\s\-]?express\w*"
    r"|astrono\w*|cosmolog\w*|astrophys\w*|galaxy|stellar"
    r"|quantum[\s\-]?(?:comput|mechan|field)\w*"
    r"|fluid[\s\-]?dynam\w*|thermodynam\w*"
    r"|seismic\w*|geolog\w*|climate[\s\-]?model"
    r"|optical[\s\-]?align|beam[\s\-]?align|laser[\s\-]?align"
    r"|sequence[\s\-]?align|structural[\s\-]?align"
    r"|crystal\w*|lattice\w*|phonon\w*"
    r")"
)

ARXIV_ALLOWED_CATEGORIES = {
    "cs.AI", "cs.CL", "cs.SE", "cs.LG", "cs.IR", "cs.MA",
    "cs.HC", "cs.CR", "cs.PL", "cs.FL", "stat.ML",
}


# ── URL 归一化 ──────────────────────────────────────────

def normalize_url(url: str) -> str:
    """URL 归一化: 去除追踪参数、统一 scheme、去除尾部斜杠、小写 host"""
    parsed = urlparse(url)

    scheme = "https" if parsed.scheme in ("http", "https") else parsed.scheme

    params = parse_qs(parsed.query, keep_blank_values=False)
    cleaned = {k: v for k, v in params.items() if k.lower() not in STRIP_PARAMS}
    query = urlencode(sorted(cleaned.items()), doseq=True)

    path = parsed.path.rstrip("/") or "/"

    return urlunparse((scheme, parsed.netloc.lower(), path, "", query, ""))


# ── seen.txt 读写 ───────────────────────────────────────

def load_seen(seen_file: str) -> dict[str, str]:
    """加载去重库, 返回 {normalized_url: iso_timestamp}

    兼容旧格式 (纯 URL, 无 '|') 和新格式 (URL|timestamp)。
    旧格式条目保留直到下次 compact 赋予时间戳。
    """
    seen: dict[str, str] = {}
    cutoff = datetime.now(timezone.utc) - timedelta(days=SEEN_EXPIRE_DAYS)
    expired = 0
    migrated = 0

    try:
        with open(seen_file) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue

                if "|" in line:
                    url_raw, ts = line.split("|", 1)
                else:
                    url_raw = line
                    ts = ""
                    migrated += 1

                if ts:
                    try:
                        if datetime.fromisoformat(ts) < cutoff:
                            expired += 1
                            continue
                    except ValueError:
                        pass

                seen[normalize_url(url_raw)] = ts
    except FileNotFoundError:
        pass

    if expired:
        print(f"[dedup] 过期移除 {expired} 条 (>{SEEN_EXPIRE_DAYS}天)", file=sys.stderr)
    if migrated:
        print(f"[dedup] 旧格式迁移 {migrated} 条", file=sys.stderr)

    return seen


def save_seen(seen_file: str, seen: dict[str, str]) -> None:
    """回写去重库 (已过期的不会出现在 seen 中)"""
    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with open(seen_file, "w") as f:
        for url in sorted(seen):
            ts = seen[url] or now_iso
            f.write(f"{url}|{ts}\n")


# ── arXiv 过滤 ──────────────────────────────────────────

def is_arxiv_source(source_name: str) -> bool:
    return source_name.lower().startswith("arxiv")


def arxiv_category_ok(entry: dict) -> bool:
    """检查 arXiv 论文是否属于白名单分类。"""
    text = f"{entry.get('link', '')} {entry.get('desc', '')} {entry.get('title', '')}"
    categories = re.findall(r"\b((?:cs|stat)\.\w{2})\b", text)
    if not categories:
        return True
    return any(cat in ARXIV_ALLOWED_CATEGORIES for cat in categories)


def passes_arxiv_filter(entry: dict) -> bool:
    """arXiv 三层增强过滤。

    Layer 1: 负面关键词排除
    Layer 2: 复合关键词 (领域 AND 技术)
    Layer 3: 分类校验
    """
    match_text = f"{entry.get('title', '')} {entry.get('desc', '')}"

    if ARXIV_NEGATIVE.search(match_text):
        return False
    if not (ARXIV_DOMAIN_ANCHORS.search(match_text) and ARXIV_TECH_SIGNALS.search(match_text)):
        return False
    if not arxiv_category_ok(entry):
        return False

    return True


# ── 主函数 ──────────────────────────────────────────────

def main():
    if len(sys.argv) < 4:
        print(
            "Usage: filter_entries.py <entries.jsonl> <seen.txt> "
            "<keywords_regex> [skip_filter] [source_name] [--compact]",
            file=sys.stderr,
        )
        sys.exit(1)

    entries_file = sys.argv[1]
    seen_file = sys.argv[2]
    keywords_re = re.compile(sys.argv[3], re.IGNORECASE)

    skip_filter = sys.argv[4] == "1" if len(sys.argv) > 4 else False
    source_name = sys.argv[5] if len(sys.argv) > 5 else ""

    compact_mode = "--compact" in sys.argv

    use_arxiv_filter = is_arxiv_source(source_name)

    # 加载去重库 (自动过期)
    seen = load_seen(seen_file)

    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    new_count = 0

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

            # 归一化去重
            norm_link = normalize_url(link)
            if norm_link in seen:
                continue

            # 关键词过滤
            if not skip_filter:
                if use_arxiv_filter:
                    if not passes_arxiv_filter(entry):
                        continue
                else:
                    match_text = f"{title} {desc}"
                    if not keywords_re.search(match_text):
                        continue

            # 清理 desc 中的 HTML 标签
            clean_desc = re.sub(r"<[^>]+>", "", desc)[:150]

            # 标记为已见
            seen[norm_link] = now_iso
            new_count += 1

            print(json.dumps({
                "title": title,
                "link": link,
                "date": date,
                "desc": clean_desc,
            }, ensure_ascii=False))

    # compact 模式: 回写 seen.txt (含过期清理 + 新条目 + 旧格式迁移)
    if compact_mode:
        save_seen(seen_file, seen)
        print(f"[dedup] 库大小 {len(seen)} 条, 本次新增 {new_count} 条", file=sys.stderr)


if __name__ == "__main__":
    main()
