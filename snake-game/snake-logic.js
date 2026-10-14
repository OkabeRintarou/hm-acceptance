// Pure game logic (no DOM) so it can be unit-tested with Node.
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory();
  else root.SnakeLogic = factory();
})(typeof self !== "undefined" ? self : this, function () {
  const DIRS = {
    up: { x: 0, y: -1 },
    down: { x: 0, y: 1 },
    left: { x: -1, y: 0 },
    right: { x: 1, y: 0 },
  };

  function isOpposite(a, b) {
    return DIRS[a].x + DIRS[b].x === 0 && DIRS[a].y + DIRS[b].y === 0;
  }

  function placeFood(cols, rows, snake, rng) {
    const free = [];
    for (let y = 0; y < rows; y++) {
      for (let x = 0; x < cols; x++) {
        if (!snake.some((s) => s.x === x && s.y === y)) free.push({ x, y });
      }
    }
    if (free.length === 0) return null;
    return free[Math.floor(rng() * free.length)];
  }

  function createState(cols = 20, rows = 20, rng = Math.random) {
    const cx = Math.floor(cols / 2);
    const cy = Math.floor(rows / 2);
    const snake = [
      { x: cx, y: cy },
      { x: cx - 1, y: cy },
      { x: cx - 2, y: cy },
    ];
    return {
      cols,
      rows,
      snake,
      dir: "right",
      queue: [], // buffered direction changes, one consumed per tick
      food: placeFood(cols, rows, snake, rng),
      score: 0,
      over: false,
      won: false,
    };
  }

  function setDirection(state, dir) {
    if (!DIRS[dir] || state.over) return;
    const last = state.queue.length ? state.queue[state.queue.length - 1] : state.dir;
    if (dir === last || isOpposite(dir, last)) return;
    if (state.queue.length < 2) state.queue.push(dir);
  }

  // Advances the game one tick. Walls are deadly.
  function step(state, rng = Math.random) {
    if (state.over) return state;
    if (state.queue.length) state.dir = state.queue.shift();
    const d = DIRS[state.dir];
    const head = state.snake[0];
    const next = { x: head.x + d.x, y: head.y + d.y };

    if (next.x < 0 || next.y < 0 || next.x >= state.cols || next.y >= state.rows) {
      state.over = true;
      return state;
    }

    const eating = state.food && next.x === state.food.x && next.y === state.food.y;
    // The tail moves away this tick unless we grow, so it is not an obstacle.
    const body = eating ? state.snake : state.snake.slice(0, -1);
    if (body.some((s) => s.x === next.x && s.y === next.y)) {
      state.over = true;
      return state;
    }

    state.snake.unshift(next);
    if (eating) {
      state.score += 1;
      state.food = placeFood(state.cols, state.rows, state.snake, rng);
      if (!state.food) {
        state.over = true;
        state.won = true;
      }
    } else {
      state.snake.pop();
    }
    return state;
  }

  return { DIRS, isOpposite, placeFood, createState, setDirection, step };
});
