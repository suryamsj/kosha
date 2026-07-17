mod config;
mod connection;
mod keys;
mod tags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            keys::list_keys,
            keys::ensure_ssh_dir,
            keys::generate_key,
            keys::get_public_key,
            keys::delete_key,
            config::list_hosts,
            config::add_host,
            config::edit_host,
            config::delete_host,
            connection::test_connection,
            tags::list_tags,
            tags::set_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
