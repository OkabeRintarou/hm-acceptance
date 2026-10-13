import { SnakeGame } from './game.mjs';

const game = new SnakeGame();
const canvas = document.querySelector('#board');
const ctx = canvas.getContext('2d');
const score = document.querySelector('#score');
const bestLabel = document.querySelector('#best');
const overlay = document.querySelector('#overlay');
const title = document.querySelector('#title');
const message = document.querySelector('#message');
const start = document.querySelector('#start');
const pause = document.querySelector('#pause');
const status = document.querySelector('#status');
let best = 0;
try { best = Math.max(0, Number(localStorage.getItem('snake-best')) || 0); } catch { /* Storage is optional. */ }
let timer;

function render() {
  const cell = canvas.width / game.size;
  ctx.fillStyle = '#0e1914';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.strokeStyle = '#1b2b22';
  ctx.lineWidth = 1;
  for (let i = 1; i < game.size; i++) {
    ctx.beginPath(); ctx.moveTo(i * cell, 0); ctx.lineTo(i * cell, canvas.height); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(0, i * cell); ctx.lineTo(canvas.width, i * cell); ctx.stroke();
  }
  if (game.food) {
    ctx.fillStyle = '#f49378';
    ctx.beginPath(); ctx.arc((game.food.x + .5) * cell, (game.food.y + .5) * cell, cell * .3, 0, Math.PI * 2); ctx.fill();
  }
  game.snake.forEach((part, i) => {
    ctx.fillStyle = i === 0 ? '#e2ffa3' : '#a9d568';
    ctx.fillRect(part.x * cell + 2, part.y * cell + 2, cell - 4, cell - 4);
  });
  score.textContent = game.score;
  if (game.score > best) {
    best = game.score;
    try { localStorage.setItem('snake-best', String(best)); } catch { /* Keep the session best. */ }
  }
  bestLabel.textContent = best;
  overlay.hidden = game.status === 'running';
  pause.disabled = !['running', 'paused'].includes(game.status);
  pause.textContent = game.status === 'paused' ? 'Resume' : 'Pause';
  start.textContent = game.status === 'ready' ? 'Start game' : 'Restart';
  const labels = {
    ready: ['Ready to roll?', 'Grab a snack. Watch your tail.'],
    paused: ['Take a breather.', 'Press Space or Resume to keep going.'],
    over: ['Game over', `You scored ${game.score}. Try another round!`],
    won: ['You filled the board!', `Perfect run. Final score: ${game.score}.`],
  };
  if (labels[game.status]) {
    [title.textContent, message.textContent] = labels[game.status];
    status.textContent = `${title.textContent} ${message.textContent}`;
  } else status.textContent = 'Game running';
}

function schedule() {
  clearTimeout(timer);
  if (game.status !== 'running') return;
  timer = setTimeout(() => { game.step(); render(); schedule(); }, Math.max(70, 160 - game.score * .8));
}

function togglePause() {
  if (game.status === 'running') game.status = 'paused';
  else if (game.status === 'paused') game.status = 'running';
  else return;
  render(); schedule();
}

start.addEventListener('click', () => { game.reset(); game.status = 'running'; render(); schedule(); });
pause.addEventListener('click', togglePause);
const keys = { ArrowUp: 'up', ArrowDown: 'down', ArrowLeft: 'left', ArrowRight: 'right', w: 'up', s: 'down', a: 'left', d: 'right' };
document.addEventListener('keydown', event => {
  const direction = keys[event.key] ?? keys[event.key.toLowerCase()];
  if (direction) { event.preventDefault(); game.turn(direction); }
  if (event.code === 'Space' && event.target.tagName !== 'BUTTON') { event.preventDefault(); if (!event.repeat) togglePause(); }
});
document.querySelectorAll('[data-direction]').forEach(button => {
  button.addEventListener('click', () => game.turn(button.dataset.direction));
});
document.addEventListener('visibilitychange', () => {
  if (document.hidden && game.status === 'running') togglePause();
});
render();
