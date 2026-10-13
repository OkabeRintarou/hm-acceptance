# hm-acceptance
Hello from Helmsman M6 acceptance.

## Snake (ACC-5)

A dependency-free JavaScript Snake game with a responsive canvas, touch direction
buttons, score and saved best score, increasing speed, pause, and restart.

From the repository root, run `python3 -m http.server 8000`, then open
http://localhost:8000/snake/ in a modern browser. Serve the files over HTTP because
the game uses JavaScript modules.

Click **Start game**, then steer with arrow keys, WASD, or the on-screen buttons.
Eat the coral dots to grow. Hitting a wall or your body ends the round. Use
**Pause** or Space to pause/resume, and **Restart** for a fresh round. Switching
away from the tab automatically pauses the game. Best score is saved locally
when browser storage is available.

Run the game logic tests with `node --test snake/game.test.mjs`.
