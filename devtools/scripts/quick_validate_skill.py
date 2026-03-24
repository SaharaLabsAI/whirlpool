#!/usr/bin/env python3
"""
Quick validation script for Codex skills in this repo.
"""

import re
import sys
from pathlib import Path

import yaml

MAX_SKILL_NAME_LENGTH = 64
ALLOWED_FRONTMATTER_KEYS = {
    "name",
    "description",
    "license",
    "allowed-tools",
    "metadata",
}


def extract_frontmatter(content: str):
    match = re.match(r"^---\r?\n(.*?)\r?\n---(?:\r?\n|$)", content, re.DOTALL)
    if not match:
        return None
    return match.group(1)


def validate_skill(skill_path: str):
    skill_dir = Path(skill_path)
    skill_md = skill_dir / "SKILL.md"

    if not skill_md.exists():
        return False, "SKILL.md not found"

    content = skill_md.read_text(encoding="utf-8")
    frontmatter_text = extract_frontmatter(content)
    if frontmatter_text is None:
        return False, "Invalid or missing YAML frontmatter"

    try:
        frontmatter = yaml.safe_load(frontmatter_text)
    except yaml.YAMLError as err:
        return False, f"Invalid YAML in frontmatter: {err}"

    if not isinstance(frontmatter, dict):
        return False, "Frontmatter must be a YAML dictionary"

    unexpected_keys = set(frontmatter) - ALLOWED_FRONTMATTER_KEYS
    if unexpected_keys:
        allowed = ", ".join(sorted(ALLOWED_FRONTMATTER_KEYS))
        unexpected = ", ".join(sorted(unexpected_keys))
        return (
            False,
            f"Unexpected key(s) in SKILL.md frontmatter: {unexpected}. Allowed properties are: {allowed}",
        )

    name = frontmatter.get("name")
    if name is None:
        return False, "Missing 'name' in frontmatter"
    if not isinstance(name, str):
        return False, f"Name must be a string, got {type(name).__name__}"
    if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name):
        return (
            False,
            f"Name '{name}' should be lowercase hyphen-case without leading, trailing, or repeated hyphens",
        )
    if len(name) > MAX_SKILL_NAME_LENGTH:
        return (
            False,
            f"Name is too long ({len(name)} characters). Maximum is {MAX_SKILL_NAME_LENGTH} characters.",
        )

    description = frontmatter.get("description")
    if description is None:
        return False, "Missing 'description' in frontmatter"
    if not isinstance(description, str):
        return False, f"Description must be a string, got {type(description).__name__}"
    description = description.strip()
    if not description:
        return False, "Description must not be empty"
    if "<" in description or ">" in description:
        return False, "Description cannot contain angle brackets (< or >)"
    if len(description) > 1024:
        return (
            False,
            f"Description is too long ({len(description)} characters). Maximum is 1024 characters.",
        )

    return True, "Skill is valid!"


def main():
    if len(sys.argv) != 2:
        print("Usage: python3 devtools/scripts/quick_validate_skill.py <skill_directory>")
        return 1

    valid, message = validate_skill(sys.argv[1])
    print(message)
    return 0 if valid else 1


if __name__ == "__main__":
    raise SystemExit(main())
