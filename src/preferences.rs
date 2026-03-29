use crate::app::UserEvent;
use crate::config::Config;
use crate::preferences_html;
use tao::dpi::LogicalSize;
use tao::window::{Window, WindowBuilder, WindowId};
use wry::{WebView, WebViewBuilder};

pub struct PreferencesWindow {
    pub window_id: WindowId,
    _window: Window,
    _webview: WebView,
}

impl PreferencesWindow {
    pub fn new(
        event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
        config: &Config,
    ) -> Self {
        let api_key = config.api_key.as_deref().unwrap_or("");
        let current_model = &config.model;
        let current_language = &config.language;
        let current_hotkey = &config.hotkey;
        let html = preferences_html::build(api_key, current_model, current_language, current_hotkey);

        let window = WindowBuilder::new()
            .with_title("Whispy Preferences")
            .with_inner_size(LogicalSize::new(420.0, 520.0))
            .with_resizable(false)
            .with_focused(true)
            .build(event_loop)
            .expect("Failed to create preferences window");

        // Agent apps (LSUIElement) need explicit activation to show windows
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::ActivationPolicy;
            use tao::platform::macos::EventLoopWindowTargetExtMacOS;
            event_loop.set_activation_policy_at_runtime(ActivationPolicy::Accessory);
        }
        window.set_focus();

        let window_id = window.id();

        let webview = WebViewBuilder::new()
            .with_html(&html)
            .with_asynchronous_custom_protocol(
                "whispy".into(),
                move |_id, request, responder| {
                    let uri = request.uri().to_string();
                    let body = request.body().to_vec();
                    std::thread::spawn(move || {
                        let response = handle_request(&uri, &body);
                        let http_response = http::Response::builder()
                            .header("Content-Type", "application/json")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(response.into_bytes())
                            .unwrap();
                        responder.respond(http_response);
                    });
                },
            )
            .build(&window)
            .expect("Failed to create webview");

        Self {
            window_id,
            _window: window,
            _webview: webview,
        }
    }
}

fn handle_request(uri: &str, body: &[u8]) -> String {
    let path = uri
        .strip_prefix("whispy://localhost/")
        .or_else(|| uri.strip_prefix("whispy://localhost"))
        .unwrap_or(uri);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    match path {
        "save" => {
            #[derive(serde::Deserialize)]
            struct SaveReq {
                api_key: String,
                model: String,
                language: String,
                hotkey: String,
            }
            let req: SaveReq = match serde_json::from_slice(body) {
                Ok(r) => r,
                Err(e) => return format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            };
            let mut config = Config::load().unwrap_or_default();
            config.api_key = Some(req.api_key);
            config.model = req.model;
            config.language = req.language;
            config.hotkey = req.hotkey;
            match config.save() {
                Ok(()) => {
                    tracing::info!("Config saved from preferences");
                    r#"{"ok":true}"#.to_string()
                }
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            }
        }
        "fetch_models" => {
            #[derive(serde::Deserialize)]
            struct FetchReq {
                api_key: String,
            }
            let req: FetchReq = match serde_json::from_slice(body) {
                Ok(r) => r,
                Err(e) => return format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            };
            let client = reqwest::Client::new();
            match rt.block_on(fetch_transcription_models(&client, &req.api_key)) {
                Ok(models) => {
                    let json = serde_json::to_string(&models).unwrap();
                    format!(r#"{{"ok":true,"models":{}}}"#, json)
                }
                Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            }
        }
        "test" => {
            #[derive(serde::Deserialize)]
            struct TestReq {
                api_key: String,
                model: String,
            }
            let req: TestReq = match serde_json::from_slice(body) {
                Ok(r) => r,
                Err(e) => return format!(r#"{{"ok":false,"error":"{}"}}"#, e),
            };
            let client = reqwest::Client::new();
            match rt.block_on(test_api(&client, &req.api_key, &req.model)) {
                Ok(msg) => format!(r#"{{"ok":true,"message":"{}"}}"#, msg),
                Err(e) => format!(
                    r#"{{"ok":false,"error":"{}"}}"#,
                    e.to_string().replace('"', r#"\""#)
                ),
            }
        }
        _ => r#"{"ok":false,"error":"unknown endpoint"}"#.to_string(),
    }
}

async fn fetch_transcription_models(
    client: &reqwest::Client,
    api_key: &str,
) -> anyhow::Result<Vec<String>> {
    let response = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("API error: {}", body);
    }

    #[derive(serde::Deserialize)]
    struct ModelsResponse {
        data: Vec<Model>,
    }
    #[derive(serde::Deserialize)]
    struct Model {
        id: String,
    }

    let resp: ModelsResponse = response.json().await?;

    let keywords = ["whisper", "transcri"];
    let mut models: Vec<String> = resp
        .data
        .into_iter()
        .filter(|m| {
            let lower = m.id.to_lowercase();
            keywords.iter().any(|kw| lower.contains(kw))
        })
        .map(|m| m.id)
        .collect();

    models.sort();

    if models.is_empty() {
        models.push("whisper-1".to_string());
    }

    Ok(models)
}

async fn test_api(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
) -> anyhow::Result<String> {
    let response = client
        .get(format!("https://api.openai.com/v1/models/{}", model))
        .bearer_auth(api_key)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        Ok(format!("Connection OK. Model '{}' is available.", model))
    } else if status.as_u16() == 401 {
        anyhow::bail!("Invalid API key")
    } else {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Error ({}): {}", status, body)
    }
}
