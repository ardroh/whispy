pub fn build() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
    background: #f5f5f7;
    color: #1d1d1f;
    min-height: 100vh;
    -webkit-user-select: none;
    user-select: none;
  }
  header {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 18px 20px 14px;
    background: rgba(245, 245, 247, 0.94);
    border-bottom: 1px solid #d2d2d7;
    backdrop-filter: blur(16px);
  }
  h1 { font-size: 18px; font-weight: 600; }
  .toolbar { display: flex; align-items: center; gap: 8px; }
  button {
    min-width: 64px;
    padding: 7px 12px;
    font-size: 13px;
    font-weight: 500;
    border: 1px solid #d2d2d7;
    border-radius: 8px;
    background: #fff;
    color: #1d1d1f;
    cursor: pointer;
  }
  button:hover { background: #e8e8ed; }
  button:disabled { opacity: 0.5; cursor: default; }
  button.copy { color: #0071e3; }
  button.danger { color: #c10f0f; }
  main { padding: 14px 20px 20px; }
  .status {
    display: none;
    margin-bottom: 12px;
    padding: 10px 12px;
    border-radius: 8px;
    font-size: 13px;
    word-break: break-word;
  }
  .status.success {
    display: block;
    background: #d1f2d9;
    color: #1a7a2e;
    border: 1px solid #a3e4b1;
  }
  .status.error {
    display: block;
    background: #fdd;
    color: #c00;
    border: 1px solid #faa;
  }
  .empty {
    margin-top: 72px;
    text-align: center;
    color: #6e6e73;
    font-size: 14px;
  }
  .list { display: flex; flex-direction: column; gap: 10px; }
  .entry {
    background: #fff;
    border: 1px solid #d2d2d7;
    border-radius: 8px;
    padding: 12px;
  }
  .entry-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 8px;
  }
  time {
    min-width: 0;
    color: #6e6e73;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .text {
    font-size: 14px;
    line-height: 1.45;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    -webkit-user-select: text;
    user-select: text;
  }
</style>
</head>
<body>
  <header>
    <h1>Transcription History</h1>
    <div class="toolbar">
      <button id="refreshBtn" onclick="loadHistory()">Refresh</button>
      <button id="clearBtn" class="danger" onclick="clearHistory()">Clear</button>
    </div>
  </header>
  <main>
    <div id="status" class="status"></div>
    <div id="content"></div>
  </main>
<script>
function setStatus(msg, type) {
  const el = document.getElementById('status');
  el.textContent = msg;
  el.className = msg ? 'status ' + type : 'status';
}

async function apiCall(endpoint, data) {
  const resp = await fetch('whispy://localhost/' + endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data || {})
  });
  return await resp.json();
}

function formatDate(ms) {
  return new Date(ms).toLocaleString([], {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit'
  });
}

function render(entries) {
  const content = document.getElementById('content');
  content.innerHTML = '';
  document.getElementById('clearBtn').disabled = entries.length === 0;
  if (!entries.length) {
    const empty = document.createElement('div');
    empty.className = 'empty';
    empty.textContent = 'No transcriptions yet.';
    content.appendChild(empty);
    return;
  }

  const list = document.createElement('div');
  list.className = 'list';
  entries.forEach(entry => {
    const row = document.createElement('article');
    row.className = 'entry';

    const head = document.createElement('div');
    head.className = 'entry-head';

    const time = document.createElement('time');
    time.textContent = formatDate(entry.created_at_ms);

    const copy = document.createElement('button');
    copy.className = 'copy';
    copy.textContent = 'Copy';
    copy.onclick = () => copyEntry(entry.id, copy);

    const text = document.createElement('div');
    text.className = 'text';
    text.textContent = entry.text;

    head.appendChild(time);
    head.appendChild(copy);
    row.appendChild(head);
    row.appendChild(text);
    list.appendChild(row);
  });
  content.appendChild(list);
}

async function loadHistory() {
  document.getElementById('refreshBtn').disabled = true;
  try {
    const res = await apiCall('list');
    if (res.ok) {
      setStatus('', '');
      render(res.entries || []);
    } else {
      setStatus(res.error || 'Failed to load history.', 'error');
    }
  } catch (e) {
    setStatus('Load error: ' + e.message, 'error');
  }
  document.getElementById('refreshBtn').disabled = false;
}

async function copyEntry(id, button) {
  button.disabled = true;
  const previous = button.textContent;
  try {
    const res = await apiCall('copy', { id });
    if (res.ok) {
      button.textContent = 'Copied';
      setStatus('Copied to clipboard.', 'success');
      setTimeout(() => { button.textContent = previous; }, 1200);
    } else {
      setStatus(res.error || 'Failed to copy.', 'error');
    }
  } catch (e) {
    setStatus('Copy error: ' + e.message, 'error');
  }
  button.disabled = false;
}

async function clearHistory() {
  if (!confirm('Clear transcription history?')) return;
  document.getElementById('clearBtn').disabled = true;
  try {
    const res = await apiCall('clear');
    if (res.ok) {
      setStatus('History cleared.', 'success');
      render([]);
    } else {
      setStatus(res.error || 'Failed to clear history.', 'error');
    }
  } catch (e) {
    setStatus('Clear error: ' + e.message, 'error');
  }
  document.getElementById('clearBtn').disabled = false;
}

loadHistory();
</script>
</body>
</html>"#
        .to_string()
}
