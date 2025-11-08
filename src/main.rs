use qmetaobject::*;

mod crypto;
mod db;

use std::cell::RefCell;
use std::rc::Rc;
use chrono::Utc;
use log::info;

#[derive(QObject, Default)]
struct NoteQuarryBridge {
    base: qt_base_class!(trait QObject),
    
    // Signals
    entries_loaded: qt_signal!(entries: QVariantList),
    entry_selected: qt_signal!(entry_id: i64),
    
    // Slots
    load_entries: qt_method!(fn(&mut self)),
    select_entry: qt_method!(fn(&mut self, index: i32)),
    create_entry: qt_method!(fn(&mut self, title: QString, mode: QString)),
    save_content: qt_method!(fn(&mut self, content: QString)),
    
    // State
    app_state: RefCell<AppState>,
}

struct AppState {
    db: db::Database,
    current_entry_id: Option<i64>,
    current_entry_mode: Option<db::EntryMode>,
    current_page_id: Option<i64>,
    displayed_entry_ids: Vec<i64>,
    master_key: Option<crypto::MasterKey>,
}

impl NoteQuarryBridge {
    fn load_entries(&mut self) {
        let state = self.app_state.borrow();
        match db::entries::get_all(state.db.connection()) {
            Ok(entries) => {
                let qentries: QVariantList = entries
                    .iter()
                    .map(|e| {
                        let mut map = QVariantMap::default();
                        map.insert("id".into(), e.id.unwrap_or(0).into());
                        map.insert("title".into(), e.title.clone().into());
                        map.into()
                    })
                    .collect();
                
                self.entries_loaded(qentries);
            }
            Err(e) => {
                eprintln!("Failed to load entries: {}", e);
            }
        }
    }
    
    fn select_entry(&mut self, index: i32) {
        let mut state = self.app_state.borrow_mut();
        if let Some(&entry_id) = state.displayed_entry_ids.get(index as usize) {
            state.current_entry_id = Some(entry_id);
            self.entry_selected(entry_id);
        }
    }
}

fn main() {
    env_logger::init();
    info!("Starting NoteQuarry (Qt version)...");

    let database = db::init(None).expect("Failed to initialize database");
    
    let app_state = AppState {
        db: database,
        current_entry_id: None,
        current_entry_mode: None,
        current_page_id: None,
        displayed_entry_ids: Vec::new(),
        master_key: None,
    };
    
    qml_register_type::<NoteQuarryBridge>(
        cstr!("NoteQuarry"),
        1,
        0,
        cstr!("Bridge")
    );
    
    let mut engine = QmlEngine::new();
    
    let bridge = QObjectPinned::new(NoteQuarryBridge {
        app_state: RefCell::new(app_state),
        ..Default::default()
    });
    
    engine.set_object_property("bridge".into(), bridge.get_cpp_object());
    engine.load_file("qml/main.qml".into());
    engine.exec();
}