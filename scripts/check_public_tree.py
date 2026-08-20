#!/usr/bin/env python3
"""Scan every tracked file as bytes without printing matched content."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


FLAGS = re.IGNORECASE | re.DOTALL
IDENTIFIER = rb"(?:[0-9a-f]{32}|[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12})"

# Build sensitive literals in pieces so this scanner does not match its source.
USER_HOME = re.compile(
    rb"(?:"
    rb"/" + rb"Users" + rb"/(?!<user>/)[^/\x00\s]+/"
    rb"|/" + rb"home" + rb"/(?!<user>/)[^/\x00\s]+/"
    rb"|[a-z]:[\\/]" + rb"Users" + rb"[\\/](?!<user>[\\/])[^\\/\x00\r\n]+[\\/]"
    rb")",
    FLAGS,
)
PRIVATE_LABEL = re.compile(rb"com[.]" + rb"lifcc", FLAGS)
PRIVATE_REPO = re.compile(
    rb"(?:~|/" + rb"Users" + rb"/[^/]+)/Desktop/code/AI/" + rb"tools/", FLAGS
)
NOTION_FIELD = re.compile(
    rb"(?:notion[_-]?)?(?:database|data[_-]?source)[_-]?id"
    rb"[\s\"']*[:=][\s\"']*"
    + IDENTIFIER,
    FLAGS,
)
NOTION_URL = re.compile(
    rb"https?://(?:[a-z0-9-]+[.])?(?:notion[.]so|notion[.]site)/"
    rb"[^\s\"'<>]*"
    + IDENTIFIER
    + rb"(?:[^0-9a-f]|$)",
    FLAGS,
)
NOTION_TOKEN = re.compile(
    rb"notion[_-]?(?:api[_-]?)?(?:key|token)"
    rb"[\s\"']*[:=][\s\"']*"
    rb"(?:ntn|secret)_[a-z0-9_-]{16,}",
    FLAGS,
)
LOOPBACK_FEED = re.compile(
    rb"url\s*=\s*[\"']https?://"
    rb"(?:127[.]0[.]0[.]1|localhost|\[::1\])"
    rb"(?::[0-9]+)?(?=[/\"'?#\s])",
    FLAGS,
)


def tracked_paths(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "-C", os.fspath(root), "ls-files", "-z"],
        check=True,
        stdout=subprocess.PIPE,
    )
    return [root / os.fsdecode(name) for name in result.stdout.split(b"\0") if name]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()

    findings: list[tuple[str, str]] = []
    for path in tracked_paths(root):
        relative = path.relative_to(root).as_posix()
        try:
            if path.is_symlink():
                data = os.fsencode(os.readlink(path))
            else:
                data = path.read_bytes()
        except OSError as error:
            print(f"public-tree scan error: cannot read tracked file {relative}: {error}", file=sys.stderr)
            return 2

        checks = [
            ("private user path", USER_HOME),
            ("private launchd label", PRIVATE_LABEL),
            ("machine-specific repository path", PRIVATE_REPO),
            ("real Notion database/data-source identifier", NOTION_FIELD),
            ("real Notion identifier in URL", NOTION_URL),
            ("real Notion API credential", NOTION_TOKEN),
        ]
        if Path(relative).match("feeds*.toml"):
            checks.append(("loopback URL in canonical feed config", LOOPBACK_FEED))

        for label, pattern in checks:
            if pattern.search(data):
                findings.append((label, relative))

    if findings:
        for label, relative in findings:
            print(f"public-tree scan failed: {label}: {relative}", file=sys.stderr)
        return 1

    print("public-tree scan passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
