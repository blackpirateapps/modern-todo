fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    daymark_android::run_app()
}
