#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let router = match tauri::async_runtime::block_on(build_router()) {
        Ok(router) => router,
        Err(err) => {
            eprintln!("Failed to initialize application: {err}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_axum::init(router))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn build_router() -> Result<axum::Router, eks::AppError> {
    eks::logging::init();

    let state = eks::AppState::new()?;

    state.store.load().await?;

    Ok(eks::router::create(state.clone()).with_state(state))
}
