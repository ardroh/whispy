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
</style>
</head>
<body>
<canvas id="c"></canvas>
<script>
const NUM_BARS = 20;
const BAR_WIDTH = 4;
const BAR_GAP = 3;
const MIN_BAR_H = 3;
const SMOOTHING = 0.5;
const PILL_RADIUS = 18;

let levels = new Array(NUM_BARS).fill(0);
let displayLevels = new Array(NUM_BARS).fill(0);
let processing = false;
let processingTime = 0;

window.updateLevels = function(arr) {
  for (let i = 0; i < NUM_BARS && i < arr.length; i++) {
    levels[i] = arr[i];
  }
};

window.setProcessing = function() {
  processing = true;
  processingTime = performance.now();
  for (let i = 0; i < NUM_BARS; i++) levels[i] = 0;
};

window.reset = function() {
  processing = false;
  for (let i = 0; i < NUM_BARS; i++) {
    levels[i] = 0;
    displayLevels[i] = 0;
  }
};

const canvas = document.getElementById('c');
const ctx = canvas.getContext('2d');

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
  const pillW = NUM_BARS * (BAR_WIDTH + BAR_GAP) - BAR_GAP + 28;
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

  if (processing) {
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
