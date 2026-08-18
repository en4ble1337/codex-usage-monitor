#!/usr/bin/env python3
"""
Launchpad Setup Script
Creates project scaffold based on docs/PRD.md, docs/ARCH.md, and docs/RESEARCH.md.
"""

from __future__ import annotations

import os
import re
import shutil
import stat
import subprocess
import sys
import textwrap
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Tuple


ROOT = Path(__file__).resolve().parent
DOCS_DIR = ROOT / "docs"
DOC_PATHS = {
    "PRD": DOCS_DIR / "PRD.md",
    "ARCH": DOCS_DIR / "ARCH.md",
    "RESEARCH": DOCS_DIR / "RESEARCH.md",
}

BASE_DIRECTORIES = [
    "docs",
    "docs/plans",
    "docs/methodology",
    "directives",
    "execution",
    "src",
    "tests",
    ".tmp",
]


def read_required_docs() -> Dict[str, str]:
    """Read the existing planning documents from docs/."""
    docs: Dict[str, str] = {}
    missing = [str(path.relative_to(ROOT)) for path in DOC_PATHS.values() if not path.exists()]
    if missing:
        raise FileNotFoundError(
            "Missing required planning documents: " + ", ".join(missing)
        )

    for key, path in DOC_PATHS.items():
        docs[key] = path.read_text(encoding="utf-8")
    return docs


def normalize_heading(title: str) -> str:
    title = title.strip().strip("#").strip()
    title = re.sub(r"^\d+[\.)]\s*", "", title)
    title = re.sub(r"\s+", " ", title)
    return title.lower()


def extract_section(markdown: str, heading: str) -> str:
    """Return a Markdown section by heading text, ignoring numeric prefixes."""
    target = normalize_heading(heading)
    matches = list(re.finditer(r"^(#{1,6})\s+(.+?)\s*$", markdown, re.MULTILINE))

    for index, match in enumerate(matches):
        level = len(match.group(1))
        title = normalize_heading(match.group(2))
        if title != target:
            continue

        start = match.end()
        end = len(markdown)
        for next_match in matches[index + 1 :]:
            next_level = len(next_match.group(1))
            if next_level <= level:
                end = next_match.start()
                break
        return markdown[start:end].strip()

    return ""


def strip_inline_markdown(value: str) -> str:
    value = re.sub(r"`([^`]*)`", r"\1", value)
    value = re.sub(r"\*\*([^*]*)\*\*", r"\1", value)
    value = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", value)
    return value.strip()


def parse_markdown_table(section: str) -> List[Dict[str, str]]:
    """Parse simple pipe tables from a Markdown section."""
    rows: List[Dict[str, str]] = []
    headers: List[str] | None = None

    for raw_line in section.splitlines():
        line = raw_line.strip()
        if not line.startswith("|") or not line.endswith("|"):
            continue

        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if not cells:
            continue

        is_separator = all(
            cell and set(cell.replace(":", "").strip()) <= {"-"} for cell in cells
        )
        if is_separator:
            continue

        if headers is None:
            headers = [strip_inline_markdown(cell) for cell in cells]
            continue

        if len(cells) != len(headers):
            continue

        row = {
            header: strip_inline_markdown(cell)
            for header, cell in zip(headers, cells)
        }
        rows.append(row)

    return rows


def first_paragraph(section: str) -> str:
    lines: List[str] = []
    for raw_line in section.splitlines():
        line = raw_line.strip()
        if not line:
            if lines:
                break
            continue
        if line.startswith("#") or line.startswith("|"):
            continue
        lines.append(strip_inline_markdown(line))
    return " ".join(lines).strip()


def first_sentence(text: str) -> str:
    text = re.sub(r"\s+", " ", text).strip()
    if not text:
        return ""
    parts = re.split(r"(?<=[.!?])\s+", text, maxsplit=1)
    return parts[0].strip()


def extract_project_name(prd: str) -> str:
    summary = extract_section(prd, "Executive Summary")
    paragraph = first_paragraph(summary)
    match = re.search(r"\b([A-Z][A-Za-z0-9_-]{1,40})\s+is\s+", paragraph)
    if match:
        return match.group(1)

    title_match = re.search(r"^#\s+(.+?)\s*$", prd, re.MULTILINE)
    if title_match:
        title = strip_inline_markdown(title_match.group(1))
        title = re.sub(r"^PRD:\s*", "", title, flags=re.IGNORECASE).strip()
        if title:
            return title.split()[0]

    return "Project"


def extract_project_description(prd: str) -> str:
    summary = extract_section(prd, "Executive Summary")
    description = first_sentence(first_paragraph(summary))
    return description or "A local-first project generated from the product requirements."


def extract_dictionary_terms(arch: str) -> List[Tuple[str, str]]:
    rows = parse_markdown_table(extract_section(arch, "Dictionary"))
    terms: List[Tuple[str, str]] = []
    for row in rows:
        term = row.get("Term", "").strip()
        definition = row.get("Definition", "").strip()
        if term:
            terms.append((term, definition))
    return terms


def extract_tech_stack(arch: str) -> List[Dict[str, str]]:
    rows = parse_markdown_table(extract_section(arch, "Tech Stack"))
    return [
        row
        for row in rows
        if row.get("Layer") and row.get("Technology") and row.get("Version")
    ]


def stack_summary(stack: Sequence[Dict[str, str]]) -> str:
    parts: List[str] = []
    seen: set[str] = set()
    for row in stack:
        technology = row.get("Technology", "").strip()
        version = row.get("Version", "").strip()
        if not technology or technology.lower() == "n/a":
            continue
        key = technology.lower()
        if key in seen:
            continue
        seen.add(key)
        if version and version.lower() != "n/a":
            parts.append(f"{technology} {version}")
        else:
            parts.append(technology)
    return "; ".join(parts) or "See docs/ARCH.md Tech Stack."


def normalize_repo_dir(path_text: str) -> str | None:
    path_text = strip_inline_markdown(path_text).strip()
    path_text = path_text.replace("\\", "/")
    path_text = path_text.strip("/")
    if not path_text:
        return None
    if path_text.startswith(("%", "$", "~")):
        return None
    if "://" in path_text:
        return None
    if re.search(r"\.(json|ndjson|log|md|toml|yaml|yml|rs|tsx|ts|sh|html)$", path_text):
        return None
    return path_text


def add_parent_directories(paths: Iterable[str]) -> List[str]:
    expanded: set[str] = set()
    for path in paths:
        normalized = normalize_repo_dir(path)
        if not normalized:
            continue
        parts = normalized.split("/")
        for index in range(1, len(parts) + 1):
            expanded.add("/".join(parts[:index]))
    return sorted(expanded, key=lambda value: (value.count("/"), value))


def extract_directories(arch: str) -> List[str]:
    section = extract_section(arch, "Directory Structure")
    repo_section = section.split("Storage is outside the repository:", 1)[0]
    rows = parse_markdown_table(repo_section)
    directories: set[str] = set(BASE_DIRECTORIES)

    for row in rows:
        raw_path = row.get("Path", "")
        if not raw_path:
            continue

        matches = re.findall(r"`([^`]+)`", raw_path) or [raw_path]
        for match in matches:
            normalized = normalize_repo_dir(match)
            if normalized:
                directories.add(normalized)

    return add_parent_directories(directories)


def directory_tree(directories: Sequence[str]) -> str:
    tree: Dict[str, dict] = {}
    for directory in add_parent_directories(directories):
        node = tree
        for part in directory.split("/"):
            node = node.setdefault(part, {})

    def sort_key(name: str) -> Tuple[int, str]:
        return (0 if name.startswith(".") else 1, name.lower())

    lines: List[str] = []

    def walk(node: Dict[str, dict], depth: int) -> None:
        for name in sorted(node, key=sort_key):
            lines.append(f"{'  ' * depth}{name}/")
            walk(node[name], depth + 1)

    walk(tree, 0)
    return "\n".join(lines)


def extract_env_vars(arch: str) -> List[str]:
    security = extract_section(arch, "Security Considerations")
    integrations = extract_section(arch, "Integration Points")
    candidates = set(re.findall(r"\b[A-Z][A-Z0-9_]{2,}\b", security + "\n" + integrations))
    env_vars = {
        value
        for value in candidates
        if "_" in value
        and not value.endswith("_ERROR")
        and not value.startswith("XDG_")
    }
    return sorted(env_vars)


def write_file(relative_path: str, content: str, executable: bool = False) -> None:
    path = ROOT / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    normalized = textwrap.dedent(content).lstrip("\n").rstrip() + "\n"
    path.write_text(normalized, encoding="utf-8", newline="\n")
    if executable:
        path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def create_directories(directories: Sequence[str]) -> None:
    for directory in directories:
        (ROOT / directory).mkdir(parents=True, exist_ok=True)


def create_gitignore() -> None:
    write_file(
        ".gitignore",
        """
        # Python
        __pycache__/
        *.py[cod]
        *$py.class
        .venv/
        venv/
        env/
        *.egg-info/
        dist/
        build/

        # Environment
        .env
        .env.local
        *.local

        # IDE
        .idea/
        .vscode/
        *.swp
        *.swo

        # Node / pnpm
        node_modules/
        .pnpm-store/
        npm-debug.log*
        pnpm-debug.log*
        yarn-debug.log*
        yarn-error.log*
        coverage/

        # Rust / Tauri
        target/
        **/target/
        apps/desktop/dist/
        apps/desktop/src-tauri/target/
        *.msi
        *.app
        *.dmg
        *.deb
        *.rpm
        *.AppImage

        # Local app state and secrets
        *.log
        logs/
        *.tmp
        *.secret

        # Project
        .tmp/
        """,
    )


def create_env_example(project_name: str, env_vars: Sequence[str]) -> None:
    lines = [
        f"# {project_name} Environment Example",
        "# Copy this file to .env for local development overrides.",
        "# Do not commit real secrets.",
        "",
        "# Provider credentials are owned by local provider tools.",
        "# Do not place provider account tokens or API keys here.",
        "",
    ]

    if env_vars:
        lines.append("# Optional local overrides")
        for env_var in env_vars:
            lines.append(f"{env_var}=")
    else:
        lines.extend(
            [
                "# No required environment variables are defined by the current architecture.",
                "# Optional alert or provider overrides can be added here after ARCH.md defines them.",
            ]
        )

    write_file(".env.example", "\n".join(lines))


def create_readme(project_name: str, description: str, tree_text: str) -> None:
    tree_block = textwrap.indent(tree_text, "        ")
    write_file(
        "README.md",
        f"""
        # {project_name}

        {description}

        ## Quick Start

        1. Clone the repository
        2. Copy `.env.example` to `.env` and configure local overrides
        3. Run `python setup_launchpad.py` if the scaffold has not already been generated
        4. Follow `directives/001_initial_setup.md`

        ## Documentation

        - [Product Requirements](docs/PRD.md)
        - [Technical Architecture](docs/ARCH.md)
        - [Implementation Research](docs/RESEARCH.md)
        - [Prototype Reference](docs/prototype-reference.md)
        - [Agent Instructions](AGENTS.md)

        ## Development Methodology

        - [Implementation Planning](docs/methodology/implementation-planning.md)
        - [Review Gates](docs/methodology/review-gates.md)
        - [Debugging Guide](docs/methodology/debugging-guide.md)

        ## Project Structure

        ```text
{tree_block}
        ```

        ## Prototype Reference

        The existing `local/` folder contains the working Codex Limits prototype and should remain usable as reference material while Ida is built in `apps/`, `core/`, and `providers/`.
        """,
    )


def create_requirements(stack: Sequence[Dict[str, str]]) -> None:
    lines = [
        "# Python helper requirements",
        "# The project runtime stack is defined in docs/ARCH.md, not by pip.",
        "# Current non-Python stack extracted from docs/ARCH.md Tech Stack:",
    ]
    for row in stack:
        layer = row.get("Layer", "").strip()
        technology = row.get("Technology", "").strip()
        version = row.get("Version", "").strip()
        if layer and technology:
            suffix = f" {version}" if version and version.lower() != "n/a" else ""
            lines.append(f"# - {layer}: {technology}{suffix}")
    lines.extend(
        [
            "",
            "# No Python packages are required for the generated setup and verification scripts.",
        ]
    )
    write_file("requirements.txt", "\n".join(lines))


def create_agents_md(
    project_name: str,
    description: str,
    stack_text: str,
    dictionary_terms: Sequence[Tuple[str, str]],
) -> None:
    term_lines = "\n".join(f"- {term}" for term, _definition in dictionary_terms)
    if not term_lines:
        term_lines = "- See docs/ARCH.md Dictionary"
    term_block = textwrap.indent(term_lines, "        ")

    write_file(
        "AGENTS.md",
        f"""
        # AGENTS.md - System Kernel

        ## Project Context

        **Name:** {project_name}
        **Purpose:** {description}
        **Stack:** {stack_text}

        ## Core Domain Entities

{term_block}

        ---

        ## 1. The Prime Directive

        You are an agent operating on the {project_name} codebase.

        **Before writing ANY code:**
        1. Read `docs/PRD.md` to understand WHAT we are building
        2. Read `docs/ARCH.md` to understand HOW we structure it
        3. Consult `docs/RESEARCH.md` for proven patterns to follow
        4. Check `directives/` for your current assignment

        **Core Rules:**
        - Use ONLY the technologies defined in ARCH.md Tech Stack
        - Use ONLY the terms defined in ARCH.md Dictionary
        - Follow ONLY the API contracts defined in ARCH.md
        - Place code ONLY in the directories specified in ARCH.md
        - Preserve the existing prototype/reference folders unless a directive explicitly says otherwise
        - Treat `local/` as known-good working prototype material; read from it, but do not edit or migrate it without an explicit directive and a branch/checkpoint

        ---

        ## 2. The 3-Layer Workflow

        ### Layer 1: Directives (Orders)
        - Location: `directives/`
        - Purpose: Task assignments with specific acceptance criteria
        - Action: Read the lowest-numbered incomplete directive

        ### Layer 2: Orchestration (Planning)
        - Location: `docs/plans/`
        - Purpose: Granular implementation plans for each directive
        - Action: Before coding, break the directive into tasks following `docs/methodology/implementation-planning.md`

        ### Layer 3: Execution (Automation)
        - Location: `execution/`
        - Purpose: Reusable scripts for repetitive tasks
        - Examples: `verify_setup.py`, `run_checks.py`, `update_fixtures.py`

        ---

        ## 3. The TDD Iron Law

        **NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.**

        ### The Mandatory Cycle

        For every piece of functionality:

        1. **RED:** Write a test that describes the expected behavior. Run it. Confirm it **fails** and fails for the *right reason*.
        2. **GREEN:** Write the **minimum** code to make the test pass. Run all relevant tests. Confirm they **all pass**.
        3. **REFACTOR:** Clean up the code while keeping tests green. Run all tests again. Confirm they still pass.
        4. **COMMIT:** Only after all tests pass.

        ### The Nuclear Rule

        If you write production code before writing its test:
        - **Delete it.** Not "keep as reference." Not "adapt it while writing tests." Delete means delete.
        - Write the test first.
        - Implement fresh, guided by the failing test.

        ### Test File Locations

        Mirror the source structure and local framework conventions:
        - `core/ida-core/src/models/snapshot.rs` -> `core/ida-core/tests/snapshot_tests.rs`
        - `providers/codex/src/parser.rs` -> `providers/codex/tests/parser_tests.rs`
        - `apps/desktop/src/windows/WidgetWindow.tsx` -> `apps/desktop/src/windows/WidgetWindow.test.tsx`
        - Cross-crate behavior -> `tests/`

        ### TDD Rationalizations Table

        If you catch yourself thinking any of these, **STOP**:

        | Excuse | Reality |
        |--------|---------|
        | "This is too simple to test" | Simple code breaks. The test takes 30 seconds to write. |
        | "I'll write tests after" | Tests that pass immediately prove nothing. |
        | "I already tested it manually" | Manual testing has no record and cannot be re-run. |
        | "Deleting my work is wasteful" | Sunk cost fallacy. Keeping unverified code is technical debt with interest. |
        | "I'll keep it as reference and write tests first" | You'll adapt it. That's tests-after with extra steps. Delete means delete. |
        | "I need to explore first" | Explore freely. Then throw away the exploration and start with TDD. |
        | "The test is hard to write" | Listen to the test. Hard to test means hard to use. Redesign. |
        | "TDD will slow me down" | TDD is faster than debugging. |
        | "TDD is dogmatic; I'm being pragmatic" | TDD is pragmatic. Shortcuts become debugging in production. |
        | "This is different because..." | It's not. Delete the code. Start with the test. |

        ### Red Flags - Stop and Start Over

        - You wrote code before its test
        - A new test passes immediately
        - You cannot explain why a test failed
        - You are rationalizing "just this once"

        ---

        ## 4. Implementation Planning

        **Before coding any directive, create an implementation plan.**

        See `docs/methodology/implementation-planning.md` for the full template.

        **The rule:** Write every plan as if the implementer is an enthusiastic junior engineer with no project context and an aversion to testing. This forces you to be completely explicit:

        - **Exact file paths** - not "the config file" but `core/ida-core/src/config.rs`
        - **Complete code** - not "add validation" but the actual validation code
        - **Exact commands** - not "run the tests" but `cargo test -p ida-core snapshot_validation`
        - **Expected output** - what success/failure looks like

        **Granularity:** Each task should take 2-5 minutes. Each step within a task is exactly ONE action.

        Plans are saved to `docs/plans/YYYY-MM-DD-<feature-name>.md`.

        ---

        ## 5. Review Gates

        **Every completed task goes through two review stages before moving on.**

        See `docs/methodology/review-gates.md` for checklists.

        ### Gate 1: Spec Compliance Review

        After completing a task, review against the directive's acceptance criteria:
        - Does the code implement exactly what was specified?
        - Is anything **missing** from the spec?
        - Is anything **extra** that was not requested? Remove it.
        - **Do not trust self-reports.** Read the actual code. Run the actual tests.

        ### Gate 2: Code Quality Review

        Only after spec compliance passes:
        - Architecture: Does it follow ARCH.md patterns?
        - Testing: Are tests meaningful and behavior-focused?
        - DRY: Is there duplication that should be consolidated?
        - Error handling: Are failure modes covered with ARCH.md error codes?
        - Security: Are secrets kept out of snapshots, logs, fixtures, and UI dumps?

        Issues are categorized:
        - **Critical** - Must fix before proceeding. Blocks progress.
        - **Important** - Should fix. Creates tech debt if skipped.
        - **Minor** - Nice to have. Fix if time allows.

        ### Batch Checkpoints

        After every 3 completed tasks, pause and produce a progress report:
        - What's been completed
        - What's next
        - Any concerns or architectural questions
        - Request human feedback before continuing

        ---

        ## 6. Verification Before Completion

        **NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE.**

        Before marking any task, directive, or feature as "done":
        1. **Run the verification command**: tests, linter, type checker, or visual check named by the plan
        2. **Read the actual output**: not from memory, not assumed
        3. **Include the evidence** in your completion report

        ### Verification Red Flags - Stop Immediately

        If you catch yourself using these words before running verification:
        - "Should work now"
        - "That should fix it"
        - "Seems correct"
        - "I'm confident this works"
        - "Great! Done!"

        These are emotional signals, not evidence. **Run the command. Read the output. Then speak.**

        ### Verification Rationalizations Table

        | Excuse | Reality |
        |--------|---------|
        | "It should work now" | Run the verification. |
        | "I'm confident in this change" | Confidence is not evidence. |
        | "The linter passed" | Linter passing does not prove tests pass or behavior is correct. |
        | "I checked it mentally" | Mental checks miss edge cases. Run the actual command. |
        | "Just this once we can skip verification" | No exceptions. |
        | "Partial verification is enough" | Partial evidence proves nothing about what you did not check. |

        ---

        ## 7. Systematic Debugging

        **NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.**

        When something breaks, follow the 4-phase process. See `docs/methodology/debugging-guide.md` for details.

        ### Phase 1: Root Cause Investigation

        - Read the error carefully. Reproduce it consistently.
        - Check what recently changed.
        - Trace the data flow backward from the symptom to the source.

        ### Phase 2: Pattern Analysis

        - Find working examples of similar code in the codebase.
        - Compare the broken code against working references.
        - Identify all differences.

        ### Phase 3: Hypothesis and Testing

        - Form ONE hypothesis: "I think X happens because Y."
        - Test with the smallest possible change.
        - If wrong, form a NEW hypothesis. Do not stack fixes.

        ### Phase 4: Implementation

        - Write a failing test that reproduces the bug.
        - Fix with a single, targeted change.
        - Verify all tests pass: existing plus new.

        ### The 3-Strikes Rule

        If 3 consecutive fix attempts fail: **STOP.**
        - Question whether the approach or architecture is fundamentally sound.
        - Discuss with the team before attempting more fixes.
        - Consider whether you are fixing a symptom instead of the cause.

        ---

        ## 8. Anti-Rationalization Rules

        AI agents will try to bypass the processes above. This section preemptively blocks the most common escape routes.

        **The principle: The ritual IS the spirit.** Violating the letter of these rules is violating the spirit. There are no clever workarounds.

        ### Universal Red Flags

        If any of these thoughts arise, treat them as a signal to **slow down**, not speed up:

        - "I need more context before I can start" - You have PRD, ARCH, RESEARCH, and the directive. Start with the test.
        - "Let me explore the codebase first" - Read the plan. If there is no plan, write one.
        - "I'll clean this up later" - Clean it up now or do not touch it.
        - "This does not apply to this situation" - It does. Follow the process.
        - "I already know the answer" - Prove it. Write the test. Run the verification.
        - "I'll be more careful next time" - Be careful this time. Follow the process this time.

        ---

        ## 9. Definition of Done

        A task is complete when:
        - [ ] Implementation plan was written before coding
        - [ ] Code exists in the appropriate ARCH.md directory
        - [ ] All new production code has corresponding tests
        - [ ] Tests were written BEFORE implementation
        - [ ] Rust tests pass where Rust changed
        - [ ] Frontend tests pass where TypeScript/React changed
        - [ ] Type checking passes: `cargo check` and/or `pnpm typecheck`
        - [ ] Linting passes: `cargo clippy` and/or `pnpm lint`
        - [ ] Formatting passes: `cargo fmt --check` and/or `pnpm format:check`
        - [ ] Spec compliance review passed
        - [ ] Code quality review passed with no Critical or Important issues
        - [ ] Related PRD User Story acceptance criteria are met
        - [ ] UI work has browser or desktop visual verification when required by the directive
        - [ ] Directive file is marked as Complete

        ---

        ## 10. File Naming Conventions

        | Type | Convention | Example |
        |------|------------|---------|
        | Rust modules | snake_case | `provider_snapshot.rs` |
        | Rust types | PascalCase | `ProviderSnapshot` |
        | TypeScript components | PascalCase | `WidgetWindow.tsx` |
        | TypeScript utilities | camelCase or kebab file names matching local convention | `statusColor.ts` |
        | Test files | Framework convention | `parser_tests.rs`, `WidgetWindow.test.tsx` |
        | Directives | `NNN_description.md` | `001_initial_setup.md` |
        | Implementation plans | `YYYY-MM-DD-feature.md` | `2026-05-03-codex-provider.md` |
        | Tauri commands | snake_case verbs | `refresh_now`, `get_app_state` |

        ---

        ## 11. Commit Message Format

        ```text
        type(scope): description

        [optional body]

        Refs: directive-NNN
        ```

        Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

        Example:

        ```text
        feat(provider): add normalized Codex snapshot parser

        Implements parser fixtures for successful and partial Codex output.
        Refs: directive-002
        ```
        """,
    )


def create_methodology_docs() -> None:
    write_file(
        "docs/methodology/implementation-planning.md",
        """
        # Implementation Planning Guide

        ## Purpose

        Before coding any directive, break it into a detailed implementation plan. This prevents context drift and ensures each step is small enough to verify independently.

        ## Plan Template

        Save plans to `docs/plans/YYYY-MM-DD-<feature-name>.md`.

        ```markdown
        # [Feature Name] Implementation Plan

        **Directive:** [NNN]
        **Date:** [YYYY-MM-DD]
        **Goal:** [One sentence - what this achieves]
        **Architecture Notes:** [2-3 sentences - key ARCH.md patterns that apply]

        ---

        ### Task 1: [Component Name]

        **Files:**
        - Create: `core/ida-core/src/models/provider_snapshot.rs`
        - Create: `core/ida-core/tests/provider_snapshot_tests.rs`

        **Step 1:** Write failing test
        - File: `core/ida-core/tests/provider_snapshot_tests.rs`
        - Code: [complete test code]
        - Run: `cargo test -p ida-core provider_snapshot -- --nocapture`
        - Expected: 1 failed test for the intended assertion, not an import or compile error

        **Step 2:** Implement minimum code
        - File: `core/ida-core/src/models/provider_snapshot.rs`
        - Code: [complete implementation code]
        - Run: `cargo test -p ida-core provider_snapshot -- --nocapture`
        - Expected: targeted test passes

        **Step 3:** Refactor (if needed)
        - [Describe what to clean up]
        - Run: `cargo test --workspace`
        - Expected: all Rust tests pass

        **Step 4:** Commit
        - `git add core/ida-core/src/models/provider_snapshot.rs core/ida-core/tests/provider_snapshot_tests.rs`
        - `git commit -m "feat(core): add provider snapshot model"`
        ```

        ## Task Decomposition Rules

        1. **2-5 minutes per task.** If a task takes longer, break it down further.
        2. **One action per step.** "Write the test" is one step. "Run the test" is a separate step.
        3. **Exact file paths.** Never say "the config file" - say `core/ida-core/src/config.rs`.
        4. **Complete code.** Never say "add validation" - write the actual validation code.
        5. **Exact commands with expected output.** Never say "run tests" - say `cargo test -p ida-core parser_handles_missing_weekly_limit` and describe what success looks like.
        6. **Write for someone with no context.** Assume the implementer cannot infer anything. Be painfully explicit.

        ## Plan Execution

        Execute tasks sequentially. After each task:
        1. Run the spec compliance review.
        2. Run the code quality review.
        3. Move to the next task only when both reviews pass.

        After every 3 tasks, produce a checkpoint report.
        """,
    )

    write_file(
        "docs/methodology/review-gates.md",
        """
        # Review Gates Guide

        ## Purpose

        Every completed task passes through two review stages before moving on. This catches issues early, before they compound across multiple tasks.

        ## Gate 1: Spec Compliance Review

        **Goal:** Does the code do what the directive asked?

        ### Checklist

        - [ ] Read the directive's acceptance criteria line by line
        - [ ] For each criterion, read the actual code that implements it
        - [ ] For each criterion, run the verification command and confirm it passes
        - [ ] Check for **missing** requirements - things the directive asked for that were not implemented
        - [ ] Check for **extra** additions - things that were implemented but were not asked for
        - [ ] Check for **misinterpretations** - things that were implemented but do not match the spec's intent

        ### Adversarial Posture

        Assume the self-report is optimistic. Do NOT trust claims like:
        - "All tests pass" - run them yourself
        - "Implemented as specified" - read the code and compare to the spec
        - "No issues found" - look for issues anyway

        ### Outcome

        - **Pass:** Proceed to Gate 2
        - **Issues Found:** Fix issues, then re-review from the beginning

        ## Gate 2: Code Quality Review

        **Goal:** Is the code well-built?

        Only run this AFTER Gate 1 passes.

        ### Checklist

        - [ ] **Architecture:** Does it follow ARCH.md patterns and directory structure?
        - [ ] **Domain Language:** Are ARCH.md Dictionary terms used correctly and consistently?
        - [ ] **Testing:** Are tests testing real behavior, not just mock existence?
        - [ ] **Error Handling:** Are failure modes covered with appropriate ARCH.md error codes?
        - [ ] **DRY:** Is there duplication that should be extracted?
        - [ ] **Security:** Does it follow ARCH.md Security Considerations?
        - [ ] **UI:** For widget/settings work, has the layout been visually verified?

        ### Issue Categorization

        | Category | Definition | Action |
        |----------|-----------|--------|
        | **Critical** | Breaks functionality, security vulnerability, or violates ARCH.md contract | Must fix before proceeding |
        | **Important** | Tech debt, poor patterns, insufficient tests | Should fix; creates compound problems if skipped |
        | **Minor** | Style, naming, documentation | Fix if time allows |

        ### Outcome

        - **Pass:** Task is complete. Move to next task.
        - **Critical/Important issues:** Fix, then re-review from Gate 1

        ## Batch Checkpoints

        After every 3 completed tasks, pause and report:

        ```markdown
        ## Checkpoint Report

        ### Completed
        - Task 1: [description] - Done
        - Task 2: [description] - Done
        - Task 3: [description] - Done

        ### Up Next
        - Task 4: [description]
        - Task 5: [description]

        ### Concerns
        - [Any architectural questions, blockers, or scope issues]

        ### Request
        [Ask for human feedback before continuing]
        ```
        """,
    )

    write_file(
        "docs/methodology/debugging-guide.md",
        """
        # Systematic Debugging Guide

        ## Purpose

        When something breaks, resist the urge to guess-and-fix. Follow the 4-phase process to find and fix the actual root cause, not just the symptom.

        ## Phase 1: Root Cause Investigation

        Before changing any code:

        1. **Read the error carefully.** Read the full error message, stack trace, and logs.
        2. **Reproduce consistently.** If you cannot reproduce it on demand, you do not understand it yet.
        3. **Check recent changes.** What was the last change before this broke? Start there.
        4. **Trace backward.** Start at the symptom. Ask: "What called this with the bad value?" Trace up the call stack until you find where the bad data originated.
        5. **Log at boundaries.** In multi-component systems, add logging at component boundaries to isolate which layer introduced the problem.

        ## Phase 2: Pattern Analysis

        1. **Find working examples.** Is there similar code in the codebase that works? Read it completely.
        2. **Compare differences.** What is different between the working code and the broken code?
        3. **Check documentation.** Does the library or framework documentation say something you missed?

        ## Phase 3: Hypothesis and Testing

        1. **Form ONE hypothesis.** "I think [symptom] happens because [cause]."
        2. **Test with the smallest possible change.** One variable at a time.
        3. **If wrong:** Form a NEW hypothesis. Do NOT stack multiple changes.
        4. **If right:** Proceed to Phase 4.

        Do not guess. Do not try random fixes. Do not change multiple things at once.

        ## Phase 4: Implementation

        1. **Write a failing test** that reproduces the bug.
        2. **Fix with a single, targeted change.**
        3. **Run ALL tests** relevant to the change to confirm no regressions.
        4. **Add defense-in-depth validation** to prevent this class of bug from recurring:
           - Entry point validation
           - Business logic assertions
           - Clear error messages that point to the cause

        ## The 3-Strikes Rule

        If 3 consecutive fix attempts fail: **STOP.**

        Before attempting a 4th fix, answer these questions:
        - "Is this architecture fundamentally sound, or am I fighting the design?"
        - "Am I fixing the root cause or a downstream symptom?"
        - "Should I discuss this with the team before continuing?"

        If you cannot confidently answer all three, escalate to a human.

        ## Common Debugging Anti-Patterns

        | Anti-Pattern | What To Do Instead |
        |-------------|-------------------|
        | Guessing and trying random fixes | Form a hypothesis, test one variable at a time |
        | Changing multiple things at once | Revert all, change one thing, verify |
        | Fixing the symptom not the cause | Trace backward to the source of bad data |
        | "It works on my machine" | Check environment differences systematically |
        | Adding try/catch to suppress errors | Fix the cause; errors exist for a reason |
        | Reading the error too quickly | Read it word by word, including the full stack trace |
        """,
    )


def create_prototype_reference_doc() -> None:
    write_file(
        "docs/prototype-reference.md",
        """
        # Prototype Reference

        ## Purpose

        The `local/` folder contains the existing working Codex Limits prototype. It is a known-good reference implementation for capture, dashboard display, and local data files while the new Ida product is built separately.

        ## Preservation Rules

        - Keep `local/` usable unless a future directive explicitly says to migrate or remove it.
        - Do not place new Ida desktop app, core, or provider code in `local/`.
        - Port useful behavior into `apps/`, `core/`, or `providers/` with tests instead of mutating the prototype in place.
        - Before any directive touches `local/`, create or switch to a dedicated branch and record the reason in the directive notes.
        - If local prototype data must be captured before an experiment, place temporary backups under `.tmp/` and do not commit secrets or machine-specific state.

        ## Current Reference Files

        - `local/monitor.sh` - existing Codex usage capture script
        - `local/dashboard.html` - existing prototype dashboard
        - `local/data.json` - latest prototype output
        - `local/history.json` - prototype history output
        - `local/images/` - prototype visual assets

        ## Migration Policy

        The new product structure is:

        - `apps/desktop/` for the Tauri desktop app
        - `core/ida-core/` for provider-neutral Rust logic
        - `providers/codex/` for Codex-specific capture and parsing
        - `providers/claude/` for a future placeholder only

        Any extraction from `local/` must be covered by tests in the new structure before the prototype behavior is considered replaced.
        """,
    )


def create_initial_directive(project_name: str) -> None:
    write_file(
        "directives/001_initial_setup.md",
        f"""
        # Directive 001: Initial Environment Setup

        ## Objective

        Configure the development environment for {project_name} and verify the scaffold, documentation, and local toolchain are ready.

        ## Prerequisites

        - Python 3.11+ installed
        - Rust stable toolchain compatible with docs/ARCH.md
        - Node.js and pnpm compatible with docs/ARCH.md
        - Windows desktop prerequisites for Tauri development when building the desktop app
        - Codex CLI authenticated locally when provider capture work begins
        - WSL available on Windows if native provider capture is unavailable
        - Git initialized

        ## Steps

        ### Step 1: Virtual Environment

        ```bash
        python -m venv .venv
        source .venv/bin/activate  # Linux/Mac
        # or: .venv\\Scripts\\activate  # Windows
        ```

        ### Step 2: Install Python Helper Dependencies

        ```bash
        pip install -r requirements.txt
        ```

        ### Step 3: Verify Rust and Node Toolchains

        ```bash
        rustc --version
        cargo --version
        node --version
        pnpm --version
        ```

        ### Step 4: Configure Environment

        ```bash
        cp .env.example .env
        # Edit .env with local overrides if needed
        ```

        ### Step 5: Verify Scaffold

        ```bash
        python execution/verify_setup.py
        ```

        ### Step 6: Run Initial Checks

        ```bash
        # Once Rust workspace crates exist:
        cargo test --workspace

        # Once the desktop package exists:
        pnpm install
        pnpm test
        ```

        ## Acceptance Criteria

        - [ ] Virtual environment created and activated
        - [ ] Python helper dependencies installed without errors
        - [ ] Rust toolchain is available
        - [ ] Node.js and pnpm are available
        - [ ] `.env` file exists with valid local configuration
        - [ ] `verify_setup.py` passes all scaffold checks
        - [ ] Initial test commands are documented and ready for the first implementation directive

        ## Development Methodology

        Starting from Directive 002 onward, all work follows the processes defined in AGENTS.md:
        - **Implementation Planning** before coding (Section 4)
        - **TDD Iron Law** during coding (Section 3)
        - **Review Gates** after each task (Section 5)
        - **Verification Before Completion** before marking done (Section 6)

        See `docs/methodology/` for detailed reference guides.

        ## Status: [ ] Incomplete / [ ] Complete

        ## Notes

        [Agent: Add any issues encountered or decisions made]
        """,
    )


def create_verify_script(directories: Sequence[str]) -> None:
    required_dirs_repr = repr(list(directories))
    write_file(
        "execution/verify_setup.py",
        f'''
        #!/usr/bin/env python3
        """
        Verify that the development environment is correctly configured.
        Run this after initial setup to confirm everything works.
        """

        import re
        import shutil
        import subprocess
        import sys
        from pathlib import Path


        REQUIRED_DIRS = {required_dirs_repr}


        def run_version(command):
            executable = shutil.which(command[0])
            if executable is None:
                return False, f"{{command[0]}} not found on PATH"
            try:
                completed = subprocess.run(
                    command,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
            except Exception as error:
                return False, f"{{command[0]}} failed to run: {{error}}"

            output = (completed.stdout or completed.stderr).strip()
            if completed.returncode != 0:
                return False, output or f"{{command[0]}} returned {{completed.returncode}}"
            return True, output


        def parse_version(text):
            match = re.search(r"(\\d+)\\.(\\d+)(?:\\.(\\d+))?", text)
            if not match:
                return None
            return tuple(int(part or 0) for part in match.groups())


        def check_python_version():
            required = (3, 11)
            current = sys.version_info[:2]
            if current < required:
                return False, f"Python {{required[0]}}.{{required[1]}}+ required, found {{current[0]}}.{{current[1]}}"
            return True, f"Python {{current[0]}}.{{current[1]}}"


        def check_rust_toolchain():
            passed, message = run_version(["rustc", "--version"])
            if not passed:
                return False, message
            version = parse_version(message)
            if version and version[:2] < (1, 82):
                return False, f"Rust 1.82+ required, found {{message}}"
            cargo_passed, cargo_message = run_version(["cargo", "--version"])
            if not cargo_passed:
                return False, cargo_message
            return True, f"{{message}}; {{cargo_message}}"


        def check_node_toolchain():
            node_passed, node_message = run_version(["node", "--version"])
            if not node_passed:
                return False, node_message
            version = parse_version(node_message)
            if version and not (version[0] >= 24 or (version[0] == 22 and version[1] >= 12)):
                return False, f"Node.js 24 LTS preferred or 22.12+ minimum, found {{node_message}}"

            pnpm_passed, pnpm_message = run_version(["pnpm", "--version"])
            if not pnpm_passed:
                return False, pnpm_message
            pnpm_version = parse_version(pnpm_message)
            if pnpm_version and pnpm_version[0] < 10:
                return False, f"pnpm 10+ required, found {{pnpm_message}}"
            return True, f"node {{node_message}}; pnpm {{pnpm_message}}"


        def check_env_file():
            env_path = Path(".env")
            if not env_path.exists():
                return False, ".env file not found (copy from .env.example)"
            return True, ".env file exists"


        def check_required_dirs():
            missing = [directory for directory in REQUIRED_DIRS if not Path(directory).is_dir()]
            if missing:
                return False, f"Missing directories: {{', '.join(missing)}}"
            return True, "All required directories exist"


        def check_docs():
            docs = ["docs/PRD.md", "docs/ARCH.md", "docs/RESEARCH.md"]
            missing = [doc for doc in docs if not Path(doc).exists()]
            if missing:
                return False, f"Missing documents: {{', '.join(missing)}}"
            return True, "PRD.md, ARCH.md, and RESEARCH.md exist"


        def check_methodology():
            docs = [
                "docs/methodology/implementation-planning.md",
                "docs/methodology/review-gates.md",
                "docs/methodology/debugging-guide.md",
            ]
            missing = [doc for doc in docs if not Path(doc).exists()]
            if missing:
                return False, f"Missing methodology docs: {{', '.join(missing)}}"
            return True, "All methodology documents exist"


        def check_prototype_reference():
            files = [
                "docs/prototype-reference.md",
                "local/monitor.sh",
                "local/dashboard.html",
                "local/data.json",
                "local/history.json",
            ]
            missing = [file for file in files if not Path(file).exists()]
            if missing:
                return False, f"Missing prototype reference files: {{', '.join(missing)}}"
            return True, "Prototype reference files are present"


        def check_bootloader_files():
            files = [
                "AGENTS.md",
                "directives/001_initial_setup.md",
                ".gitignore",
                ".env.example",
                ".cursorrules",
                "requirements.txt",
            ]
            missing = [file for file in files if not Path(file).exists()]
            if missing:
                return False, f"Missing scaffold files: {{', '.join(missing)}}"
            return True, "All bootloader files exist"


        def main():
            checks = [
                ("Python Version", check_python_version),
                ("Rust Toolchain", check_rust_toolchain),
                ("Node Toolchain", check_node_toolchain),
                ("Environment File", check_env_file),
                ("Directory Structure", check_required_dirs),
                ("Documentation", check_docs),
                ("Methodology", check_methodology),
                ("Prototype Reference", check_prototype_reference),
                ("Bootloader Files", check_bootloader_files),
            ]

            print("=" * 50)
            print("Environment Verification")
            print("=" * 50)

            all_passed = True
            for name, check_func in checks:
                passed, message = check_func()
                status = "OK" if passed else "FAIL"
                print(f"[{{status}}] {{name}}: {{message}}")
                if not passed:
                    all_passed = False

            print("=" * 50)
            if all_passed:
                print("All checks passed. Environment is ready.")
                return 0

            print("Some checks failed. Please fix the issues above.")
            return 1


        if __name__ == "__main__":
            sys.exit(main())
        ''',
        executable=True,
    )


def create_ide_config(project_name: str) -> None:
    write_file(
        ".cursorrules",
        f"""
        # Cursor AI Rules for {project_name}

        ## Session Start Protocol
        ALWAYS read these files at the start of EVERY session:
        1. AGENTS.md (this project's conventions, workflow, and enforcement rules)
        2. docs/ARCH.md (technical architecture and constraints)
        3. docs/RESEARCH.md (proven patterns and anti-patterns)
        4. directives/ (find your current task)

        ## Code Generation Rules
        - Use ONLY technologies listed in docs/ARCH.md Tech Stack
        - Follow directory structure defined in docs/ARCH.md
        - Use domain terms EXACTLY as defined in ARCH.md Dictionary
        - Write tests BEFORE implementation (TDD Iron Law)
        - Create implementation plans BEFORE coding (AGENTS.md Section 4)
        - Pass both review gates BEFORE marking tasks done (AGENTS.md Section 5)
        - Keep provider-specific logic behind provider boundaries
        - Keep secrets out of snapshots, history, fixtures, logs, screenshots, and generated bindings

        ## Forbidden Actions
        - Do NOT install packages not listed in docs/ARCH.md or generated project manifests without approval
        - Do NOT create files outside the defined directory structure
        - Do NOT deviate from API contracts in ARCH.md
        - Do NOT use .tmp/ for anything except temporary planning notes
        - Do NOT write production code before its failing test
        - Do NOT claim completion without running verification commands
        - Do NOT place new product app code in prototype/reference folders
        """,
    )


def main() -> int:
    os.chdir(ROOT)
    docs = read_required_docs()

    project_name = extract_project_name(docs["PRD"])
    description = extract_project_description(docs["PRD"])
    directories = extract_directories(docs["ARCH"])
    stack = extract_tech_stack(docs["ARCH"])
    dictionary_terms = extract_dictionary_terms(docs["ARCH"])
    env_vars = extract_env_vars(docs["ARCH"])
    tree_text = directory_tree(directories)

    print(f"Initializing {project_name}...")
    create_directories(directories)
    create_gitignore()
    create_env_example(project_name, env_vars)
    create_readme(project_name, description, tree_text)
    create_requirements(stack)
    create_agents_md(project_name, description, stack_summary(stack), dictionary_terms)
    create_methodology_docs()
    create_prototype_reference_doc()
    create_initial_directive(project_name)
    create_verify_script(directories)
    create_ide_config(project_name)
    print("Launchpad complete! Run: python execution/verify_setup.py")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"Launchpad failed: {error}", file=sys.stderr)
        raise SystemExit(1)
