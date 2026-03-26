# fs-container-app

FreeSynergy Container App — manages containers, services, and bots.

Part of the [FreeSynergy](https://github.com/FreeSynergy) platform.

## Purpose

Provides a Dioxus desktop UI for managing system services (via systemd), containers
(via Podman/Docker), and bots. Uses `fs-container` for the backend logic.

## Architecture

- `Container` — root Dioxus component
- `ServiceList` / `ServiceDetail` — systemd unit management
- `BuildView` — container image build UI
- `InstanceConfig` — per-instance configuration
- `LogViewer` — live log streaming

## Build

```bash
cargo build                   # default: desktop feature
```

## Dependencies

- **fs-libs** (`../fs-libs/`) — `fs-components`, `fs-container`, `fs-error`, `fs-i18n`
- **fs-desktop** (`../fs-desktop/vendor/dioxus-desktop`) — patched Dioxus desktop
