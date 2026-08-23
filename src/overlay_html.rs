pub fn build() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  html, body {
    width: 100%;
    height: 100%;
    background: transparent;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  }
  canvas {
    position: absolute;
    inset: 0;
    display: block;
    width: 100%;
    height: 100%;
  }
  #status {
    position: absolute;
    left: 20px;
    top: 16px;
    display: flex;
    align-items: center;
    width: 78px;
    height: 34px;
    color: rgba(255,255,255,0.94);
    pointer-events: none;
  }
  #statusDot {
    flex: 0 0 auto;
    width: 8px;
    height: 8px;
    margin-right: 8px;
    border-radius: 50%;
    background: #ff5c68;
    box-shadow: 0 0 0 3px rgba(255,92,104,0.16);
  }
  #statusDot.paused {
    background: #ffc450;
    box-shadow: 0 0 0 3px rgba(255,196,80,0.16);
  }
  #statusDot.processing {
    background: #a8a5ff;
    box-shadow: 0 0 0 3px rgba(168,165,255,0.16);
  }
  #statusCopy { min-width: 0; }
  #statusLabel {
    display: block;
    font-size: 12px;
    font-weight: 650;
    line-height: 15px;
    white-space: nowrap;
  }
  #statusHint {
    display: block;
    color: rgba(255,255,255,0.48);
    font-size: 9px;
    line-height: 12px;
    white-space: nowrap;
  }
  button {
    position: absolute;
    top: 15px;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 34px;
    border: 1px solid rgba(255,255,255,0.16);
    border-radius: 10px;
    color: rgba(255,255,255,0.92);
    background: rgba(255,255,255,0.12);
    font: 600 11px -apple-system, BlinkMacSystemFont, sans-serif;
    letter-spacing: -0.1px;
    cursor: pointer;
    transition: background 120ms ease, border-color 120ms ease, transform 120ms ease;
  }
  button:hover {
    background: rgba(255,255,255,0.22);
    border-color: rgba(255,255,255,0.28);
  }
  button:active { transform: scale(0.97); }
  button:focus { outline: 2px solid rgba(168,165,255,0.85); outline-offset: 2px; }
  .buttonIcon {
    margin-right: 6px;
    color: rgba(255,255,255,0.68);
    font-size: 14px;
    line-height: 1;
  }
  #pauseBtn {
    right: 127px;
    width: 82px;
  }
  #finishBtn {
    right: 15px;
    width: 104px;
    border-color: rgba(168,165,255,0.44);
    background: rgba(113,108,220,0.72);
  }
  #finishBtn:hover {
    border-color: rgba(202,200,255,0.75);
    background: rgba(126,120,240,0.88);
  }
</style>
</head>
<body>
<canvas id="c"></canvas>
<div id="status" aria-live="polite">
  <span id="statusDot"></span>
  <span id="statusCopy">
    <span id="statusLabel">Listening</span>
    <span id="statusHint">Recording</span>
  </span>
</div>
<button id="pauseBtn" type="button" aria-label="Pause recording">
  <span class="buttonIcon">Ⅱ</span><span>Pause</span>
</button>
<button id="finishBtn" type="button" aria-label="Finish recording and paste">
  <span class="buttonIcon">✓</span><span>Finish &amp; Paste</span>
</button>
<script>
const NUM_BARS = 18;
const BAR_WIDTH = 4;
const BAR_GAP = 3;
const MIN_BAR_H = 3;
const SMOOTHING = 0.5;
const PILL_RADIUS = 28;
const WAVE_START_X = 112;

let levels = new Array(NUM_BARS).fill(0);
let displayLevels = new Array(NUM_BARS).fill(0);
let mode = 'idle';
let processingTime = 0;

window.updateLevels = function(arr) {
  for (let i = 0; i < NUM_BARS && i < arr.length; i++) {
    levels[i] = arr[i];
  }
};

window.setProcessing = function() {
  mode = 'processing';
  pauseBtn.style.display = 'none';
  finishBtn.style.display = 'none';
  statusLabel.textContent = 'Transcribing';
  statusHint.textContent = 'Working…';
  statusDot.className = 'processing';
  processingTime = performance.now();
  for (let i = 0; i < NUM_BARS; i++) levels[i] = 0;
};

window.setRecording = function() {
  mode = 'recording';
  pauseBtn.style.display = 'flex';
  finishBtn.style.display = 'flex';
  pauseBtn.innerHTML = '<span class="buttonIcon">Ⅱ</span><span>Pause</span>';
  pauseBtn.setAttribute('aria-label', 'Pause recording');
  pauseBtn.title = 'Pause recording';
  finishBtn.title = 'Finish recording and paste';
  statusLabel.textContent = 'Listening';
  statusHint.textContent = 'Recording';
  statusDot.className = '';
};

window.setPaused = function(pauseShortcut, finishShortcut) {
  mode = 'paused';
  pauseBtn.style.display = 'flex';
  finishBtn.style.display = 'flex';
  pauseBtn.innerHTML = '<span class="buttonIcon">▶</span><span>Resume</span>';
  pauseBtn.setAttribute('aria-label', 'Resume recording');
  pauseBtn.title = 'Resume recording (' + pauseShortcut + ')';
  finishBtn.title = 'Finish recording and paste (' + finishShortcut + ')';
  statusLabel.textContent = 'Paused';
  statusHint.textContent = 'Ready when you are';
  statusDot.className = 'paused';
  for (let i = 0; i < NUM_BARS; i++) levels[i] = 0;
};

window.reset = function() {
  mode = 'idle';
  pauseBtn.style.display = 'none';
  finishBtn.style.display = 'none';
  statusLabel.textContent = 'Listening';
  statusHint.textContent = 'Recording';
  statusDot.className = '';
  for (let i = 0; i < NUM_BARS; i++) {
    levels[i] = 0;
    displayLevels[i] = 0;
  }
};

const canvas = document.getElementById('c');
const ctx = canvas.getContext('2d');
const pauseBtn = document.getElementById('pauseBtn');
const finishBtn = document.getElementById('finishBtn');
const statusDot = document.getElementById('statusDot');
const statusLabel = document.getElementById('statusLabel');
const statusHint = document.getElementById('statusHint');
pauseBtn.style.display = 'none';
finishBtn.style.display = 'none';

pauseBtn.addEventListener('click', () => window.ipc.postMessage('toggle_pause'));
finishBtn.addEventListener('click', () => window.ipc.postMessage('finish'));

function resize() {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * dpr;
  canvas.height = canvas.clientHeight * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}
resize();
window.addEventListener('resize', resize);

function draw() {
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  const pillW = w - 16;
  const pillH = h - 8;
  const pillX = (w - pillW) / 2;
  const pillY = (h - pillH) / 2;

  ctx.clearRect(0, 0, w, h);

  ctx.save();
  ctx.shadowColor = 'rgba(0, 0, 0, 0.24)';
  ctx.shadowBlur = 14;
  ctx.shadowOffsetY = 5;
  ctx.beginPath();
  ctx.roundRect(pillX, pillY, pillW, pillH, PILL_RADIUS);
  ctx.fillStyle = 'rgba(20, 20, 24, 0.94)';
  ctx.fill();
  ctx.restore();

  ctx.beginPath();
  ctx.roundRect(pillX, pillY, pillW, pillH, PILL_RADIUS);
  ctx.strokeStyle = 'rgba(255,255,255,0.11)';
  ctx.lineWidth = 1;
  ctx.stroke();

  const totalBarsW = NUM_BARS * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
  const startX = mode === 'processing' ? (w - totalBarsW) / 2 : WAVE_START_X;
  const midY = h / 2;
  const maxBarH = (pillH - 18) / 2;

  if (mode === 'processing') {
    const elapsed = (performance.now() - processingTime) / 1000;
    for (let i = 0; i < NUM_BARS; i++) {
      const wave = Math.sin(elapsed * 4 - i * 0.4) * 0.5 + 0.5;
      const level = wave * 0.45 + 0.05;
      displayLevels[i] += (level - displayLevels[i]) * 0.15;
      drawBar(startX, midY, maxBarH, i, displayLevels[i], 'rgba(168,165,255,');
    }
  } else if (mode === 'paused') {
    for (let i = 0; i < NUM_BARS; i++) {
      const level = 0.08 + (i % 3) * 0.025;
      displayLevels[i] += (level - displayLevels[i]) * 0.12;
      drawBar(startX, midY, maxBarH, i, displayLevels[i], 'rgba(255,196,80,');
    }
  } else if (mode === 'recording') {
    for (let i = 0; i < NUM_BARS; i++) {
      displayLevels[i] += (levels[i] - displayLevels[i]) * SMOOTHING;
      drawBar(startX, midY, maxBarH, i, displayLevels[i], 'rgba(255,255,255,');
    }
  }

  requestAnimationFrame(draw);
}

function drawBar(startX, midY, maxBarH, index, level, color) {
  const barH = Math.max(MIN_BAR_H, level * maxBarH);
  const x = startX + index * (BAR_WIDTH + BAR_GAP);
  const alpha = 0.56 + level * 0.44;
  ctx.fillStyle = color + alpha + ')';
  ctx.beginPath();
  ctx.roundRect(x, midY - barH, BAR_WIDTH, barH * 2, 2);
  ctx.fill();
}

requestAnimationFrame(draw);
</script>
</body>
</html>"#
        .to_string()
}
