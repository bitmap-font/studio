slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let landing = LandingWindow::new()?;

    landing.on_new_project({
        let landing = landing.as_weak();
        move || {
            let wizard = WizardWindow::new().unwrap();
            wizard.show().unwrap();
            landing.upgrade().unwrap().hide().unwrap();
        }
    });
    
    landing.show()?;
    slint::run_event_loop()?;
    landing.hide()?;
    Ok(())
}
