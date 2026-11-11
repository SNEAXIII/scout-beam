#!/usr/bin/env python3
"""Project-specific smells no off-the-shelf linter reports. Advisory: it never fails a build.

Run: make sniff     (or: python scripts/sniff.py)
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC, MIGRATIONS = ROOT / "src", ROOT / "migrations"


def rel(path: Path) -> str:
    """Posix-style relative path, so output reads the same on both platforms."""
    return path.relative_to(ROOT).as_posix()


def section(title: str) -> None:
    print(f"\n=== {title} ===")


# --- 1. The same numeric bound rewritten in Rust more than once ---------------
# The Rust/CHECK pair is intended, so only Rust files count; bounds under 3 mean nothing.
MIN_BOUND = 3
RUST_BOUNDS = (re.compile(r"m(?:in|ax)\s*=\s*(\d+)"), re.compile(r"\((\d+)\.\.=(\d+)\)"))
SQL_BOUNDS = (re.compile(r"BETWEEN\s+(\d+)\s+AND\s+(\d+)"), re.compile(r"VARCHAR\((\d+)\)"))


def check_duplicated_bounds() -> None:
    section("bornes numeriques recopiees en Rust (>= 2 fichiers .rs)")
    rust_files: dict[str, set[str]] = defaultdict(set)
    sql_files: dict[str, set[str]] = defaultdict(set)

    for path, patterns, bucket in (
        *((p, RUST_BOUNDS, rust_files) for p in SRC.rglob("*.rs")),
        *((p, SQL_BOUNDS, sql_files) for p in MIGRATIONS.glob("*.sql")),
    ):
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern in patterns:
            for match in pattern.finditer(text):
                for value in match.groups():
                    if int(value) >= MIN_BOUND:
                        bucket[value].add(rel(path))

    for value in sorted(rust_files, key=int):
        files = rust_files[value]
        if len(files) < 2:
            continue
        context = f"   [aussi en base : {', '.join(sorted(sql_files[value]))}]" if sql_files.get(value) else ""
        print(f"  borne {value:<5} -> {len(files)} fichiers .rs : {', '.join(sorted(files))}{context}")


# --- 2. Production #[allow] without a written reason --------------------------
# Inner `#![allow]` in a test module is the convention, so only outer attributes count.
OUTER_ALLOW = re.compile(r"^\s*#\[allow\(")


def check_bare_allows() -> None:
    section("exemptions #[allow] sans justification (code de production)")
    for path in sorted(SRC.rglob("*.rs")):
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        for number, line in enumerate(lines, start=1):
            if not OUTER_ALLOW.match(line):
                continue
            previous = lines[number - 2] if number >= 2 else ""
            if "//" not in line and "//" not in previous:
                print(f"  {rel(path)}:{number}  {line.strip()}")


# --- 3. Services chaining two writes with no transaction ---------------------
# Each statement autocommits: if the second fails, the first stays. Some pairs are independent.
FN_START = re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?async fn (\w+)")
WRITE_CALL = re.compile(r"repository::(?:insert|update|set|mark|touch|delete|join|release|add|remove)")


def check_untransacted_writes() -> None:
    section("fonctions de service enchainant >= 2 ecritures sans transaction")
    for path in sorted(SRC.rglob("service.rs")):
        text = path.read_text(encoding="utf-8", errors="replace")
        if "begin(" in text or "Transaction" in text:
            continue
        name, writes = None, 0
        for line in text.splitlines():
            start = FN_START.match(line)
            if start:
                name, writes = start.group(1), 0
            elif WRITE_CALL.search(line):
                writes += 1
            elif line.startswith("}") and name:
                if writes >= 2:
                    print(f"  {rel(path)} :: {name} ({writes} ecritures)")
                name, writes = None, 0


# --- 4. Off-the-shelf tools worth having, if installed -----------------------
EXTERNAL = (
    ("cargo-machete", ["cargo", "machete"], "dependances inutilisees"),
    ("cargo-deny", ["cargo", "deny", "check", "advisories"], "avis de securite, licences"),
)


def check_external_tools() -> None:
    section("outils externes")
    for binary, command, purpose in EXTERNAL:
        if shutil.which(binary) is None:
            print(f"  {binary} absent ({purpose}) : cargo install {binary}")
            continue
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        for line in (result.stdout + result.stderr).splitlines()[-5:]:
            print(f"  {line}")


def main() -> int:
    for check in (check_duplicated_bounds, check_bare_allows, check_untransacted_writes, check_external_tools):
        check()
    print("\nRappel : chaque ligne est une question, pas un verdict.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
