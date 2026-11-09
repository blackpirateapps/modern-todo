pub mod backup;
pub mod haptics;
pub mod notifications;
pub mod permissions;

pub use backup::PlatformBackupService;
pub use haptics::AndroidHaptics;
pub use notifications::AndroidNotificationService;
pub use permissions::{AndroidPermissions, Permission};
