const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("pl", "Polish"),
    ("de", "German"),
    ("fr", "French"),
    ("es", "Spanish"),
    ("it", "Italian"),
    ("pt", "Portuguese"),
    ("nl", "Dutch"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh", "Chinese"),
    ("ru", "Russian"),
    ("uk", "Ukrainian"),
    ("ar", "Arabic"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("fi", "Finnish"),
    ("el", "Greek"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("hu", "Hungarian"),
    ("id", "Indonesian"),
    ("ms", "Malay"),
    ("no", "Norwegian"),
    ("ro", "Romanian"),
    ("sk", "Slovak"),
    ("sv", "Swedish"),
    ("th", "Thai"),
    ("tr", "Turkish"),
    ("vi", "Vietnamese"),
];

const HOTKEYS: &[(&str, &str)] = &[
    ("ctrl+shift+cmd+space", "Ctrl + Shift + Cmd + Space"),
    ("cmd+shift+space", "Cmd + Shift + Space"),
];

pub fn build(
    api_key: &str,
    current_model: &str,
    current_language: &str,
    current_hotkey: &str,
) -> String {
    let hotkey_options: String = HOTKEYS
        .iter()
        .map(|(value, label)| {
            let selected = if *value == current_hotkey { " selected" } else { "" };
            format!(r#"<option value="{value}"{selected}>{label}</option>"#)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let lang_options_auto: String = {
        let selected = if current_language.trim().eq_ignore_ascii_case("auto") {
            " selected"
        } else {
            ""
        };
        format!(r#"<option value="auto"{selected}>Auto (detect language)</option>"#)
    };
    let lang_options_rest: String = LANGUAGES
        .iter()
        .map(|(code, name)| {
            let selected = if *code == current_language { " selected" } else { "" };
            format!(r#"<option value="{code}"{selected}>{name}</option>"#)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let lang_options = format!("{lang_options_auto}\n{lang_options_rest}");

    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
    background: #f5f5f7; color: #1d1d1f; padding: 24px;
    -webkit-user-select: none; user-select: none;
  }}
  h1 {{ font-size: 18px; font-weight: 600; margin-bottom: 20px; }}
  .field {{ margin-bottom: 16px; }}
  label {{
    display: block; font-size: 13px; font-weight: 500;
    color: #6e6e73; margin-bottom: 6px;
  }}
  input, select {{
    width: 100%; padding: 8px 12px; font-size: 14px;
    border: 1px solid #d2d2d7; border-radius: 8px;
    background: #fff; color: #1d1d1f; outline: none;
    -webkit-user-select: text; user-select: text;
  }}
  input:focus, select:focus {{
    border-color: #0071e3; box-shadow: 0 0 0 3px rgba(0,113,227,0.15);
  }}
  .buttons {{ display: flex; gap: 8px; margin-top: 20px; }}
  button {{
    padding: 8px 16px; font-size: 14px; font-weight: 500;
    border: 1px solid #d2d2d7; border-radius: 8px;
    cursor: pointer; background: #fff; color: #1d1d1f;
  }}
  button:hover {{ background: #e8e8ed; }}
  button:disabled {{ opacity: 0.5; cursor: default; }}
  button.primary {{ flex: 1; background: #0071e3; color: #fff; border-color: #0071e3; }}
  button.primary:hover {{ background: #0077ed; }}
  button.test {{ background: #34c759; color: #fff; border-color: #34c759; }}
  button.test:hover {{ background: #30d158; }}
  button.fetch {{
    background: #5856d6; color: #fff; border-color: #5856d6;
    font-size: 12px; padding: 4px 10px;
  }}
  button.fetch:hover {{ background: #6361da; }}
  .status {{
    margin-top: 12px; padding: 10px 14px; border-radius: 8px;
    font-size: 13px; display: none; word-break: break-word;
  }}
  .status.success {{ display: block; background: #d1f2d9; color: #1a7a2e; border: 1px solid #a3e4b1; }}
  .status.error {{ display: block; background: #fdd; color: #c00; border: 1px solid #faa; }}
  .status.loading {{ display: block; background: #e8f0fe; color: #1a73e8; border: 1px solid #aecbfa; }}
  .hint {{ font-size: 11px; color: #8e8e93; margin-top: 4px; }}
  .model-header {{
    display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;
  }}
  .model-header label {{ margin-bottom: 0; }}
</style>
</head>
<body>
  <h1>Whispy Preferences</h1>
  <div class="field">
    <label for="api_key">OpenAI API Key</label>
    <input type="password" id="api_key" value="{api_key}" placeholder="sk-...">
    <div class="hint">Stored locally in ~/Library/Application Support/whispy/</div>
  </div>
  <div class="field">
    <div class="model-header">
      <label for="model">Transcription Model</label>
      <button class="fetch" id="fetchBtn" onclick="fetchModels()">Refresh Models</button>
    </div>
    <select id="model">
      <option value="{current_model}" selected>{current_model}</option>
    </select>
    <div class="hint" id="model-hint">Models are fetched from your OpenAI account</div>
  </div>
  <div class="field">
    <label for="language">Language</label>
    <select id="language">{lang_options}</select>
    <div class="hint">Auto uses Whisper default (language detection). Or choose a language to bias recognition.</div>
  </div>
  <div class="field">
    <label for="hotkey">Shortcut</label>
    <select id="hotkey">{hotkey_options}</select>
    <div class="hint">Global shortcut to start/stop recording (takes effect after save)</div>
  </div>
  <div class="buttons">
    <button class="test" id="testBtn" onclick="testApi()">Test API</button>
    <button class="primary" id="saveBtn" onclick="saveConfig()">Save</button>
  </div>
  <div class="status" id="status"></div>
  <script>
    const CURRENT_MODEL = "{current_model}";
    function setStatus(msg, type) {{
      const el = document.getElementById('status');
      el.textContent = msg;
      el.className = 'status ' + type;
    }}
    async function apiCall(endpoint, data) {{
      const resp = await fetch('whispy://localhost/' + endpoint, {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(data)
      }});
      return await resp.json();
    }}
    async function saveConfig() {{
      const key = document.getElementById('api_key').value.trim();
      const model = document.getElementById('model').value;
      const language = document.getElementById('language').value;
      const hotkey = document.getElementById('hotkey').value;
      document.getElementById('saveBtn').disabled = true;
      try {{
        const res = await apiCall('save', {{ api_key: key, model: model, language: language, hotkey: hotkey }});
        setStatus(res.ok ? 'Settings saved.' : 'Failed to save: ' + res.error,
                  res.ok ? 'success' : 'error');
      }} catch(e) {{ setStatus('Save error: ' + e.message, 'error'); }}
      document.getElementById('saveBtn').disabled = false;
    }}
    async function testApi() {{
      const key = document.getElementById('api_key').value.trim();
      const model = document.getElementById('model').value;
      if (!key) {{ setStatus('Please enter an API key first.', 'error'); return; }}
      document.getElementById('testBtn').disabled = true;
      setStatus('Testing API connection...', 'loading');
      try {{
        const res = await apiCall('test', {{ api_key: key, model: model }});
        setStatus(res.ok ? res.message : res.error, res.ok ? 'success' : 'error');
      }} catch(e) {{ setStatus('Test error: ' + e.message, 'error'); }}
      document.getElementById('testBtn').disabled = false;
    }}
    async function fetchModels() {{
      const key = document.getElementById('api_key').value.trim();
      if (!key) {{ setStatus('Please enter an API key to fetch models.', 'error'); return; }}
      document.getElementById('fetchBtn').disabled = true;
      document.getElementById('model-hint').textContent = 'Fetching models...';
      try {{
        const res = await apiCall('fetch_models', {{ api_key: key }});
        if (res.ok) {{
          const select = document.getElementById('model');
          const prev = select.value;
          select.innerHTML = '';
          res.models.forEach(id => {{
            const opt = document.createElement('option');
            opt.value = id; opt.textContent = id;
            if (id === prev || id === CURRENT_MODEL) opt.selected = true;
            select.appendChild(opt);
          }});
          document.getElementById('model-hint').textContent =
            res.models.length + ' transcription model(s) found';
        }} else {{
          document.getElementById('model-hint').textContent = 'Failed to fetch models';
          setStatus(res.error, 'error');
        }}
      }} catch(e) {{
        document.getElementById('model-hint').textContent = 'Failed to fetch models';
        setStatus('Fetch error: ' + e.message, 'error');
      }}
      document.getElementById('fetchBtn').disabled = false;
    }}
    if (document.getElementById('api_key').value.trim()) {{ fetchModels(); }}
  </script>
</body>
</html>"##
    )
}
