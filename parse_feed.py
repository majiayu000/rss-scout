#!/usr/bin/env python3
"""RSS/Atom feed 解析器 — 输出 JSONL"""
import json
import sys
import xml.etree.ElementTree as ET

ATOM_NS = {"a": "http://www.w3.org/2005/Atom"}


def parse_rss(root, max_items):
    for item in root.findall(".//item")[:max_items]:
        title = (item.findtext("title") or "").strip()
        link = (item.findtext("link") or "").strip()
        date = (item.findtext("pubDate") or "").strip()
        desc = (item.findtext("description") or "").strip()[:200]
        if title and link:
            yield {"title": title, "link": link, "date": date, "desc": desc}


def parse_atom(root, max_items):
    entries = root.findall(".//a:entry", ATOM_NS) or root.findall(".//entry")
    for entry in entries[:max_items]:
        title = (
            entry.findtext("a:title", "", ATOM_NS)
            or entry.findtext("title", "")
        ).strip()

        link = ""
        for link_el in entry.findall("a:link", ATOM_NS) or entry.findall("link"):
            href = link_el.get("href", "")
            rel = link_el.get("rel", "alternate")
            if href and rel == "alternate":
                link = href.strip()
                break
        if not link:
            link_el = entry.find("a:link", ATOM_NS) or entry.find("link")
            if link_el is not None:
                link = (link_el.get("href") or link_el.text or "").strip()

        date = (
            entry.findtext("a:published", "", ATOM_NS)
            or entry.findtext("a:updated", "", ATOM_NS)
            or entry.findtext("published", "")
            or entry.findtext("updated", "")
        ).strip()

        summary_el = entry.find("a:summary", ATOM_NS) or entry.find("summary")
        desc = ""
        if summary_el is not None and summary_el.text:
            desc = summary_el.text.strip()[:200]

        if title and link:
            yield {"title": title, "link": link, "date": date, "desc": desc}


def main():
    if len(sys.argv) < 2:
        print("Usage: parse_feed.py <file.xml> [max_items]", file=sys.stderr)
        sys.exit(1)

    xml_file = sys.argv[1]
    max_items = int(sys.argv[2]) if len(sys.argv) > 2 else 30

    try:
        tree = ET.parse(xml_file)
    except ET.ParseError as e:
        print(f"XML parse error: {e}", file=sys.stderr)
        sys.exit(1)

    root = tree.getroot()
    tag = root.tag.lower()

    if "feed" in tag:
        entries = parse_atom(root, max_items)
    else:
        entries = parse_rss(root, max_items)

    for entry in entries:
        print(json.dumps(entry, ensure_ascii=False))


if __name__ == "__main__":
    main()
