# 贪吃蛇 Web 游戏

纯静态、零依赖的贪吃蛇：直接用浏览器打开 `index.html` 即可游玩。

## 玩法

- 方向键 / WASD 控制方向；空格 开始 / 暂停
- 吃到红色食物 +1 分并变长；撞墙或咬到自己则游戏结束
- 可选慢/中/快三档速度，最高分保存在浏览器 `localStorage`
- 触屏设备支持在棋盘上滑动或使用屏幕方向按钮

## 结构

- `snake-logic.js` — 与 DOM 无关的纯游戏逻辑（可在 Node 中测试）
- `game.js` — Canvas 渲染、输入、计时器
- `index.html` / `style.css` — 页面与样式

## 测试

```sh
node --test snake-game/snake-logic.test.js
```
