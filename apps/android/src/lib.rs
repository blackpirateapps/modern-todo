use std::str::FromStr;
use std::sync::Arc;
use application::*;
use domain::*;
use platform_android::*;
use slint::{Color, ModelRc, VecModel};
use storage::*;
use time::{Duration, OffsetDateTime};

slint::include_modules!();

fn get_db_path() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        let p = std::path::PathBuf::from("/data/data/com.daymark.todo/files");
        let _ = std::fs::create_dir_all(&p);
        p.join("daymark.db")
    }
    #[cfg(not(target_os = "android"))]
    {
        std::path::PathBuf::from("daymark.db")
    }
}

pub fn create_service() -> Arc<TaskService> {
    let db_path = get_db_path();
    let db = Database::open(&db_path).unwrap_or_else(|_| {
        tracing::warn!("Falling back to in-memory database");
        Database::in_memory().expect("In-memory SQLite failed")
    });

    let task_repo = Arc::new(SqliteTaskRepository::new(db.clone()));
    let list_repo = Arc::new(SqliteListRepository::new(db.clone()));
    let tag_repo = Arc::new(SqliteTagRepository::new(db));

    let notifications = Arc::new(AndroidNotificationService::new());
    let haptics = Arc::new(AndroidHaptics::new());

    // Populate default lists if empty
    if let Ok(lists) = list_repo.get_all() {
        if lists.is_empty() {
            let _ = list_repo.create(&List::new("Personal").unwrap());
            let _ = list_repo.create(&List::new("Work").unwrap());
            let _ = list_repo.create(&List::new("Shopping").unwrap());
        }
    }

    Arc::new(TaskService::new(
        task_repo,
        list_repo,
        tag_repo,
        Some(notifications),
        Some(haptics),
    ))
}

pub fn refresh_ui(window: &AppWindow, service: &Arc<TaskService>) {
    let today = OffsetDateTime::now_utc().date();
    let active_tab = window.get_active_tab();

    // Fetch tasks according to active tab
    let tasks_res = match active_tab {
        0 => {
            window.set_header_title("Today".into());
            window.set_header_subtitle("Focus on your day".into());
            service.get_today_tasks(today)
        }
        1 => {
            window.set_header_title("Inbox".into());
            window.set_header_subtitle("Unassigned thoughts & tasks".into());
            service.get_inbox_tasks()
        }
        2 => {
            window.set_header_title("Upcoming".into());
            window.set_header_subtitle("Scheduled ahead".into());
            service.get_upcoming_tasks(today)
        }
        3 => {
            window.set_header_title("Lists".into());
            window.set_header_subtitle("Organize by category".into());
            service.get_all_lists().map(|_| vec![])
        }
        _ => service.get_today_tasks(today),
    };

    if let Ok(tasks) = tasks_res {
        let items: Vec<TaskItemData> = tasks
            .into_iter()
            .map(|t| {
                let priority_color = match t.priority {
                    Priority::High => Color::from_rgb_u8(255, 59, 48),    // Red
                    Priority::Medium => Color::from_rgb_u8(255, 149, 0),  // Orange
                    Priority::Low => Color::from_rgb_u8(0, 122, 255),     // Blue
                    Priority::None => Color::from_rgb_u8(142, 142, 147),  // Gray
                };

                let is_overdue = t.is_overdue(today);
                let due_date_str = match t.due_date {
                    Some(d) if d == today => "Today".to_string(),
                    Some(d) if d == today.saturating_add(Duration::days(1)) => "Tomorrow".to_string(),
                    Some(d) => format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day()),
                    None => "".to_string(),
                };

                TaskItemData {
                    id: t.id.to_string().into(),
                    title: t.title.as_str().into(),
                    description: t.description.unwrap_or_default().into(),
                    priority: t.priority.display_label().to_uppercase().into(),
                    priority_color,
                    completed: t.completed,
                    due_date: due_date_str.into(),
                    is_overdue,
                    list_name: "".into(),
                }
            })
            .collect();

        window.set_task_list(ModelRc::new(VecModel::from(items)));
    }

    // Refresh lists
    if let Ok(lists) = service.get_all_lists() {
        let list_data: Vec<ListItemData> = lists
            .into_iter()
            .map(|l| {
                let count = service.get_tasks_by_list(&l.id).map(|ts| ts.len()).unwrap_or(0);
                ListItemData {
                    id: l.id.to_string().into(),
                    name: l.name.as_str().into(),
                    color_hex: l.color.unwrap_or_else(|| "#007AFF".to_string()).into(),
                    task_count: count as i32,
                }
            })
            .collect();
        window.set_list_items(ModelRc::new(VecModel::from(list_data)));
    }

    // Refresh statistics
    if let Ok(stats) = service.get_task_statistics(today) {
        window.set_stat_total(stats.total as i32);
        window.set_stat_completed(stats.completed as i32);
        window.set_stat_pending(stats.pending as i32);
        window.set_stat_overdue(stats.overdue as i32);
    }
}

pub fn run_app() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let service = create_service();

    // Initial load
    refresh_ui(&app, &service);

    // Callbacks
    let app_weak = app.as_weak();
    let svc = service.clone();
    app.on_task_completed_toggled(move |id_str| {
        if let Ok(uuid) = uuid::Uuid::parse_str(id_str.as_str()) {
            let task_id = TaskId::from_uuid(uuid);
            if let Ok(Some(task)) = svc.get_today_tasks(OffsetDateTime::now_utc().date()).map(|ts| ts.into_iter().find(|t| t.id == task_id)) {
                if task.completed {
                    let _ = svc.reopen_task(ReopenTaskCommand { id: task_id });
                } else {
                    let _ = svc.complete_task(CompleteTaskCommand { id: task_id });
                }
            } else {
                let _ = svc.complete_task(CompleteTaskCommand { id: task_id });
            }
            if let Some(w) = app_weak.upgrade() {
                refresh_ui(&w, &svc);
            }
        }
    });

    let app_weak = app.as_weak();
    let svc = service.clone();
    app.on_task_deleted(move |id_str| {
        if let Ok(uuid) = uuid::Uuid::parse_str(id_str.as_str()) {
            let _ = svc.delete_task(DeleteTaskCommand {
                id: TaskId::from_uuid(uuid),
                hard_delete: false,
            });
            if let Some(w) = app_weak.upgrade() {
                refresh_ui(&w, &svc);
            }
        }
    });

    let app_weak = app.as_weak();
    let svc = service.clone();
    app.on_tab_selected(move |tab_idx| {
        if let Some(w) = app_weak.upgrade() {
            w.set_active_tab(tab_idx);
            refresh_ui(&w, &svc);
        }
    });

    let app_weak = app.as_weak();
    let svc = service.clone();
    app.on_refresh_requested(move || {
        if let Some(w) = app_weak.upgrade() {
            refresh_ui(&w, &svc);
        }
    });

    let app_weak = app.as_weak();
    let svc = service.clone();
    app.on_add_task_submitted(move |title_str, desc_str, priority_str, due_offset| {
        let title = title_str.trim().to_string();
        if title.is_empty() {
            return;
        }

        let description = if desc_str.trim().is_empty() {
            None
        } else {
            Some(desc_str.to_string())
        };

        let priority = Priority::from_str(priority_str.as_str()).unwrap_or(Priority::None);
        let today = OffsetDateTime::now_utc().date();
        let due_date = match due_offset {
            1 => Some(today),
            2 => Some(today.saturating_add(Duration::days(1))),
            3 => Some(today.saturating_add(Duration::days(7))),
            _ => None,
        };

        let _ = svc.create_task(CreateTaskCommand {
            title,
            description,
            priority,
            list_id: None,
            due_date,
            reminder_at: None,
            parent_task_id: None,
        });

        if let Some(w) = app_weak.upgrade() {
            refresh_ui(&w, &svc);
        }
    });

    app.run()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).expect("Failed to initialize Slint Android backend");
    let _ = run_app();
}
