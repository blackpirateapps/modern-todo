# Daymark — Rust Architecture and Implementation Blueprint

## 1. Architectural philosophy

The application should follow a layered architecture with clear boundaries.

The UI must never access SQLite directly.

The database must never depend on the UI framework.

Business logic must not depend on Android APIs.

The Android integration layer should only implement platform-specific functionality.

This gives us four main layers:

1. Presentation
2. Application
3. Domain
4. Infrastructure

## 2. UI framework comparison

Three frameworks are worth considering.

### Dioxus

**Approach:** Rust components with a React-inspired development model.

Advantages:
- UI components are written in Rust.
- Declarative UI development.
- CSS allows detailed Cupertino-inspired styling.
- Component-based architecture.
- Convenient separation of UI and application state.

Trade-offs:
- Dioxus Mobile uses a WebView-based rendering approach.
- It is not the same as using native Android widgets.
- Native capabilities require platform integration.
- WebView behavior and styling need testing on actual devices.

Best suited for: A custom-designed interface where expressing UI components in Rust is important.

### Slint

**Approach:** Declarative UI markup with Rust application logic.

Advantages:
- Designed for Rust-oriented application development.
- Dedicated UI language.
- Custom-rendered interface.
- Clear separation between UI layout and application logic.
- Suitable for consistent custom styling.

Trade-offs:
- UI layouts are written in `.slint` files rather than ordinary Rust.
- Custom Cupertino components still need to be designed.
- Android packaging and individual platform integrations require validation.

Best suited for: A custom-rendered app where a separate declarative UI language is acceptable.

### Iced

**Approach:** Rust GUI framework with an Elm-inspired architecture.

Advantages:
- Rust-first application development.
- Explicit application state.
- Message-based interaction model.
- Custom interface styling.

Trade-offs:
- You will need to build most of the Cupertino-inspired component system yourself.
- Android-specific capabilities and packaging should be evaluated carefully.

Best suited for: Developers who prefer an explicit, message-driven GUI architecture.

### Comparison

| Criteria | Dioxus | Slint | Iced |
|---|---|---|---|
| UI components in Rust | Yes | No, `.slint` | Yes |
| Custom styling | Excellent | Good | Good |
| Rendering approach | WebView | Custom renderer | Custom renderer |
| Cupertino-inspired design | Excellent | Good | Good |
| UI architecture | Component-based | Declarative | Message-based |
| Android integration | Required | Required | Required |

### Recommendation

My provisional recommendation is **Dioxus** for Daymark.

The reasons are:

1. You want the UI to be written in Rust.
2. Cupertino-inspired design is primarily a visual requirement.
3. CSS gives us considerable control over rounded cards, grouped lists, spacing, typography, sheets, and transitions.
4. Component-based development suits our planned screen structure.

However, the WebView rendering model is an important trade-off.

Before committing, build the same small prototype in Dioxus and Slint:

- Today screen
- Task editor
- Bottom navigation

Compare scrolling, keyboard handling, animations, accessibility, and Android packaging.

If WebView behavior is unsatisfactory, Slint is the alternative I would test first.

Do not make the final choice based solely on desktop screenshots or framework demonstrations.

## 3. Rust architecture

### Presentation layer

Responsibilities:
- Render application screens.
- Display task information.
- Collect user input.
- Handle navigation.
- Display loading and error states.
- Present application state.

It should contain no SQL statements or Android-specific implementation details.

### Application layer

Responsibilities:
- Create tasks.
- Update tasks.
- Complete tasks.
- Delete tasks.
- Change task priority.
- Assign tasks to lists.
- Schedule reminders.
- Search tasks.
- Export and import backups.

This layer coordinates the other layers.

### Domain layer

Responsibilities:
- Task definitions.
- Priority definitions.
- Task validation.
- Completion rules.
- Due-date rules.
- Project/list rules.
- Domain errors.

This layer should be framework-independent.

### Infrastructure layer

Responsibilities:
- SQLite access.
- Database migrations.
- Local notifications.
- Android platform integration.
- Backup file handling.
- Device-specific functionality.

## 4. Proposed Rust workspace

Use a Cargo workspace so the application can grow without becoming one enormous crate.

```text
daymark/
│
├── Cargo.toml
├── Cargo.lock
│
├── apps/
│   └── android/
│       ├── Cargo.toml
│       ├── src/
│       │   └── main.rs
│       └── android/
│           └── AndroidManifest.xml
│
├── crates/
│   ├── domain/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── task.rs
│   │       ├── list.rs
│   │       ├── tag.rs
│   │       ├── priority.rs
│   │       └── error.rs
│   │
│   ├── application/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── commands/
│   │       ├── queries/
│   │       ├── services/
│   │       └── ports/
│   │
│   ├── storage/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── database.rs
│   │       ├── migrations.rs
│   │       ├── task_repository.rs
│   │       ├── list_repository.rs
│   │       └── tag_repository.rs
│   │
│   └── platform_android/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── notifications.rs
│           ├── permissions.rs
│           ├── haptics.rs
│           └── backup.rs
│
└── resources/
    ├── icons/
    ├── fonts/
    └── styles/
```

The UI framework should remain inside the Android app or a dedicated presentation crate.

The domain and application crates should not depend on Dioxus, Slint, or Android.

## 5. Domain model

The main entities are:

### Task

A task contains:

- `id`: UUID
- `title`: String
- `description`: Optional String
- `priority`: Priority
- `completed`: Boolean
- `list_id`: Optional UUID
- `due_date`: Optional calendar date
- `reminder_at`: Optional timestamp
- `parent_task_id`: Optional UUID
- `position`: Integer
- `created_at`: Timestamp
- `updated_at`: Timestamp
- `completed_at`: Optional timestamp
- `deleted_at`: Optional timestamp

### Priority

An enumeration:

- None
- Low
- Medium
- High

### List

A list contains:

- `id`
- `name`
- `description`
- `icon`
- `color`
- `position`
- `created_at`
- `updated_at`

### Tag

A tag contains:

- `id`
- `name`
- `color`

Tasks and tags have a many-to-many relationship.

### Subtask

Subtasks use the task model with a parent task reference. Version 1 should enforce a maximum nesting depth of one.

The domain model should use dedicated types for IDs, dates, priorities, and validated titles rather than passing arbitrary strings and integers everywhere.

## 6. SQLite database design

Use SQLite as the persistent local database.

Recommended tables:

| Table | Responsibility |
|---|---|
| `tasks` | Main task records |
| `lists` | Projects and categories |
| `tags` | User-created tags |
| `task_tags` | Task-to-tag relationships |
| `settings` | Local application preferences |

Important constraints:

- Primary keys must be unique.
- Task titles cannot be empty.
- Task references must use foreign keys.
- Deleting a list must have an explicit task-handling policy.
- Completed tasks should be retained.
- Deleted tasks should support recovery where practical.

### Recommended indexes

- `tasks(due_date)`
- `tasks(list_id, completed)`
- `tasks(completed, deleted_at)`
- `tasks(updated_at)`
- `task_tags(tag_id, task_id)`

Database writes should use transactions.

Database schema changes should use numbered migrations rather than modifying the database structure informally.

For dates, store due dates as calendar dates (`YYYY-MM-DD`) rather than inventing midnight UTC timestamps.

For timestamps such as creation, modification, and reminders, use a consistent UTC representation.

For version 1, reminders should represent a specific scheduled instant. Store the associated timezone for display and diagnostics.

## 7. Application use cases

Organize application functionality into commands and queries.

### Commands

- `CreateTask`
- `UpdateTask`
- `CompleteTask`
- `ReopenTask`
- `DeleteTask`
- `MoveTask`
- `ChangePriority`
- `CreateList`
- `UpdateList`
- `DeleteList`
- `ScheduleReminder`
- `CancelReminder`

### Queries

- `GetTodayTasks`
- `GetUpcomingTasks`
- `GetInboxTasks`
- `GetTasksByList`
- `GetCompletedTasks`
- `SearchTasks`
- `GetTaskStatistics`

Commands change state.

Queries return information without modifying state.

This separation will make the code easier to understand and test.

## 8. Repository interfaces

The application layer should define repository interfaces.

For example:

- `TaskRepository`
- `ListRepository`
- `TagRepository`

The SQLite implementation provides the actual persistence.

The application layer should not know whether a task is stored in SQLite, an in-memory test repository, or a future cloud-backed implementation.

For version 1, only SQLite is needed.

## 9. Application state management

The UI should receive application state through a predictable event-driven flow.

Example:

1. User taps the completion button.
2. UI dispatches a `CompleteTask` command.
3. Application layer validates the operation.
4. Repository updates the database.
5. Application returns the result.
6. UI refreshes its task collection.
7. Any associated reminder is cancelled.

The UI should never optimistically report success if the database operation fails.

However, it can show an immediate loading or pending state to keep interactions responsive.

Keep database operations off the UI thread. A dedicated database worker with a message channel is a reasonable initial implementation for a small local application.

Do not add Tokio or a complex async runtime without a demonstrated need.

## 10. Android integration

The Rust core should remain platform-independent.

Use a small Android integration boundary for capabilities that require operating-system APIs.

### Notifications

The Android adapter is responsible for:

- Requesting notification permission when necessary.
- Scheduling notifications.
- Cancelling notifications.
- Restoring scheduled reminders when appropriate.
- Handling notification taps.

Each task reminder should have a deterministic notification identifier based on its task ID.

### Haptics

Expose a small interface such as:

- `light_feedback()`
- `success_feedback()`
- `warning_feedback()`

### Backup

Use Android's file picker or document-provider facilities to let users select backup destinations and files.

The Rust layer handles JSON serialization, validation, and import/export logic.

### Lifecycle

Handle:

- App restart
- Process termination
- Device restart
- Notification delivery
- Restoring application navigation state

Do not rely on a continuously running background Rust process for reminders.

## 11. Suggested dependencies

| Crate | Purpose |
|---|---|
| `serde` | Serialization |
| `serde_json` | JSON backup |
| `rusqlite` | SQLite |
| `uuid` | Identifiers |
| `time` | Dates and timestamps |
| `thiserror` | Error definitions |
| `tracing` | Logging |
| `tracing-subscriber` | Logging configuration |

Additional dependencies:

- UI framework dependencies.
- Android/JNI integration dependencies.
- A migration helper if useful.
- A channel library if standard channels prove insufficient.

Keep the dependency count small.

Avoid introducing frameworks just because they are popular. Every dependency should solve an actual problem.

## 12. Error handling

Use explicit domain and infrastructure errors.

Examples:

- Task title is empty.
- Task does not exist.
- Invalid date.
- List does not exist.
- Database unavailable.
- Database migration failed.
- Backup file invalid.
- Notification permission denied.

Do not show internal SQLite errors directly to users.

Log technical details through structured logging and show understandable messages in the UI.

Never silently discard user changes after an error.

## 13. Testing strategy

### Domain tests

Test:

- Task validation.
- Completion transitions.
- Priority changes.
- Date calculations.
- Subtask rules.
- List deletion behavior.

### Repository tests

Test:

- Task creation and retrieval.
- Task modification.
- Deletion.
- Search.
- Tag relationships.
- Database migrations.
- Transaction rollback.

### Application tests

Use fake repository implementations to test commands and queries without Android.

### Android tests

Test on a physical device:

- Startup.
- Keyboard behavior.
- Navigation.
- Date and time selection.
- Notification permission.
- Reminder delivery.
- Device restart.
- Backup and restore.
- Dark mode.
- Larger font sizes.

### UI acceptance criteria

- Completing a task does not duplicate or lose it.
- A task remains persisted after an application restart.
- An overdue task is never silently rescheduled.
- Editing a task does not accidentally clear unrelated fields.
- A failed database write produces a visible error.
- Importing an invalid backup does not damage existing data.

## 14. Development roadmap

### Phase 1 — Technical prototype

Evaluate Dioxus and Slint.

Implement:
- Basic navigation
- Today screen
- Task editor
- Light/dark appearance

Confirm Android build and packaging.

### Phase 2 — Core Rust engine

Implement:
- Domain types
- Task validation
- Application commands
- Repository interfaces
- SQLite persistence
- Database migrations

### Phase 3 — Main functionality

Implement:
- Inbox
- Today
- Upcoming
- Lists
- Priorities
- Search
- Subtasks
- Tags

### Phase 4 — Android features

Implement:
- Notifications
- Haptics
- Backup
- Restore
- Lifecycle integration

### Phase 5 — Product polish

Implement:
- Animations
- Accessibility
- Empty states
- Undo interactions
- Performance improvements
- Final visual consistency

### Phase 6 — Release preparation

Complete:
- Physical-device testing
- Data migration testing
- Permission testing
- Backup verification
- Release build
- APK/AAB packaging

---

## 15. Architectural decisions to preserve

The following are important long-term decisions:

1. The Rust domain must not depend on the UI framework.
2. The application layer must not depend on SQLite.
3. Android-specific APIs must remain behind platform interfaces.
4. SQLite is the source of truth for version 1.
5. All user-facing task operations should go through application use cases.
6. All schema changes require migrations.
7. JSON backup files must include a schema version.
8. No cloud dependency should be introduced into the initial application.
9. Recurring tasks should be implemented as a separate feature with explicit scheduling semantics.
10. The application must continue to work without an internet connection.

**Final architectural direction:** a modular Rust workspace, a framework-specific presentation layer, a pure Rust application and domain core, SQLite persistence, and a small Android platform adapter.
