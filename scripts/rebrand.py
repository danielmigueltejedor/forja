#!/usr/bin/env python3
"""Rename an upstream product to this app's own name, in file contents and in paths.

The rules live in a TOML file (default: `.github/rebrand.toml`), so the same script serves every
app that is built on an upstream project: each repository keeps its own rules.

    rebrand.py apply [--root DIR] [--pin-from REF]
                                    rewrite the tracked files under DIR (default: the repository);
                                    with REF, lines named by [[pin]] rules take REF's value
    rebrand.py check [--root DIR]   list what the rules would still change; exit 1 if anything

`apply` is what the weekly upstream sync runs on upstream's tree before merging it, so upstream's
changes arrive already speaking this app's names. `check` is the guard that keeps old names from
coming back by hand.

Rules (see the config file for this repository's):

    [[replace]]  from = "...", to = "..."   literal, case-sensitive, applied in order to file
                                            contents and to every path
    keep = ["regex", ...]                    spans that stay as they are (URLs, upstream's repo...)
    skip = ["glob", ...]                     files left alone entirely (licence notices...)
    [[pin]]  file = "...", pattern = "regex" the first match in `file` takes the value it has in the
                                            --pin-from checkout (this app's own version number)
    after = [["cmd", "arg", ...], ...]       run in DIR after `apply` (a formatter: shorter names
                                            change how lines wrap and imports sort)

A line containing the marker `rebrand: keep` is never changed, for compatibility code that has to
name the old product on purpose.
"""

from __future__ import annotations

import argparse
import fnmatch
import re
import subprocess
import sys
import tomllib
from pathlib import Path

MARKER = "rebrand: keep"


class Rules:
    def __init__(self, config: Path):
        data = tomllib.loads(config.read_text(encoding="utf-8"))
        self.pairs = [(r["from"], r["to"]) for r in data.get("replace", [])]
        if not self.pairs:
            sys.exit(f"{config}: no [[replace]] rules")
        self.keep = [re.compile(k) for k in data.get("keep", [])]
        self.skip = list(data.get("skip", []))
        self.pins = [(p["file"], re.compile(p["pattern"], re.M)) for p in data.get("pin", [])]
        self.after = [list(c) for c in data.get("after", [])]
        sources = "|".join(re.escape(f) for f, _ in self.pairs)
        self.any_source = re.compile(sources)

    def skipped(self, path: str) -> bool:
        return any(fnmatch.fnmatch(path, g) for g in self.skip)

    def line(self, text: str) -> str:
        if MARKER in text or not self.any_source.search(text):
            return text
        kept: list[str] = []

        def hold(m: re.Match) -> str:
            kept.append(m.group(0))
            return f"\0{len(kept) - 1}\0"

        for k in self.keep:
            text = k.sub(hold, text)
        for f, t in self.pairs:
            text = text.replace(f, t)
        return re.sub(r"\0(\d+)\0", lambda m: kept[int(m.group(1))], text)

    def path(self, p: str) -> str:
        for f, t in self.pairs:
            p = p.replace(f, t)
        return p


def tracked(root: Path) -> list[str]:
    out = subprocess.run(["git", "ls-files", "-z"], cwd=root, check=True, capture_output=True).stdout
    return [p for p in out.decode("utf-8").split("\0") if p]


def text_of(path: Path) -> str | None:
    """The file as text, or None for a binary (a NUL byte, or not UTF-8) or a symlink."""
    if path.is_symlink() or not path.is_file():
        return None
    data = path.read_bytes()
    if b"\0" in data[:8192]:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def rewrite(rules: Rules, text: str) -> str:
    return "".join(rules.line(l) for l in text.splitlines(keepends=True))


def apply(rules: Rules, root: Path) -> int:
    changed = renamed = 0
    emptied: set[Path] = set()
    for rel in tracked(root):
        if rules.skipped(rel):
            continue
        p = root / rel
        text = text_of(p)
        if text is not None:
            new = rewrite(rules, text)
            if new != text:
                p.write_bytes(new.encode("utf-8"))
                changed += 1
        dest = rules.path(rel)
        if dest != rel:
            # `git mv` keeps the rename visible to git and works in a bare checkout of upstream.
            (root / dest).parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(["git", "mv", "-f", "--", rel, dest], cwd=root, check=True)
            renamed += 1
            emptied.add((root / rel).parent)
    # Directories left empty by the renames (git does not track directories, but cargo sees them).
    for d in sorted(emptied, key=lambda d: len(d.parts), reverse=True):
        while d != root and d.is_dir() and not any(d.iterdir()):
            d.rmdir()
            d = d.parent
    print(f"rebrand: {changed} files rewritten, {renamed} paths renamed")
    return 0


def pin(rules: Rules, root: Path, reference: Path) -> None:
    """Give each pinned line the value it has in `reference` (this app's own checkout)."""
    for file, pattern in rules.pins:
        ours, theirs = reference / file, root / file
        if not ours.is_file() or not theirs.is_file():
            continue
        want = pattern.search(ours.read_text(encoding="utf-8"))
        text = theirs.read_text(encoding="utf-8")
        if want and pattern.search(text):
            theirs.write_text(pattern.sub(lambda _m: want.group(0), text, count=1), encoding="utf-8")


def check(rules: Rules, root: Path) -> int:
    found = 0
    for rel in tracked(root):
        if rules.skipped(rel):
            continue
        if rules.path(rel) != rel:
            print(f"{rel}: path still uses an upstream name")
            found += 1
        text = text_of(root / rel)
        if text is None:
            continue
        for n, l in enumerate(text.splitlines(), 1):
            if rules.line(l) != l:
                print(f"{rel}:{n}: {l.strip()[:160]}")
                found += 1
    if found:
        print(f"rebrand check: {found} places still use upstream names. Run `scripts/rebrand.py apply`,"
              f" or mark deliberate ones with `{MARKER}`.", file=sys.stderr)
        return 1
    print("rebrand check: clean")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("action", choices=["apply", "check"])
    ap.add_argument("--root", type=Path, default=None, help="checkout to work on (default: this repository)")
    ap.add_argument("--pin-from", type=Path, default=None, help="checkout whose values [[pin]] lines keep")
    ap.add_argument("--config", type=Path, default=None, help="rules (default: <this repository>/.github/rebrand.toml)")
    a = ap.parse_args()
    here = Path(__file__).resolve().parent.parent
    root = (a.root or here).resolve()
    config = (a.config or here / ".github" / "rebrand.toml").resolve()
    rules = Rules(config)
    if a.action == "check":
        return check(rules, root)
    status = apply(rules, root)
    if a.pin_from is not None:
        pin(rules, root, a.pin_from.resolve())
    for cmd in rules.after:
        subprocess.run(cmd, cwd=root, check=True)
    return status


if __name__ == "__main__":
    sys.exit(main())
