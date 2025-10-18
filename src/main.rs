slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    // Set up callbacks
    let ui_handle = ui.as_weak();
    ui.on_create_journal(move |title| {
        println!("Creating journal: {}", title);
        // Add your logic here
    });

    let ui_handle = ui.as_weak();
    ui.on_select_journal(move |index| {
        println!("Selected journal at index: {}", index);
        // Add your logic here
    });

    let ui_handle = ui.as_weak();
    ui.on_save_entry(move |title, content| {
        println!("Saving entry: {} - {}", title, content);
        // Add your logic here
    });

    // Set initial data
    ui.set_journal_list(slint::VecModel::from_slice(&[
        "Work Journal".into(),
        "Personal Thoughts".into(),
        "Project Ideas".into(),
    ]));

    ui.run()
}
