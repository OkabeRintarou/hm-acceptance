const test = require("node:test");
const assert = require("node:assert");
const L = require("./snake-logic.js");

const fixed = (v) => () => v;

test("initial state: 3-long snake heading right, food off the snake", () => {
  const s = L.createState(10, 10, fixed(0));
  assert.strictEqual(s.snake.length, 3);
  assert.strictEqual(s.dir, "right");
  assert.ok(!s.snake.some((p) => p.x === s.food.x && p.y === s.food.y));
});

test("moves one cell per step", () => {
  const s = L.createState(10, 10, fixed(0));
  const head = { ...s.snake[0] };
  L.step(s);
  assert.deepStrictEqual(s.snake[0], { x: head.x + 1, y: head.y });
  assert.strictEqual(s.snake.length, 3);
});

test("cannot reverse direction", () => {
  const s = L.createState(10, 10, fixed(0));
  L.setDirection(s, "left");
  L.step(s);
  assert.strictEqual(s.dir, "right");
});

test("two quick turns within one tick cannot cause a reversal", () => {
  const s = L.createState(10, 10, fixed(0));
  L.setDirection(s, "up");
  L.setDirection(s, "left");
  L.step(s);
  assert.strictEqual(s.dir, "up");
  L.step(s);
  assert.strictEqual(s.dir, "left");
  assert.ok(!s.over);
});

test("eating grows snake and increments score", () => {
  const s = L.createState(10, 10, fixed(0));
  const h = s.snake[0];
  s.food = { x: h.x + 1, y: h.y };
  L.step(s, fixed(0));
  assert.strictEqual(s.score, 1);
  assert.strictEqual(s.snake.length, 4);
  assert.ok(!s.snake.some((p) => p.x === s.food.x && p.y === s.food.y));
});

test("hitting a wall ends the game", () => {
  const s = L.createState(5, 5, fixed(0));
  for (let i = 0; i < 5 && !s.over; i++) L.step(s);
  assert.ok(s.over);
  assert.ok(!s.won);
});

test("running into itself ends the game", () => {
  const s = L.createState(10, 10, fixed(0));
  s.snake = [
    { x: 5, y: 5 }, { x: 5, y: 6 }, { x: 4, y: 6 }, { x: 4, y: 5 }, { x: 4, y: 4 }, { x: 5, y: 4 },
  ];
  s.dir = "up";
  s.food = { x: 0, y: 0 };
  L.setDirection(s, "left");
  L.step(s);
  assert.ok(s.over);
});

test("moving into the cell the tail just left is allowed", () => {
  const s = L.createState(10, 10, fixed(0));
  s.snake = [{ x: 5, y: 5 }, { x: 5, y: 6 }, { x: 6, y: 6 }, { x: 6, y: 5 }];
  s.dir = "up";
  s.food = { x: 0, y: 0 };
  L.setDirection(s, "right");
  L.step(s);
  assert.ok(!s.over);
});

test("filling the board wins", () => {
  const s = L.createState(4, 1, fixed(0));
  s.snake = [{ x: 2, y: 0 }, { x: 1, y: 0 }, { x: 0, y: 0 }];
  s.food = { x: 3, y: 0 };
  L.step(s);
  assert.ok(s.over && s.won);
  assert.strictEqual(s.food, null);
});

test("placeFood returns null on a full board", () => {
  assert.strictEqual(L.placeFood(1, 1, [{ x: 0, y: 0 }], fixed(0)), null);
});
