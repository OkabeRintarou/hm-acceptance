export const DIRECTIONS = {
  up: { x: 0, y: -1 }, down: { x: 0, y: 1 },
  left: { x: -1, y: 0 }, right: { x: 1, y: 0 },
};

export class SnakeGame {
  constructor(size = 20, random = Math.random) {
    this.size = size;
    this.random = random;
    this.reset();
  }

  reset() {
    const mid = Math.floor(this.size / 2);
    this.snake = [{ x: mid, y: mid }, { x: mid - 1, y: mid }, { x: mid - 2, y: mid }];
    this.direction = 'right';
    this.pendingDirection = null;
    this.score = 0;
    this.status = 'ready';
    this.placeFood();
  }

  placeFood() {
    const free = [];
    for (let y = 0; y < this.size; y++) {
      for (let x = 0; x < this.size; x++) {
        if (!this.snake.some(part => part.x === x && part.y === y)) free.push({ x, y });
      }
    }
    this.food = free.length ? free[Math.floor(this.random() * free.length)] : null;
    if (!this.food) this.status = 'won';
  }

  turn(direction) {
    if (!DIRECTIONS[direction] || this.pendingDirection || this.status !== 'running') return;
    const current = DIRECTIONS[this.direction];
    const next = DIRECTIONS[direction];
    if (current.x + next.x === 0 && current.y + next.y === 0) return;
    if (direction !== this.direction) this.pendingDirection = direction;
  }

  step() {
    if (this.status !== 'running') return;
    this.direction = this.pendingDirection ?? this.direction;
    this.pendingDirection = null;
    const delta = DIRECTIONS[this.direction];
    const head = { x: this.snake[0].x + delta.x, y: this.snake[0].y + delta.y };
    const eating = head.x === this.food?.x && head.y === this.food?.y;
    const body = eating ? this.snake : this.snake.slice(0, -1);
    if (head.x < 0 || head.y < 0 || head.x >= this.size || head.y >= this.size ||
        body.some(part => part.x === head.x && part.y === head.y)) {
      this.status = 'over';
      return;
    }
    this.snake.unshift(head);
    if (eating) {
      this.score += 10;
      this.placeFood();
    } else this.snake.pop();
  }
}
