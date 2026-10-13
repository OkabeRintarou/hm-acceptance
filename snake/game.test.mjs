import test from 'node:test';
import assert from 'node:assert/strict';
import { SnakeGame } from './game.mjs';

const running = () => { const game = new SnakeGame(20, () => 0); game.status = 'running'; return game; };

test('moves one cell and preserves length', () => {
  const game = running(); game.step();
  assert.deepEqual(game.snake[0], { x: 11, y: 10 });
  assert.equal(game.snake.length, 3);
});
test('food grows the snake, scores, and respawns in a free cell', () => {
  const game = running(); game.food = { x: 11, y: 10 }; game.step();
  assert.equal(game.score, 10); assert.equal(game.snake.length, 4);
  assert.ok(!game.snake.some(p => p.x === game.food.x && p.y === game.food.y));
});
test('rejects reversal and limits turns to one per tick', () => {
  const game = running(); game.turn('left'); assert.equal(game.pendingDirection, null);
  game.turn('up'); game.turn('left'); game.step();
  assert.deepEqual(game.snake[0], { x: 10, y: 9 });
  game.turn('left'); game.step(); assert.equal(game.direction, 'left');
});
test('wall collision ends the game', () => {
  const game = running(); for (let i = 0; i < 10; i++) game.step();
  assert.equal(game.status, 'over');
});
test('body collision ends the game', () => {
  const game = running();
  game.snake = [{ x: 2, y: 2 }, { x: 3, y: 2 }, { x: 3, y: 3 }, { x: 2, y: 3 }];
  game.step(); assert.equal(game.status, 'over');
});
test('moving into the departing tail is legal', () => {
  const game = running();
  game.snake = [{ x: 2, y: 2 }, { x: 2, y: 3 }, { x: 3, y: 3 }, { x: 3, y: 2 }];
  game.step(); assert.equal(game.status, 'running');
});
test('paused and ready games do not advance; reset clears a round', () => {
  const game = running(); game.status = 'paused'; const before = structuredClone(game.snake);
  game.step(); assert.deepEqual(game.snake, before);
  game.score = 20; game.reset(); game.step();
  assert.equal(game.score, 0); assert.equal(game.status, 'ready'); assert.equal(game.snake.length, 3);
});
test('filling the board wins without trying to spawn food forever', () => {
  const game = new SnakeGame(3, () => 0);
  game.snake = [{ x: 1, y: 0 }, { x: 0, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 2, y: 2 }, { x: 1, y: 2 }, { x: 0, y: 2 }];
  game.direction = 'right'; game.food = { x: 2, y: 0 }; game.status = 'running'; game.step();
  assert.equal(game.status, 'won'); assert.equal(game.food, null); assert.equal(game.snake.length, 9);
});
