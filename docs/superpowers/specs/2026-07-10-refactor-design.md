# Refactor Design: Module Split & Cleanup

**Date:** 2026-07-10  
**Status:** Approved

## Goals

- Split `main.rs` rendering into focused modules
- Deduplicate context/quota bar drawing
- Remove unused parsed fields
- Keep zero crate dependencies
- Small display improvements allowed; no new features

## Module Layout

```
src/
  main.rs      # CLI entry only
  lib.rs       # re-exports
  parse.rs     # JSON → ParsedInput (used fields only)
  sys.rs       # git_info → (branch, dirty)
  icons.rs     # ANSI + Icons + select_icons
  format.rs    # human_format, format_reset_time helpers
  bar.rs       # shared 10-seg 1/8-step bars
  render.rs    # build_view + render_line
```

## Data Flow

`stdin JSON → parse_input → render_line(+git_info) → stdout`

## Bar API

```rust
pub fn build_bar(filled_pct: f64, color: &str, classic: bool) -> String
pub fn build_quota_bar(remaining_pct: f64, label: &str, reset_sec: i64, classic: bool, reset_icon: &str) -> String
```

- Context: fill by `used_percentage`; color higher = redder
- Quota: fill by remaining; color lower = redder; N/A when pct < -0.5

## Removed From ParsedInput

`terminal_width`, `conversation_id`, `version`, `plan_tier`, `email`

Unknown JSON keys continue to be skipped.

## Testing

- Update parser/sys/roundtrip tests
- Add bar unit tests
- `cargo test` must pass; no new dependencies
