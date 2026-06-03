use crate::app::UserEvent;
use crate::history;
use crate::history_html;
use tao::dpi::LogicalSize;
use tao::window::{Window, WindowBuilder, WindowId};
use wry::{WebView, WebViewBuilder};

pub struct HistoryWindow {
    pub window_id: WindowId,
    _window: Window,
    _webview: WebView,
}

impl HistoryWindow {
    pub fn new(event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>) -> Self {
        let html = history_html::build();
        let window = WindowBuilder::new()
            .with_title("Whispy History")
            .with_inner_size(LogicalSize::new(520.0, 620.0))
            .with_min_inner_size(LogicalSize::new(420.0, 420.0))
            .with_resizable(true)
            .with_focused(true)
            .build(event_loop)
            .expect("Failed to create history window");

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
            .with_asynchronous_custom_protocol("whispy".into(), move |_id, request, responder| {
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
            })
            .build(&window)
            .expect("Failed to create history webview");

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

    let response = match path {
        "list" => list_response(),
        "copy" => copy_response(body),
        "clear" => clear_response(),
        _ => ApiResponse::error("unknown endpoint"),
    };

    serde_json::to_string(&response)
        .unwrap_or_else(|_| r#"{"ok":false,"error":"Failed to encode response"}"#.to_string())
}

fn list_response() -> ApiResponse {
    match history::list() {
        Ok(entries) => ApiResponse::with_entries(entries),
        Err(e) => ApiResponse::error(&e.to_string()),
    }
}

fn copy_response(body: &[u8]) -> ApiResponse {
    #[derive(serde::Deserialize)]
    struct CopyReq {
        id: String,
    }

    let req: CopyReq = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(e) => return ApiResponse::error(&e.to_string()),
    };

    match history::copy_entry(&req.id) {
        Ok(()) => ApiResponse::ok(),
        Err(e) => ApiResponse::error(&e.to_string()),
    }
}

fn clear_response() -> ApiResponse {
    match history::clear() {
        Ok(()) => ApiResponse::ok(),
        Err(e) => ApiResponse::error(&e.to_string()),
    }
}

#[derive(serde::Serialize)]
struct ApiResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<history::HistoryEntry>>,
}

impl ApiResponse {
    fn ok() -> Self {
        Self {
            ok: true,
            error: None,
            entries: None,
        }
    }

    fn with_entries(entries: Vec<history::HistoryEntry>) -> Self {
        Self {
            ok: true,
            error: None,
            entries: Some(entries),
        }
    }

    fn error(error: &str) -> Self {
        Self {
            ok: false,
            error: Some(error.to_string()),
            entries: None,
        }
    }
}
