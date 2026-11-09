#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Notifications,
    Vibrate,
}

#[derive(Clone, Default)]
pub struct AndroidPermissions;

impl AndroidPermissions {
    pub fn new() -> Self {
        Self
    }

    pub fn check_permission(&self, permission: Permission) -> bool {
        match permission {
            Permission::Notifications => {
                #[cfg(target_os = "android")]
                {
                    // Check ContextCompat.checkSelfPermission(POST_NOTIFICATIONS)
                    true
                }
                #[cfg(not(target_os = "android"))]
                {
                    true
                }
            }
            Permission::Vibrate => true,
        }
    }

    pub fn request_permission(&self, permission: Permission) -> bool {
        tracing::info!(?permission, "Requesting Android permission");
        true
    }
}
