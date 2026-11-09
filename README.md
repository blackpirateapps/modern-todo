# Daymark — Modern Todo

A modern, clean, and intuitive Todo application built with Rust and Cupertino-inspired design.

## Architecture

Daymark follows a clean, four-layer architecture with strict separation of concerns:

```
daymark/
├── apps/
│   └── android/              # Presentation layer (Slint UI, Cupertino design, Android entry point)
├── crates/
│   ├── domain/               # Core domain entities, priorities, validations & business rules
│   ├── application/          # Use cases, commands, queries, ports & services
│   ├── storage/              # SQLite persistence, migrations & repository implementations
│   └── platform_android/     # Notifications, haptics, permissions & backup
├── resources/                # App assets, icons, fonts, and styles
└── .github/workflows/        # CI workflows (tests and Android APK builds)
```

## Features

- **Layered Architecture:** UI never directly accesses SQLite; business logic is platform-independent.
- **Cupertino-Inspired UI:** Rounded cards, grouped lists, statistics summary badges, custom priority chips, segmented controls, and dark mode support.
- **SQLite Local Storage:** Fast local persistence with numbered migrations, foreign keys, and WAL mode.
- **Rich Task Management:** Due dates, reminders, subtasks (1-level depth), priority levels, search, list organization, and soft/hard deletes.
- **JSON Backup & Restore:** Versioned schema backup and restore.
- **Cross-Platform:** NativeActivity for Android via `cargo-apk` and native desktop executable for rapid testing.

## Building and Testing

### Running Tests
```bash
cargo test --workspace
```

### Running Desktop Preview
```bash
cargo run --bin daymark-android
```

## GitHub Actions Workflows

- **`.github/workflows/test.yml`**: Runs all workspace unit and integration tests on push and pull requests.
- **`.github/workflows/build-apk.yml`**: Configures Android SDK, NDK, and `cargo-apk` to compile the release APK and publish it as an artifact.

## License

MIT
