(function () {
  const { createState, setDirection, step } = window.SnakeLogic;
  const CELL = 24;
  const COLS = 20;
  const ROWS = 20;
  const BEST_KEY = "snake-best-score";

  const canvas = document.getElementById("board");
  const ctx = canvas.getContext("2d");
  const scoreEl = document.getElementById("score");
  const bestEl = document.getElementById("best");
  const overlay = document.getElementById("overlay");
  const overlayMsg = document.getElementById("overlay-msg");
  const speedEl = document.getElementById("speed");
  const startBtn = document.getElementById("start");

  canvas.width = COLS * CELL;
  canvas.height = ROWS * CELL;

  let state = createState(COLS, ROWS);
  let timer = null;
  let paused = false;
  let started = false;
  let best = 0;
  try {
    best = Number(localStorage.getItem(BEST_KEY)) || 0;
  } catch (e) {}
  bestEl.textContent = best;

  function draw() {
    ctx.fillStyle = "#14181f";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = "#1d232d";
    ctx.lineWidth = 1;
    for (let i = 1; i < COLS; i++) {
      ctx.beginPath();
      ctx.moveTo(i * CELL + 0.5, 0);
      ctx.lineTo(i * CELL + 0.5, canvas.height);
      ctx.stroke();
    }
    for (let i = 1; i < ROWS; i++) {
      ctx.beginPath();
      ctx.moveTo(0, i * CELL + 0.5);
      ctx.lineTo(canvas.width, i * CELL + 0.5);
      ctx.stroke();
    }
    if (state.food) {
      ctx.fillStyle = "#ff5a5f";
      ctx.beginPath();
      ctx.arc(state.food.x * CELL + CELL / 2, state.food.y * CELL + CELL / 2, CELL / 2 - 3, 0, Math.PI * 2);
      ctx.fill();
    }
    state.snake.forEach((s, i) => {
      ctx.fillStyle = i === 0 ? "#7CFC9A" : "#3fbf6b";
      ctx.fillRect(s.x * CELL + 1, s.y * CELL + 1, CELL - 2, CELL - 2);
    });
  }

  function showOverlay(msg, btn) {
    overlayMsg.textContent = msg;
    startBtn.textContent = btn;
    overlay.hidden = false;
  }

  function tick() {
    step(state);
    scoreEl.textContent = state.score;
    if (state.score > best) {
      best = state.score;
      bestEl.textContent = best;
      try {
        localStorage.setItem(BEST_KEY, String(best));
      } catch (e) {}
    }
    draw();
    if (state.over) {
      stop();
      started = false;
      showOverlay(state.won ? "你赢了！🎉" : "游戏结束 · 得分 " + state.score, "再来一局");
    }
  }

  function stop() {
    clearInterval(timer);
    timer = null;
  }

  function run() {
    stop();
    timer = setInterval(tick, Number(speedEl.value));
  }

  function start() {
    state = createState(COLS, ROWS);
    scoreEl.textContent = 0;
    paused = false;
    started = true;
    overlay.hidden = true;
    draw();
    run();
  }

  function togglePause() {
    if (!started) return;
    paused = !paused;
    if (paused) {
      stop();
      showOverlay("已暂停", "继续");
    } else {
      overlay.hidden = true;
      run();
    }
  }

  const KEYS = {
    ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right",
    w: "up", s: "down", a: "left", d: "right",
    W: "up", S: "down", A: "left", D: "right",
  };

  document.addEventListener("keydown", (e) => {
    if (e.key === " ") {
      e.preventDefault();
      if (!started) start();
      else togglePause();
      return;
    }
    const dir = KEYS[e.key];
    if (dir) {
      e.preventDefault();
      if (started && !paused) setDirection(state, dir);
    }
  });

  startBtn.addEventListener("click", () => {
    if (started && paused) togglePause();
    else start();
  });
  speedEl.addEventListener("change", () => {
    if (started && !paused) run();
  });

  // Touch: swipe on the board, plus on-screen buttons.
  let touch = null;
  canvas.addEventListener("touchstart", (e) => {
    const t = e.touches[0];
    touch = { x: t.clientX, y: t.clientY };
  }, { passive: true });
  canvas.addEventListener("touchmove", (e) => e.preventDefault(), { passive: false });
  canvas.addEventListener("touchend", (e) => {
    if (!touch) return;
    const t = e.changedTouches[0];
    const dx = t.clientX - touch.x;
    const dy = t.clientY - touch.y;
    touch = null;
    if (Math.max(Math.abs(dx), Math.abs(dy)) < 20) return;
    const dir = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? "right" : "left") : dy > 0 ? "down" : "up";
    if (started && !paused) setDirection(state, dir);
  });
  document.querySelectorAll("[data-dir]").forEach((b) =>
    b.addEventListener("click", () => {
      if (started && !paused) setDirection(state, b.dataset.dir);
    })
  );

  draw();
  showOverlay("按 空格 或点击按钮开始", "开始游戏");
})();
