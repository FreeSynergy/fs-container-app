# CLAUDE.md – fs-container-app

## What is this?

FreeSynergy Container App — manages systemd services, containers, and bots.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
1. Design Pattern (Traits, Object hierarchy)
2. Structs + Traits — no impl code yet
3. cargo check
4. Impl (OOP)
5. cargo clippy --all-targets -- -D warnings
6. cargo fmt --check
7. Unit tests (min. 1 per public module)
8. cargo test
9. commit + push
```

Every lib.rs / main.rs must have:
```rust
#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings)]
```

## Architecture

- `Container` — root Dioxus component
- `ServiceList` / `ServiceDetail` — systemd unit management
- `BuildView` — container image build
- `UnitActiveStateDisplay` trait — extension trait for status display

## CSS Variables Prefix

Always `--fs-` (e.g., `--fs-color-primary`).

## Dependencies

- **fs-libs** (`../fs-libs/`) — `fs-components`, `fs-container`, `fs-error`, `fs-i18n`
- **fs-desktop** (`../fs-desktop/vendor/dioxus-desktop`) — patched Dioxus desktop
