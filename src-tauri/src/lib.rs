mod db;

use db::{
    batch_import_questions, get_all_questions, get_mistakes_by_tag, init_database, save_attempt,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_database,
            batch_import_questions,
            get_all_questions,
            save_attempt,
            get_mistakes_by_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
