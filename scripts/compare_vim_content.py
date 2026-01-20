#!/usr/bin/env python3
"""
Compare vim-highlighted source code content between two HTML snapshots.

Strips HTML tags from vimhighlight sections and compares the plain text,
ignoring differences in syntax highlighting spans.
"""

import re
import sys
from pathlib import Path
from html import unescape
from typing import Optional


def extract_vimhighlight_sections(html: str) -> list[str]:
    """Extract all vimhighlight div contents from HTML."""
    # Find all <div class="vimhighlight">...</div> sections
    pattern = r'<div class="vimhighlight">(.*?)</div>'
    matches = re.findall(pattern, html, re.DOTALL)
    return matches


def strip_html_tags(text: str) -> str:
    """Remove all HTML tags and decode entities."""
    # Remove all HTML tags
    text = re.sub(r'<[^>]+>', '', text)
    # Decode HTML entities
    text = unescape(text)
    return text


def normalize_whitespace(text: str, normalize_tabs: bool = False) -> str:
    """Normalize whitespace for comparison."""
    # Normalize line endings
    text = text.replace('\r\n', '\n').replace('\r', '\n')
    # Optionally normalize tabs to spaces (vim may expand tabs differently)
    if normalize_tabs:
        text = text.replace('\t', '        ')  # 8 spaces (vim default tabstop)
    # Remove trailing whitespace on each line
    lines = [line.rstrip() for line in text.split('\n')]
    return '\n'.join(lines)


def compare_vimhighlight_content(file1: Path, file2: Path, normalize_tabs: bool = False) -> tuple[bool, str]:
    """
    Compare vim-highlighted content between two HTML files.

    Returns (is_identical, diff_message)
    """
    html1 = file1.read_text(encoding='utf-8')
    html2 = file2.read_text(encoding='utf-8')

    sections1 = extract_vimhighlight_sections(html1)
    sections2 = extract_vimhighlight_sections(html2)

    if len(sections1) != len(sections2):
        return False, f"Different number of vimhighlight sections: {len(sections1)} vs {len(sections2)}"

    if len(sections1) == 0:
        return True, "No vimhighlight sections found"

    for i, (s1, s2) in enumerate(zip(sections1, sections2)):
        text1 = normalize_whitespace(strip_html_tags(s1), normalize_tabs)
        text2 = normalize_whitespace(strip_html_tags(s2), normalize_tabs)

        if text1 != text2:
            # Find first difference
            lines1 = text1.split('\n')
            lines2 = text2.split('\n')

            diff_lines = []
            for j, (l1, l2) in enumerate(zip(lines1, lines2)):
                if l1 != l2:
                    diff_lines.append(f"  Line {j+1}:")
                    diff_lines.append(f"    prod:  {repr(l1[:100])}")
                    diff_lines.append(f"    local: {repr(l2[:100])}")
                    if len(diff_lines) > 15:
                        diff_lines.append("  ... (more differences)")
                        break

            if len(lines1) != len(lines2):
                diff_lines.append(f"  Different line counts: {len(lines1)} vs {len(lines2)}")

            return False, f"Section {i+1} content differs:\n" + '\n'.join(diff_lines)

    return True, f"All {len(sections1)} vimhighlight section(s) match"


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Compare vim-highlighted source code content between two HTML snapshots."
    )
    parser.add_argument("prod_dir", help="Production snapshot directory")
    parser.add_argument("local_dir", help="Local snapshot directory")
    parser.add_argument("file", nargs="?", help="Optional: single file to compare")
    parser.add_argument(
        "--normalize-tabs",
        action="store_true",
        help="Normalize tabs to 8 spaces (handles vim tabstop differences)"
    )

    args = parser.parse_args()

    prod_dir = Path(args.prod_dir)
    local_dir = Path(args.local_dir)
    normalize_tabs = args.normalize_tabs

    if args.file:
        # Compare single file
        files = [args.file]
    else:
        # Compare all HTML files
        files = sorted([f.name for f in prod_dir.glob("*.html")])

    all_match = True
    files_with_code = 0
    files_checked = 0

    for filename in files:
        prod_file = prod_dir / filename
        local_file = local_dir / filename

        if not prod_file.exists():
            print(f"SKIP: {filename} - not in prod")
            continue
        if not local_file.exists():
            print(f"SKIP: {filename} - not in local")
            continue

        files_checked += 1
        is_match, message = compare_vimhighlight_content(prod_file, local_file, normalize_tabs)

        if "No vimhighlight sections" not in message:
            files_with_code += 1

        if is_match:
            if "No vimhighlight sections" not in message:
                print(f"OK:   {filename} - {message}")
        else:
            all_match = False
            print(f"DIFF: {filename}")
            print(message)
            print()

    print()
    print(f"Summary: {files_checked} files checked, {files_with_code} with vim-highlighted code")

    if all_match:
        print("All vim-highlighted content matches!")
        sys.exit(0)
    else:
        print("Some files have content differences!")
        sys.exit(1)


if __name__ == "__main__":
    main()
