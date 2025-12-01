slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // P0-005: Rust-to-UI property binding (static)
    // We can set the initial property here if we want to override the default "Ready"
    // ui.set_status_text("Phase 0 test".into()); 
    // But let's stick to the default for now, or maybe set it to "Initializing..." then "Ready"
    
    let ui_handle = ui.as_weak();
    let timer = slint::Timer::default();
    
    // P0-006: Rust-to-UI property update (dynamic)
    timer.start(slint::TimerMode::SingleShot, std::time::Duration::from_secs(1), move || {
        let ui = ui_handle.unwrap();
        ui.set_status_text("System Ready".into());
    });

    ui.run()
}
