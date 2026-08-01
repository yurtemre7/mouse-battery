#!/usr/bin/env python3
"""
SteelMouse CHANGELOG Generator
------------------------------
Generates a structured, categorized CHANGELOG.md based on git tags and commit history.
Reads current version from Cargo.toml.
"""

import datetime
import os
import re
import sys
import subprocess
from pathlib import Path

def get_current_cargo_version() -> str:
    cargo_toml = Path("Cargo.toml")
    if not cargo_toml.exists():
        return "Unknown"
    content = cargo_toml.read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    return match.group(1) if match else "Unknown"

def get_tags() -> list[str]:
    res = subprocess.run(
        ["git", "tag", "-l", "--sort=-v:refname"],
        capture_output=True,
        text=True,
        check=True,
    )
    return [t.strip() for t in res.stdout.splitlines() if t.strip()]

def get_tag_date(tag: str) -> str:
    res = subprocess.run(
        ["git", "log", "-1", "--format=%ad", "--date=short", tag],
        capture_output=True,
        text=True,
    )
    return res.stdout.strip() or "Unknown Date"

def get_commits_between(start_ref: str, end_ref: str) -> list[dict]:
    range_spec = f"{start_ref}..{end_ref}" if start_ref else end_ref
    cmd = ["git", "log", range_spec, "--format=%h|%an|%ad|%s", "--date=short"]
    res = subprocess.run(cmd, capture_output=True, text=True)
    
    commits = []
    for line in res.stdout.splitlines():
        if not line.strip():
            continue
        parts = line.split("|", 3)
        if len(parts) == 4:
            commits.append({
                "hash": parts[0],
                "author": parts[1],
                "date": parts[2],
                "subject": parts[3].strip(),
            })
    return commits

def categorize_commit(subject: str) -> str:
    subj_lower = subject.lower()
    if subj_lower.startswith("feat") or "release:" in subj_lower or "add" in subj_lower or "new" in subj_lower:
        return "Features"
    elif subj_lower.startswith("fix") or "bug" in subj_lower or "fix" in subj_lower or "broken" in subj_lower:
        return "Bug Fixes"
    elif subj_lower.startswith("refactor") or subj_lower.startswith("perf") or subj_lower.startswith("ci") or "architecture" in subj_lower or "decouple" in subj_lower:
        return "Performance & Architecture"
    elif subj_lower.startswith("docs") or subj_lower.startswith("test") or subj_lower.startswith("chore"):
        return "Documentation & Chores"
    else:
        return "Maintenance & Improvements"

def format_section(title: str, emoji: str, commits: list[dict]) -> str:
    if not commits:
        return ""
    lines = [f"### {emoji} {title}\n"]
    for c in commits:
        lines.append(f"- {c['subject']} ([`{c['hash']}`](https://github.com/yurtemre7/steel-mouse/commit/{c['hash']}))")
    lines.append("")
    return "\n".join(lines)

def generate_changelog() -> tuple[str, str]:
    cargo_version = get_current_cargo_version()
    current_tag_name = f"v{cargo_version}"
    today_date = datetime.date.today().isoformat()
    tags = get_tags()

    output = []
    output.append("# ⚡ SteelMouse Changelog\n\n")
    output.append(f"> **Current Version:** `{current_tag_name}`  \n")
    output.append("> All notable changes to SteelMouse are automatically documented in this file based on release tags and git commit history.\n\n")

    latest_release_notes = []

    # Handle commits on HEAD since latest tag
    if tags:
        latest_tag = tags[0]
        unreleased_commits = get_commits_between(latest_tag, "HEAD")
        if unreleased_commits:
            header_title = current_tag_name if current_tag_name not in tags else f"{current_tag_name}-next"
            output.append(f"## [{header_title}](https://github.com/yurtemre7/steel-mouse/releases/tag/{header_title}) - {today_date}\n\n")
            
            categories = {"Features": [], "Bug Fixes": [], "Performance & Architecture": [], "Documentation & Chores": [], "Maintenance & Improvements": []}
            for c in unreleased_commits:
                cat = categorize_commit(c["subject"])
                categories[cat].append(c)

            for title, emoji in [
                ("Features", "🚀"),
                ("Bug Fixes", "🐛"),
                ("Performance & Architecture", "⚡"),
                ("Documentation & Chores", "📝"),
                ("Maintenance & Improvements", "🔧"),
            ]:
                sec = format_section(title, emoji, categories[title])
                if sec:
                    output.append(sec)
                    latest_release_notes.append(sec)

            output.append("---\n\n")

    # Group commits tag by tag
    for i, tag in enumerate(tags):
        tag_date = get_tag_date(tag)
        prev_tag = tags[i + 1] if i + 1 < len(tags) else ""
        commits = get_commits_between(prev_tag, tag)

        output.append(f"## [{tag}](https://github.com/yurtemre7/steel-mouse/releases/tag/{tag}) - {tag_date}\n\n")
        
        categories = {"Features": [], "Bug Fixes": [], "Performance & Architecture": [], "Documentation & Chores": [], "Maintenance & Improvements": []}
        for c in commits:
            cat = categorize_commit(c["subject"])
            categories[cat].append(c)

        has_content = False
        tag_notes = []
        for title, emoji in [
            ("Features", "🚀"),
            ("Bug Fixes", "🐛"),
            ("Performance & Architecture", "⚡"),
            ("Documentation & Chores", "📝"),
            ("Maintenance & Improvements", "🔧"),
        ]:
            sec = format_section(title, emoji, categories[title])
            if sec:
                output.append(sec)
                tag_notes.append(sec)
                has_content = True

        if not has_content:
            output.append("_No detailed commit log for this release._\n\n")

        output.append("---\n\n")

        if not latest_release_notes and i == 0:
            latest_release_notes = tag_notes

    full_changelog = "".join(output)
    release_notes = "".join(latest_release_notes)
    return full_changelog, release_notes

def main():
    changelog_content, release_notes_content = generate_changelog()
    
    output_path = Path("CHANGELOG.md")
    output_path.write_text(changelog_content, encoding="utf-8")
    print(f"✅ CHANGELOG.md successfully updated for v{get_current_cargo_version()}!")

    notes_path = Path("RELEASE_NOTES.md")
    notes_path.write_text(release_notes_content, encoding="utf-8")
    print(f"✅ RELEASE_NOTES.md generated for GitHub Release page!")

if __name__ == "__main__":
    main()
