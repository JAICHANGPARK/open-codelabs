# Open Codelabs (ハンズオンシステム)

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)
[![Firebase](https://img.shields.io/badge/firebase-yellow.svg)](https://firebase.google.com/)
[![Supabase](https://img.shields.io/badge/supabase-3FCF8E.svg)](https://supabase.com/)

**Open Codelabs** は、Google Codelab スタイルのハンズオンセッションを簡単に運営・管理できるように設計されたオープンソースプラットフォームです。最新の技術スタックで構築されており、ファシリテーター（管理者）と参加者の両方のロールをサポートしています。コンテンツは Markdown で直接管理するか、AI を使用して自動生成することができます。

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文](README.zh.md)

![Open Codelabs Hero](docs/imgs/20260208-rzac.png)

---

## 🚀 主な特徴

- **ファシリテーターと参加者の分離**: 管理者はコードラボを作成・管理し、参加者は洗練された UI を通じてステップに従うことができます。
- **AI コードラボ生成器**: Google Gemini AI を使用して、ソースコードや参照ドキュメントからプロフェッショナルなコードラボを自動生成し、永続的な対話コンテキストをサポートします。
- **監査ログとバックアップ**: 管理者のアクションを追跡する詳細な監査ログと、システムデータを簡単に管理できるバックアップ/復元機能を提供します。
- **Code Server ワークスペース（任意）**: コードラボごとの code-server ワークスペースを作成し、ステップごとのスナップショット（ブランチ/フォルダモード）とダウンロードを提供します。
- **クイズ・フィードバック・修了証**: クイズやフィードバック提出を修了条件に設定し、検証 URL 付きの修了証を自動発行します。
- **準備ガイド & 資料管理**: 事前準備ガイドを手書きまたは AI 生成し、リンク/ファイルを一括配布できます。
- **ライブワークショップツール**: ライブチャット/DM、リアルタイムのヘルプリクエスト解決キュー、修了証保持者だけを対象にするルーレット抽選を提供します。
- **双方向ライブ画面共有**: ファシリテーターが自分の画面を全参加者に配信すると同時に、参加者の画面をリアルタイムのグリッドビューで監視できます。参加者用のサイズ調整可能なPiPと、ファシリテーター用の全画面拡大(Enlarge)機能をサポートしています。
- **マルチランタイムサポート**: ローカル/プライベートセッション用の **Rust (Axum) + SQLite** バックエンド、またはサーバーレス環境用の **Firebase (Firestore/Hosting)** または **Supabase** デプロイをサポートしています。
- **Google Codelab Look & Feel**: 慣れ親しんだ、読み取りやすい Google スタイルのデザインを採用しています。
- **簡単な外部公開**: `ngrok`、`bore`、`cloudflared`(Cloudflare Tunnel) 統合スクリプトにより、ローカルサーバーを即座に外部に公開し、参加者が QR コードでアクセスできるようにサポートします。
- **多言語対応**: グローバルなワークショップ運営のための i18n サポート（日本語、英語、韓国語、中国語）が組み込まれています。

---

## ⚡ クイックスタート (Quickstart)

数秒でシステムを起動できます：

```bash
# リポジトリをクローン
git clone https://github.com/JAICHANGPARK/open-codelabs.git
cd open-codelabs

# Docker Compose で起動
docker compose up --build
```

### 🦭 Podman ユーザーガイド
Podman を使用している場合は、`podman-compose` を使用できます：
```bash
podman-compose up --build
```
または Podman の Docker 互換レイヤーを使用してください。

### 🧱 事前ビルドイメージを使用 (GHCR)
ローカルビルドを省略したい場合は、公開済みイメージを利用できます：

```bash
cp .env.sample .env
docker compose -f docker-compose.images.yml up
```

---

## 🛠 技術スタック

### フロントエンド
- **フレームワーク**: [SvelteKit 5](https://svelte.dev/) (Vite + TypeScript)
- **ランタイム**: [Bun](https://bun.sh/)
- **スタイリング**: Tailwind CSS 4.0
- **状態管理**: Svelte Runes
- **i18n**: `svelte-i18n`

### バックエンド (セルフホスト)
- **言語**: [Rust](https://www.rust-lang.org/)
- **フレームワーク**: Axum (Tokio stack)
- **データベース**: SQLite (via [SQLx](https://github.com/launchbadge/sqlx))

### クラウド (サーバーレスオプション)
- **プラットフォーム**: [Firebase](https://firebase.google.com/) (Hosting, Firestore, Storage) または [Supabase](https://supabase.com/) (Postgres, Auth, Storage, Realtime)

---

## 📂 プロジェクト構造

```text
open-codelabs/
├── backend/          # Rust Axum API サーバー
│   ├── src/          # API ロジック
│   └── migrations/   # データベースマイグレーション
├── frontend/         # SvelteKit クライアント
│   ├── src/          # コンポーネント、ルート、ライブラリ
│   └── static/       # 静的アセット
├── docs/             # ドキュメント (MkDocs)
├── docker-compose.yml # システムオーケストレーション
└── run-public.sh     # 公開デプロイスクリプト (ngrok/bore/cloudflare)
```

---

## 🚦 はじめに

### 事前準備
- [Docker](https://www.docker.com/) & Docker Compose
- [Bun](https://bun.sh/) (ローカル開発用)
- [Rust](https://www.rust-lang.org/) (ローカルバックエンド開発用)

### 1. Docker で実行 (推奨)
システム全体を起動する最も簡単な方法です。

> **注意**: デフォルトでは、データはホストマシンの `~/open-codelabs` フォルダに保存されます。保存場所をカスタマイズするには、ルートディレクトリの .env ファイルで DATA_VOLUME_PATH を編集してください。
> - **macOS/Linux**: `~/open-codelabs` 
> - **Windows**: `C:/open-codelabs` (スラッシュ `/` を使用)

```bash
docker compose up --build
```
- **フロントエンド**: [http://localhost:5173](http://localhost:5173)
- **バックエンド API**: [http://localhost:8080](http://localhost:8080)

### 2. ローカル開発環境

#### バックエンド
```bash
cd backend
# .env ファイルを作成 (DATABASE_URL=sqlite:data/sqlite.db?mode=rwc)
# 必須: ADMIN_ID, ADMIN_PW
cargo run
```

#### フロントエンド
```bash
cd frontend
bun install
# .env ファイルを作成 (VITE_API_URL=http://localhost:8080)
bun run dev
```

### 3. 環境設定 (.env)

Docker Compose はリポジトリルートの `.env` を読み込みます。`.env.sample` をコピーして `.env` を作成し、必要な値を変更してください。
(ローカル開発は `backend/.env.sample` と `frontend/.env.sample` を最小の開始点として使えます。)

**イメージ (docker-compose.images.yml)**
- `IMAGE_REGISTRY`: 事前ビルドイメージのレジストリ（既定 `ghcr.io`）。
- `IMAGE_NAMESPACE`: イメージのネームスペース/組織名（既定 `open-codelabs`）。
- `IMAGE_TAG`: 取得するイメージタグ（既定 `latest`）。

**Backend**
- `DATABASE_URL`: SQLx 接続文字列。例: `sqlite:/app/data/sqlite.db?mode=rwc`。
- `ADMIN_ID`: 管理者ログイン ID。
- `ADMIN_PW`: 管理者ログインパスワード。
- `AUTH_SECRETS`: JWT 署名用シークレット(カンマ区切り)。先頭が有効キーで、他はローテーション用。未設定時は `ADMIN_PW` を使用。
- `AUTH_ISSUER`: JWT issuer クレーム。
- `AUTH_AUDIENCE`: JWT audience クレーム。
- `ADMIN_SESSION_TTL_SECONDS`: 管理者セッション TTL(秒)。
- `ATTENDEE_SESSION_TTL_SECONDS`: 参加者セッション TTL(秒)。
- `COOKIE_SECURE`: HTTPS 時は `true` (Secure クッキー + `__Host-` プレフィックス)。`COOKIE_SAMESITE=none` には必須。
- `COOKIE_SAMESITE`: `lax`(既定)、`strict`、`none`。
- `TRUST_PROXY`: リバースプロキシ配下で `X-Forwarded-*` ヘッダーを信頼する場合 `true`。
- `CORS_ALLOWED_ORIGINS`: 許可するオリジン一覧(カンマ区切り)。空ならローカルの既定値。
- `RATE_LIMIT_GENERAL_PER_MINUTE`: 一般 API の分/IP 制限。
- `RATE_LIMIT_LOGIN_PER_5_MIN`: ログインの 5 分/IP 制限。
- `RATE_LIMIT_AI_PER_MINUTE`: AI プロキシの分/IP 制限。
- `RATE_LIMIT_UPLOAD_PER_MINUTE`: アップロード/提出 POST の分/IP 制限。
- `CSP_HEADER`: UI 応答の Content-Security-Policy ヘッダー上書き。空なら既定値。
- `HSTS_HEADER`: Strict-Transport-Security ヘッダー上書き(HTTPS のみ)。
- `ALLOWED_GEMINI_MODELS`: 許可する Gemini モデル ID。

**AI**
- `GEMINI_API_KEY`: 既定の Gemini API キー。

**Frontend**
- `VITE_API_URL`: バックエンド API の基底 URL。
- `VITE_ADMIN_ENCRYPTION_PASSWORD`: ブラウザで Gemini API キーを暗号化するパスワード。バックエンド `ADMIN_PW` と一致する必要あり。
- `VITE_USE_SUPABASE`: `true` に設定すると Supabase モード（サーバーレス、Rust バックエンドなし）を有効化します。
- `VITE_SUPABASE_URL`: Supabase プロジェクト URL。
- `VITE_SUPABASE_ANON_KEY`: Supabase の anon キー。
- `VITE_SUPABASE_STORAGE_BUCKET`: Supabase Storage のバケット名（既定 `open-codelabs`）。
- `VITE_ADMIN_ID`: Firebase/Supabase モードの管理者ログイン ID。
- `VITE_ADMIN_PW`: Firebase/Supabase モードの管理者ログインパスワード。
- `FRONTEND_PORT`: フロントサーバーのポート。
- `FRONTEND_HOST`: フロントサーバーのバインドホスト(例: `0.0.0.0`)。

### 4. クラウドデプロイ (AWS / GCP / Firebase)
サーバーレス環境やクラウドで運用する場合は、以下を参照してください。

- **AWS**: コンテナ or VM 配置。 [AWS デプロイガイド](docs/self-hosting/aws.md) を参照。
- **GCP (Cloud Run)**: コンテナ配置。 [GCP デプロイガイド](docs/self-hosting/gcp.md) を参照。
- **Firebase**: 迅速なサーバーレス設定。 [Firebase デプロイガイド](docs/self-hosting/firebase.md) を参照。
- **Supabase**: サーバーレス Postgres + Storage 構成。 [Supabase ガイド](docs/self-hosting/supabase.md) を参照。

---

## 🤖 AI コードラボ生成器
Open Codelabs には、コードを構造化されたチュートリアルに変換する AI 生成器が内蔵されています。
1. 設定で Gemini API キーを入力します。
2. ソースコードまたは技術的な説明を入力します。
3. AI が各ステップ、説明、検証手順を自動生成します。
4. **永続的なスレッド**: AI 生成時の対話コンテキストが維持され、より精緻な微調整が可能です。

---

## 🧭 ファシリテーターツールキット（新機能）
- **ライブモード**: 参加者の進捗リアルタイム追跡、チャット/DM、ヘルプリクエスト即時解決。
- **画面共有監視**: 全参加者の画面をグリッドビューで確認。特定のストリームを拡大して詳細な技術サポートを提供できます。
- **監査ログ**: ログイン、コードラボ作成、設定変更などの管理者操作を記録。
- **バックアップと復元**: 管理パネルからシステム全体の SQLite データベースを簡単にエクスポート/インポート可能。
- **クイズ & フィードバック**: 修了条件の設定と結果集計。
- **準備ガイド & 資料**: 準備ガイドの作成と添付ファイル管理。
- **修了証ルーレット抽選**: 修了証発行済みの参加者を対象に抽選。

---

## 🌐 外部への公開 (ngrok / bore / cloudflare)
ローカルマシンでワークショップを開催する場合、`run-public.sh` スクリプトを使用して外部アクセスを提供できます。

```bash
chmod +x run-public.sh
./run-public.sh --ngrok  # ngrok を使用
# または
./run-public.sh --bore   # bore を使用 (Rust ベース)
# または
./run-public.sh --cloudflare  # Cloudflare Tunnel を使用
```

---

## 📚 ドキュメント
完全なドキュメントは GitHub Pages で確認できます：
**[📖 Open Codelabs ドキュメントを見る](https://JAICHANGPARK.github.io/open-codelabs/)**

追加ガイド:
- [Code Server ワークスペース設定](docs/CODE_SERVER_SETUP.md)

---

## 📄 ライセンス
このプロジェクトは [Apache License 2.0](LICENSE) に基づいてライセンスされています。
