# Module Split Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor agy-statusline into focused modules with shared bar logic, unused field removal, and zero dependencies.

**Architecture:** Extract icons/format/bar/render from main.rs; thin main entry; lib re-exports for tests.

**Tech Stack:** Rust 2021, no external crates, existing integration tests.

---

### Task 1: Bar unit tests + bar module

**Files:**
- Create: `src/bar.rs`
- Create: `tests/bar_tests.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write bar tests**
- [ ] **Step 2: Implement `build_bar` / `build_quota_bar`**
- [ ] **Step 3: `cargo test --test bar_tests` green**

### Task 2: format + icons modules

**Files:**
- Create: `src/format.rs`, `src/icons.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Move human_format / format_reset_time to format.rs**
- [ ] **Step 2: Move ANSI + Icons to icons.rs**
- [ ] **Step 3: Existing tests still pass**

### Task 3: parse cleanup

**Files:**
- Modify: `src/parse.rs`, `tests/parser_tests.rs`

- [ ] **Step 1: Update tests to drop unused field asserts**
- [ ] **Step 2: Remove fields from ParsedInput and parse_field**
- [ ] **Step 3: `cargo test --test parser_tests` green**

### Task 4: sys API + render + thin main

**Files:**
- Modify: `src/sys.rs`, `tests/sys_tests.rs`
- Create: `src/render.rs`
- Modify: `src/main.rs`, `src/lib.rs`

- [ ] **Step 1: git_info → (branch, dirty)**
- [ ] **Step 2: render_line in render.rs**
- [ ] **Step 3: main only wires CLI**
- [ ] **Step 4: `cargo test` all green**

### Task 5: Verify

- [ ] `cargo test`
- [ ] `cargo build --release`
