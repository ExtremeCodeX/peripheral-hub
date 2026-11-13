#[tauri::command]
fn forget_connection_permissions(app: tauri::AppHandle) -> Result<(), String> {
    native_confirmation::forget(&app)
}

fn validate_external_url(value: &str) -> Result<String, String> {
    if value.chars().any(char::is_control) {
        return Err("O link contém caracteres inválidos.".into());
    }
    let url = tauri::Url::parse(value).map_err(|_| "O link é inválido.".to_string())?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Apenas links HTTPS sem credenciais são permitidos.".into());
    }
    Ok(url.to_string())
}

#[tauri::command]
async fn open_external_url(url: String) -> Result<(), String> {
    let url = validate_external_url(&url)?;
    tauri::async_runtime::spawn_blocking(move || {
        // Argumento separado; nunca executar uma string de shell recebida do frontend.
        let status = std::process::Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|_| "Não foi possível iniciar o navegador padrão.".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("O sistema não conseguiu abrir o link.".to_string())
        }
    })
    .await
    .map_err(|_| "Falha ao solicitar abertura do link.".to_string())?
}

fn main() {
    let builder = tauri::Builder::default();
    let builder = builder.manage(std::sync::Arc::new(std::sync::Mutex::new(
        plugin_host::DevHost::default(),
    )));
    #[cfg(debug_assertions)]
    let builder = builder
        .manage(std::sync::Arc::new(std::sync::Mutex::new(
            starter_host::StarterHost::default(),
        )))
        .invoke_handler(tauri::generate_handler![
            forget_connection_permissions,
            catalog_sync,
            software_distribution,
            plugin_public,
            open_external_url,
            plugin_poc,
            enumerate_passive_usb,
            starter_local
        ]);
    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        forget_connection_permissions,
        catalog_sync,
        software_distribution,
        plugin_public,
        open_external_url,
        plugin_poc,
        enumerate_passive_usb
    ]);
    builder
        .build(tauri::generate_context!())
        .expect("Não foi possível iniciar o Peripheral Hub")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                use tauri::Manager;
                let state = app.state::<std::sync::Arc<std::sync::Mutex<plugin_host::DevHost>>>();
                // A native confirmation may be running a GTK loop while its
                // worker owns the host. Never block the GTK thread on that
                // mutex; destroying the parent cancels the pending dialog.
                if let Ok(mut host) = state.try_lock() {
                    let _ = host.execute(plugin_host::DevAction::Stop {});
                };
                #[cfg(debug_assertions)]
                {
                    let state =
                        app.state::<std::sync::Arc<std::sync::Mutex<starter_host::StarterHost>>>();
                    if let Ok(mut host) = state.try_lock() {
                        let _ = host.execute(starter_host::StarterAction::Stop {});
                    };
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::validate_external_url;

    #[test]
    fn accepts_https_profile_and_release_links() {
        assert!(validate_external_url("https://example.org/creator").is_ok());
        assert!(validate_external_url("https://example.org/project/releases/tag/1.0.0").is_ok());
    }

    #[test]
    fn rejects_non_web_schemes_credentials_and_control_characters() {
        for value in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "http://example.org/",
            "https://user:secret@example.org/",
            "https://example.org/\n",
            "not-a-url",
        ] {
            assert!(validate_external_url(value).is_err(), "{value}");
        }
    }
}
mod plugin_host;
#[cfg(debug_assertions)]
mod starter_host;

mod passive_usb;

mod catalog;
mod distribution;

mod native_confirmation;
mod remembered_permissions;

#[cfg(debug_assertions)]
#[tauri::command]
async fn starter_local(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, std::sync::Arc<std::sync::Mutex<starter_host::StarterHost>>>,
    request: starter_host::StarterAction,
) -> Result<serde_json::Value, serde_json::Value> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .try_lock()
            .map_err(|_| "BUSY".to_string())?
            .execute_with_confirmation(request, |operation| {
                native_confirmation::confirm(window, operation.clone())
            })
    })
    .await
    .map_err(|_| plugin_host::error_detail("HOST_TASK_FAILED".into()))?
    .map_err(plugin_host::error_detail)
}

#[tauri::command]
async fn catalog_sync(app: tauri::AppHandle, refresh: bool) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || catalog::sync(app, refresh))
        .await
        .map_err(|_| "WORKER_ERROR".to_string())?
}

#[tauri::command]
async fn software_distribution(
    request: distribution::Request,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || distribution::execute(request))
        .await
        .map_err(|_| "INSTALLATION_FAILED".to_string())?
}

#[tauri::command]
async fn plugin_public(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, std::sync::Arc<std::sync::Mutex<plugin_host::DevHost>>>,
    request: plugin_host::DevAction,
) -> Result<serde_json::Value, serde_json::Value> {
    match &request {
        plugin_host::DevAction::Start {
            plugin_id,
            development,
        }
        | plugin_host::DevAction::StartHardware {
            plugin_id,
            development,
        } if *development
            || !distribution::registry()
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["package"]["id"].as_str() == Some(plugin_id)) =>
        {
            return Err(plugin_host::error_detail("UNAUTHORIZED_PLUGIN".into()))
        }
        _ => (),
    }
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .try_lock()
            .map_err(|_| "BUSY".to_string())?
            .execute_with_confirmation(request, |operation| {
                native_confirmation::confirm(window, operation.clone())
            })
    })
    .await
    .map_err(|_| plugin_host::error_detail("HOST_TASK_FAILED".into()))?
    .map_err(plugin_host::error_detail)
}

#[tauri::command]
async fn enumerate_passive_usb() -> Result<passive_usb::Inventory, String> {
    tauri::async_runtime::spawn_blocking(passive_usb::enumerate)
        .await
        .map_err(|_| "SYSFS_TASK_FAILED".into())
}

#[tauri::command]
async fn plugin_poc(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, std::sync::Arc<std::sync::Mutex<plugin_host::DevHost>>>,
    request: plugin_host::DevAction,
) -> Result<serde_json::Value, serde_json::Value> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut host = state.try_lock().map_err(|_| "BUSY")?;
        host.execute_with_confirmation(request, |operation| {
            native_confirmation::confirm(window, operation.clone())
        })
    })
    .await
    .map_err(|_| plugin_host::error_detail("HOST_TASK_FAILED".into()))?
    .map_err(plugin_host::error_detail)
}
