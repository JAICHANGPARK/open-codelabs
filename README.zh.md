# Open Codelabs（动手实践系统）

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)
[![Firebase](https://img.shields.io/badge/firebase-yellow.svg)](https://firebase.google.com/)
[![Supabase](https://img.shields.io/badge/supabase-3FCF8E.svg)](https://supabase.com/)

**Open Codelabs** 是一个开源平台，用于轻松托管与管理 Google Codelab 风格的动手实践课程。基于现代技术栈，支持组织者（Facilitator）与参与者（Attendee）角色，内容可由 Markdown 管理或由 AI 自动生成。

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文](README.zh.md)

---

## 🚀 主要特性

- **组织者与参与者分离**：管理员可创建与管理 Codelab，参与者通过精致的 UI 按步骤学习。
- **AI Codelab 生成器**：使用 Google Gemini AI 从源码或参考文档自动生成专业级教程。
- **Code Server 工作区（可选）**：为每个 Codelab 创建 code-server 工作区，支持分步骤快照（分支/文件夹模式）与下载归档。
- **测验、反馈与证书**：将测验与反馈作为结业条件，并自动生成带验证 URL 的证书。
- **准备指南与资料**：编写或 AI 生成准备指南，并集中管理链接/文件。
- **工作坊实时工具**：实时聊天/私信、求助队列、提交面板与仅限证书持有者的抽奖轮盘。
- **多运行方式**：支持本地/私有运行的 **Rust (Axum) + SQLite** 后端，或 **Firebase (Firestore/Hosting)** / **Supabase** 的无服务器部署。
- **Google Codelab 风格**：熟悉、易读的 Google Codelab 视觉体验。
- **便捷公网访问**：集成 `ngrok`、`bore` 与 `cloudflared` (Cloudflare Tunnel) 脚本，可快速对外开放并生成二维码访问。
- **多语言支持**：内置 i18n，便于全球化工作坊。

---

## ⚡ 快速开始

几秒钟启动系统：

```bash
# 克隆仓库
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs

# Docker Compose 启动
docker compose up --build
```

### 🦭 Podman 用户指南
如果使用 Podman，可使用 `podman-compose`：
```bash
podman-compose up --build
```
或使用 Podman 的 Docker 兼容层。

### 🧱 使用预构建镜像（GHCR）
如果不想本地构建，可使用已发布镜像：

```bash
cp .env.sample .env
docker compose -f docker-compose.images.yml up
```

---

## 🛠 技术栈

### 前端
- **框架**：[SvelteKit 5](https://svelte.dev/) (Vite + TypeScript)
- **运行时**：[Bun](https://bun.sh/)
- **样式**：Tailwind CSS 4.0
- **状态管理**：Svelte Runes
- **i18n**：`svelte-i18n`

### 后端（自托管）
- **语言**：[Rust](https://www.rust-lang.org/)
- **框架**：Axum (Tokio stack)
- **数据库**：SQLite（通过 [SQLx](https://github.com/launchbadge/sqlx)）

### 云端（无服务器选项）
- **平台**：[Firebase](https://firebase.google.com/)（Hosting, Firestore, Storage）或 [Supabase](https://supabase.com/)（Postgres, Auth, Storage, Realtime）

---

## 📂 项目结构

```text
open-codelabs/
├── backend/          # Rust Axum API 服务器
│   ├── src/          # API 逻辑
│   └── migrations/   # 数据库迁移
├── frontend/         # SvelteKit 客户端
│   ├── src/          # 组件、路由与库
│   └── static/       # 静态资源
├── docs/             # 文档 (MkDocs)
├── docker-compose.images.yml # 预构建镜像 compose 文件
├── docker-compose.yml # 系统编排
└── run-public.sh     # 公网发布脚本 (ngrok/bore/cloudflare)
```

---

## 🚦 开始使用

### 前置条件
- [Docker](https://www.docker.com/) & Docker Compose
- [Bun](https://bun.sh/)（本地开发）
- [Rust](https://www.rust-lang.org/)（本地后端开发）

### 1. 使用 Docker 运行（推荐）
最简单的方式启动整个系统。

> **提示**：默认数据存储在主机的 `~/open-codelabs` 目录。可在根目录 `.env` 中修改 `DATA_VOLUME_PATH`。
> - **macOS/Linux**：`~/open-codelabs`
> - **Windows**：`C:/open-codelabs`（建议使用 `/`）

```bash
docker-compose up --build
```
- **前端**：[http://localhost:5173](http://localhost:5173)
- **后端 API**：[http://localhost:8080](http://localhost:8080)

### 2. 本地开发

#### 后端
```bash
cd backend
# 创建 .env (DATABASE_URL=sqlite:data/sqlite.db?mode=rwc)
# 必填：ADMIN_ID, ADMIN_PW
# 可选：见下方环境变量
cargo run
```

#### 前端
```bash
cd frontend
bun install
# 创建 .env (VITE_API_URL=http://localhost:8080)
bun run dev
```

### 3. 环境变量 (.env)

Docker Compose 读取仓库根目录的 `.env`。复制 `.env.sample` 为 `.env` 并按需修改。
（本地开发可参考 `backend/.env.sample`、`frontend/.env.sample` 作为最小起点。）

**镜像 (docker-compose.images.yml)**
- `IMAGE_REGISTRY`：预构建镜像仓库（默认 `ghcr.io`）。
- `IMAGE_NAMESPACE`：镜像命名空间或组织（默认 `open-codelabs`）。
- `IMAGE_TAG`：拉取的镜像标签（默认 `latest`）。

**Backend**
- `DATABASE_URL`：SQLx 连接字符串 (sqlite/postgres)。示例：`sqlite:/app/data/sqlite.db?mode=rwc`。
- `ADMIN_ID`：管理员登录用户名。
- `ADMIN_PW`：管理员登录密码；也用于解密前端加密的 Gemini API Key。
- `AUTH_SECRETS`：JWT 签名密钥（逗号分隔）。第一个为主密钥，其余用于轮换；为空则回退到 `ADMIN_PW`。
- `AUTH_ISSUER`：JWT issuer。
- `AUTH_AUDIENCE`：JWT audience。
- `ADMIN_SESSION_TTL_SECONDS`：管理员会话 TTL（秒）。
- `ATTENDEE_SESSION_TTL_SECONDS`：参与者会话 TTL（秒）。
- `COOKIE_SECURE`：HTTPS 下设为 `true`（Secure Cookie + `__Host-` 前缀），`COOKIE_SAMESITE=none` 时必需。
- `COOKIE_SAMESITE`：`lax`（默认）、`strict` 或 `none`。
- `TRUST_PROXY`：位于反向代理后时设为 `true` 以信任 `X-Forwarded-*` 头。
- `CORS_ALLOWED_ORIGINS`：允许的源（逗号分隔）；为空使用本地默认值。
- `RATE_LIMIT_GENERAL_PER_MINUTE`：通用 API 每分钟/IP 限制。
- `RATE_LIMIT_LOGIN_PER_5_MIN`：登录请求 5 分钟/IP 限制。
- `RATE_LIMIT_AI_PER_MINUTE`：AI 代理请求每分钟/IP 限制。
- `RATE_LIMIT_UPLOAD_PER_MINUTE`：上传/提交 POST 每分钟/IP 限制。
- `CSP_HEADER`：覆盖 UI 的 Content-Security-Policy 头；为空使用默认值。
- `HSTS_HEADER`：覆盖 Strict-Transport-Security 头；仅 HTTPS 生效。
- `ALLOWED_GEMINI_MODELS`：允许的 Gemini 模型 ID 列表（逗号分隔）。

**AI**
- `GEMINI_API_KEY`：未配置管理员密钥时使用的默认 Gemini API Key。

**Frontend**
- `VITE_API_URL`：后端 API 基础 URL（如 `http://localhost:8080`，Docker 内为 `http://backend:8080`）。
- `VITE_ADMIN_ENCRYPTION_PASSWORD`：前端加密 Gemini API Key 的密码；必须与后端 `ADMIN_PW` 一致。
- `VITE_USE_SUPABASE`：设为 `true` 启用 Supabase 模式（无 Rust 后端）。
- `VITE_SUPABASE_URL`：Supabase 项目 URL。
- `VITE_SUPABASE_ANON_KEY`：Supabase anon Key。
- `VITE_SUPABASE_STORAGE_BUCKET`：Supabase Storage 桶名（默认 `open-codelabs`）。
- `VITE_ADMIN_ID`：Firebase/Supabase 模式下的管理员登录 ID。
- `VITE_ADMIN_PW`：Firebase/Supabase 模式下的管理员登录密码。
- `FRONTEND_PORT`：前端服务/容器端口。
- `FRONTEND_HOST`：前端服务绑定地址（如 `0.0.0.0`）。

### 4. 云端部署 (AWS / GCP / Firebase)
如需在无服务器环境或云端运行：
- **AWS**：容器或 VM 部署。见 [AWS 部署指南](docs/self-hosting/aws.md)。
- **GCP (Cloud Run)**：容器部署。见 [GCP 部署指南](docs/self-hosting/gcp.md)。
- **Firebase**：快速无服务器部署。见 [Firebase 部署指南](docs/self-hosting/firebase.md)。
- **Supabase**：无服务器 Postgres + Storage。见 [Supabase 指南](docs/self-hosting/supabase.md)。

---

## 🤖 AI Codelab 生成器
Open Codelabs 内置 AI 生成器，可将代码转换为结构化教程。
1. 在设置中输入 Gemini API Key。
2. 提供源码或技术说明。
3. AI 自动生成步骤、说明与验证流程。

---

## 🧭 组织者工具包（新增）
- **Live 标签页**：实时监控参与者、聊天/私信与求助队列。
- **测验 & 反馈**：设置结业条件并汇总结果。
- **准备指南 & 资料**：编写/AI 生成准备指南并发布链接/文件。
- **提交物管理**：收集并审核参与者上传的产出物。
- **证书抽奖**：仅从证书持有者中公平抽取。

---

## 🌐 公网发布 (ngrok / bore / cloudflare)
在本地举办工作坊时，使用 `run-public.sh` 快速公开访问。

```bash
chmod +x run-public.sh
./run-public.sh --ngrok  # 使用 ngrok
# 或
./run-public.sh --bore   # 使用 bore (Rust)
# 或
./run-public.sh --cloudflare  # 使用 Cloudflare Tunnel
```

---

## 📚 文档
完整文档见 GitHub Pages：
**[📖 查看 Open Codelabs 文档](https://JAICHANGPARK.github.io/open-codelabs/)**

附加指南：
- [Code Server 工作区设置](docs/CODE_SERVER_SETUP.md)

---

## 📄 许可证
本项目基于 [Apache License 2.0](LICENSE) 发布。
