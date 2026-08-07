# Statusline Optimization and Refactoring Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor `agy-statusline` to achieve **10x execution performance** (sub-50μs per parse+render), **1/10 startup time**, and **1/2 binary size** (under 140 KiB) while preserving 100% functionality and test compatibility.

**Architecture:** 
1. Zero-copy JSON parsing with lifetimes (`ParsedInput<'a>`) replacing heap allocations (`String`).
2. Single-buffer rendering engine removing intermediate `Vec<String>` allocations and `format!` macro overhead.
3. High-performance system metric retrieval (zero-process-spawn Git branch inspection via `.git/HEAD` reader).
4. Binary footprint reduction by eliminating `std::fmt` bloat.

**Tech Stack:** Rust (std only, 0 external crate dependencies)

## Global Constraints

- No external crate dependencies (keep `[dependencies]` empty in `Cargo.toml`).
- Maintain 100% backwards compatibility with all existing CLI options and JSON formats.
- All 52 existing tests + performance benchmark tests must pass cleanly.
- Absolute UTF-8 / ANSI color safety and identical output layout.

---

### Task 1: Zero-Copy Zero-Allocation JSON Parser (`parse.rs`)

**Files:**
- Modify: `src/parse.rs`
- Modify: `src/lib.rs`
- Test: `tests/parser_tests.rs`, `tests/perf_benchmark.rs`

**Interfaces:**
- Consumes: JSON string slice `&'a str`
- Produces: `ParsedInput<'a>` with zero heap allocations for string fields.

- [ ] **Step 1: Write/Update zero-copy ParsedInput struct and unit tests**
Define `ParsedInput<'a>` holding `&'a str` instead of `String` for all string fields (`agent_state`, `model_id`, `model_display_name`, `working_dir`, `version`, `plan_tier`, `email`, `conversation_id`, `product`, `vcs_branch`).

- [ ] **Step 2: Implement zero-copy parser with zero heap allocations**
Refactor `JsonParser<'a>` to return `&'a str` directly without UTF-8 conversions or allocations. Optimally handle string escaping if any (borrow when slice unescaped).

- [ ] **Step 3: Run parser tests to verify correctness**
Run `cargo test --test parser_tests` and ensure all 18 parser unit tests pass.

- [ ] **Step 4: Commit**
`git add src/parse.rs tests/parser_tests.rs`
`git commit -m "refactor(parse): zero-copy JSON parser with zero heap allocations"`


### Task 2: Single-Buffer Streamline Rendering Engine (`bar.rs`, `format.rs`, `render.rs`, `icons.rs`)

**Files:**
- Modify: `src/icons.rs`
- Modify: `src/format.rs`
- Modify: `src/bar.rs`
- Modify: `src/render.rs`
- Test: `tests/bar_tests.rs`, `tests/roundtrip_tests.rs`, `tests/perf_benchmark.rs`

**Interfaces:**
- Consumes: `&ParsedInput<'a>`, `classic: bool`, `override_cols: Option<usize>`
- Produces: `String` rendered output using direct buffer write with pre-allocated capacity.

- [ ] **Step 1: Implement direct buffer write formatting helpers in `format.rs` and `bar.rs`**
Replace `format!` usage with lightweight buffer push operations (`push_str`, custom integer-to-buffer formatter). Implement fast ASCII-based `visible_len`.

- [ ] **Step 2: Refactor `render_line` to use a single output buffer**
Eliminate `Vec<String>` (`active_segs`, `badge_list`, `packed_lines`). Stream segment rendering and line-packing into a single `String` buffer reserved with `with_capacity(1024)`.

- [ ] **Step 3: Run roundtrip and bar tests to verify identical output**
Run `cargo test --test roundtrip_tests` and `cargo test --test bar_tests`.

- [ ] **Step 4: Commit**
`git add src/icons.rs src/format.rs src/bar.rs src/render.rs`
`git commit -m "refactor(render): single-buffer rendering engine removing intermediate allocations"`


### Task 3: Zero-Process Fast Git & Host Telemetry (`sys.rs`)

**Files:**
- Modify: `src/sys.rs`
- Test: `tests/sys_tests.rs`

**Interfaces:**
- Consumes: `working_dir: &str`, `parsed_branch: &str`, `parsed_dirty: bool`
- Produces: `(Cow<'a, str>, bool)`, `SysInfo`, `Option<String>`, `Option<PowerInfo>` without slow `git` process spawn.

- [ ] **Step 1: Implement direct `.git/HEAD` reader in `git_info`**
Read `.git/HEAD` directly to extract branch name when `parsed_branch` is empty, avoiding expensive `Command::new("git")` subprocess invocation.

- [ ] **Step 2: Optimize host & sys telemetry**
Optimize `/proc/meminfo` and `/proc/loadavg` parsing on Linux with zero allocations.

- [ ] **Step 3: Verify system tests**
Run `cargo test --test sys_tests`.

- [ ] **Step 4: Commit**
`git add src/sys.rs`
`git commit -m "perf(sys): zero-process git branch inspection and optimized sys telemetry"`


### Task 4: Binary Size Reduction & Final Benchmark Verification

**Files:**
- Modify: `src/main.rs`
- Modify: `Cargo.toml`
- Test: All test suites + `perf_benchmark.rs`

**Interfaces:**
- Consumes: Command line arguments and STDIN
- Produces: High-speed output with reduced binary size and instant startup.

- [ ] **Step 1: Optimize `main.rs` and eliminate std::fmt bloat**
Streamline argument processing and STDIN reading in `main.rs`. Use `write!` / direct stdout write.

- [ ] **Step 2: Verify binary size & performance benchmarks**
Build release binary (`cargo build --release`), verify size <= 143 KB (1/2 size target) and benchmark execution time (< 71 μs for 10x performance target).

- [ ] **Step 3: Run full test suite**
Run `cargo test --release` to ensure all 52+ tests pass cleanly.

- [ ] **Step 4: Commit**
`git add Cargo.toml src/main.rs`
`git commit -m "perf(build): binary size optimization and performance validation"`
