#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check public-host retirement without logging matched text or contact details."""
import html
from pathlib import Path
import re
import subprocess
import sys
import unicodedata
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
MAX_TEXT = 8 * 1024 * 1024  # The tracked Phaser declarations and chain specification exceed 4 MiB.
FORMATS = set('.md .rs .ts .tsx .js .mjs .json .toml .yml .yaml .html .css .sh .ps1 .py .txt .h .cpp .qml .conf .xml .svg .webmanifest .lock'.split())
PATTERN = re.compile(r'(?<![\w-])demiurge\.(?:cloud|guru|io|network)(?![\w-])', re.I)


def locations(content):
    result = []
    for number, value in enumerate(content.splitlines(), 1):
        for _ in range(2):
            value = html.unescape(unquote(value))
        if PATTERN.search(unicodedata.normalize('NFKC', value)):
            result.append(number)
    return result


def inspect(root=ROOT):
    root = root.resolve(strict=True)
    names = subprocess.check_output(['git', '-C', str(root), 'ls-files', '-z'], timeout=30)
    if len(names) > 4_000_000:
        raise ValueError('oversized list')
    count, failures = 0, []
    for name in names.decode('utf-8').split('\0'):
        relative = Path(name)
        if not name or any('secret' in p.lower() or p.lower().startswith('.env') for p in relative.parts):
            continue
        if relative.suffix not in FORMATS and relative.name not in ('Dockerfile', 'Containerfile', '.cursorrules'):
            continue
        if relative.is_absolute() or '..' in relative.parts:
            raise ValueError('unsafe path')
        path = root
        for part in relative.parts:
            path /= part
            if path.is_symlink() or (hasattr(path, 'is_junction') and path.is_junction()):
                raise ValueError('linked path')
        if not path.resolve().is_relative_to(root) or path.stat().st_size > MAX_TEXT:
            raise ValueError('unsafe text')
        with path.open('rb') as stream:
            data = stream.read(MAX_TEXT + 1)
        if len(data) > MAX_TEXT:
            raise ValueError('oversized text')
        failures.extend((name, number) for number in locations(data.decode('utf-8-sig')))
        count += 1
    return count, failures


def main():
    try:
        count, failures = inspect()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError):
        print('Public-domain scan could not inspect tracked text.', file=sys.stderr)
        return 2
    for name, number in failures:
        print(f'Retired public host at {ascii(name)}:{number}', file=sys.stderr)
    if failures:
        return 1
    print(f'Public-domain policy: {count} tracked text files checked.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
