# 刷题神器 - 本地化 AI 刷题应用

基于 Tauri 2.0 构建的跨平台桌面应用，专为 M3 MacBook Air 优化。

## 技术栈

- **Tauri 2.0** - Rust 后端 + 原生 WebView
- **React 19 + TypeScript** - 前端框架
- **SQLite** - 本地数据库
- **llama.cpp** - 本地 AI 推理
- **pulldown-cmark** - Markdown 解析
- **KaTeX** - 数学公式渲染
- **Shadcn UI + Tailwind CSS** - UI 组件库

## 项目结构

```
shuati/
├── src-tauri/           # Rust 后端
│   ├── src/
│   │   ├── commands/    # Tauri 命令
│   │   ├── services/    # 业务逻辑服务
│   │   ├── models/      # 数据模型
│   │   └── utils/       # 工具函数
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
│
├── src/                  # React 前端
│   ├── src/
│   │   ├── components/  # UI 组件
│   │   ├── hooks/       # 自定义 Hooks
│   │   ├── lib/         # 工具库
│   │   ├── types/       # TypeScript 类型定义
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── package.json
│   ├── tailwind.config.js
│   ├── tsconfig.json
│   └── vite.config.ts
│
├── package.json          # 根目录 package.json
├── index.html
└── README.md
```

## 快速开始

### 前置条件

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装 Rust 依赖
cd src-tauri
cargo build
```

### 开发模式

```bash
# 启动前端开发服务器
npm run dev
```

### 构建

```bash
# 构建前端
npm run build

# 构建 Tauri 应用
cd src-tauri
cargo build --release
```

## 功能特性

- 📚 本地 Markdown 题库管理
- 🔢 KaTeX 数学公式渲染
- 🤖 本地 AI 推理 (llama.cpp)
- 💾 SQLite 本地数据库
- 🖥️ 跨平台支持 (macOS/Windows)
- ⚡ 针对 M3 MacBook 优化 (Metal 加速)

## 许可证

MIT
