pub fn build() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; }
  html, body {
    width: 100%;
    height: 100%;
    background: transparent;
    overflow: hidden;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
  #pauseBtn {
    position: absolute;
    right: 14px;
    top: 12px;
    width: 34px;
    height: 32px;
    border: 0;
    border-radius: 10px;
    color: rgba(255,255,255,0.92);
    background: rgba(255,255,255,0.16);
    font: 600 16px -apple-system, BlinkMacSystemFont, sans-serif;
    cursor: pointer;
  }
  #pauseBtn:hover { background: rgba(255,255,255,0.28); }
</style>
</head>
<body>
<canvas id="c"></canvas>
<button id="pauseBtn" aria-label="Pause recording">Ⅱ</button>
<script>
const NUM_BARS = 20;
const BAR_WIDTH = 4;
const BAR_GAP = 3;
const MIN_BAR_H = 3;
const SMOOTHING = 0.5;
const PILL_RADIUS = 18;

let levels = new Array(NUM_BARS).fill(0);
let displayLevels = new Array(NUM_BARS).fill(0);
let mode = 'recording';
let processingTime = 0;
let pausedRecordShortcut = '';
let pausedFinishShortcut = '';

window.updateLevels = function(arr) {
  for (let i = 0; i < NUM_BARS && i < arr.length; i++) {
    levels[i] = arr[i];
  }
};

window.setProcessing = function() {
  mode = 'processing';
  pauseBtn.style.display = 'none';
  processingTime = performance.now();
  for (let i = 0; i < NUM_BARS; i++) levels[i] = 0;
};

window.setRecording = function() {
  mode = 'recording';
  pauseBtn.style.display = 'block';
  pauseBtn.textContent = 'Ⅱ';
  pauseBtn.setAttribute('aria-label', 'Pause recording');
};

window.setPaused = function(recordShortcut, finishShortcut) {
  mode = 'paused';
  pauseBtn.style.display = 'block';
  pauseBtn.textContent = '▶';
  pauseBtn.setAttribute('aria-label', 'Resume recording');
  pausedRecordShortcut = recordShortcut;
  pausedFinishShortcut = finishShortcut;
  for (let i = 0; i < NUM_BARS; i++) levels[i] = 0;
};

window.reset = function() {
  mode = 'recording';
  pauseBtn.style.display = 'none';
  pausedRecordShortcut = '';
  pausedFinishShortcut = '';
  for (let i = 0; i < NUM_BARS; i++) {
    levels[i] = 0;
    displayLevels[i] = 0;
  }
};

const canvas = document.getElementById('c');
const ctx = canvas.getContext('2d');
const pauseBtn = document.getElementById('pauseBtn');
pauseBtn.style.display = 'none';
pauseBtn.addEventListener('click', () => window.ipc.postMessage('toggle_pause'));

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

  ctx.clearRect(0, 0, w, h);

  // Pill background
  const pillW = 332;
  const pillH = h - 4;
  const pillX = (w - pillW) / 2;
  const pillY = (h - pillH) / 2;

  ctx.beginPath();
  ctx.roundRect(pillX, pillY, pillW, pillH, PILL_RADIUS);
  ctx.fillStyle = 'rgba(20, 20, 20, 0.92)';
  ctx.fill();

  const totalBarsW = NUM_BARS * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
  const startX = (w - totalBarsW) / 2;
  const midY = h / 2;
  const maxBarH = (pillH - 12) / 2;

  if (mode === 'processing') {
    // Traveling sine wave animation during processing
    const elapsed = (performance.now() - processingTime) / 1000;
    for (let i = 0; i < NUM_BARS; i++) {
      const wave = Math.sin(elapsed * 4 - i * 0.4) * 0.5 + 0.5;
      const level = wave * 0.45 + 0.05;
      displayLevels[i] += (level - displayLevels[i]) * 0.15;

      const barH = Math.max(MIN_BAR_H, displayLevels[i] * maxBarH);
      const x = startX + i * (BAR_WIDTH + BAR_GAP);
      const alpha = 0.4 + displayLevels[i] * 0.4;
      ctx.fillStyle = `rgba(160, 160, 255, ${alpha})`;
      ctx.beginPath();
      ctx.roundRect(x, midY - barH, BAR_WIDTH, barH * 2, 2);
      ctx.fill();
    }
  } else if (mode === 'paused') {
    // Keep the pill calm and legible while the user thinks.
    ctx.fillStyle = 'rgba(255, 196, 80, 0.95)';
    ctx.font = '600 13px -apple-system, BlinkMacSystemFont, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('Paused', w / 2, 19);
    ctx.font = '11px -apple-system, BlinkMacSystemFont, sans-serif';
    ctx.fillStyle = 'rgba(255, 255, 255, 0.86)';
    ctx.fillText(
      pausedRecordShortcut + '  resume   •   ' + pausedFinishShortcut + '  finish',
      w / 2,
      36
    );
  } else {
    // Live waveform bars during recording
    for (let i = 0; i < NUM_BARS; i++) {
      displayLevels[i] += (levels[i] - displayLevels[i]) * SMOOTHING;
      const level = displayLevels[i];

      const barH = Math.max(MIN_BAR_H, level * maxBarH);
      const x = startX + i * (BAR_WIDTH + BAR_GAP);
      const alpha = 0.6 + level * 0.4;
      ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
      ctx.beginPath();
      ctx.roundRect(x, midY - barH, BAR_WIDTH, barH * 2, 2);
      ctx.fill();
    }
  }

  requestAnimationFrame(draw);
}
requestAnimationFrame(draw);
</script>
</body>
</html>"#
        .to_string()
}
