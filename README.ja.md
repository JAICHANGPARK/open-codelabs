# Open Codelabs (ハンズオンシステム)

[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/svelte-v5-ff3e00.svg)](https://svelte.dev/)
[![Bun](https://img.shields.io/badge/bun-v1.0+-black.svg)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/docker-blue.svg)](https://www.docker.com/)
[![Firebase](https://img.shields.io/badge/firebase-yellow.svg)](https://firebase.google.com/)

**Open Codelabs** は、Google Codelab スタイルのハンズオンセッションを簡単に運営・管理できるように設計されたオープンソースプラットフォームです。最新の技術スタックで構築されており、ファシリテーター（管理者）と参加者の両方のロールをサポートしています。コンテンツは Markdown で直接管理するか、AI を使用して自動生成することができます。

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md)

---

## 🚀 主な特徴

- **ファシリテーターと参加者の分離**: 管理者はコードラボを作成・管理し、参加者は洗練された UI を通じてステップに従うことができます。
- **AI コードラボ生成器**: Google Gemini AI を使用して、ソースコードや参照ドキュメントからプロフェッショナルなコードラボを自動生成します。
- **マルチランタイムサポート**: ローカル/プライベートセッション用の **Rust (Axum) + SQLite** バックエンド、またはサーバーレス環境用の **Firebase (Firestore/Hosting)** デプロイをサポートしています。
- **Google Codelab Look & Feel**: 慣れ親しんだ、読み取りやすい Google スタイルのデザインを採用しています。
- **簡単な外部公開**: `ngrok` および `bore` 統合スク리プトにより、ローカルサーバーを即座に外部に公開し、参加者が QR コードでアクセスできるようにサポートします。
- **多言語対応**: グローバルなワークショップ運営のための i18n サポートが組み込まれています。

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
- **プラットフォーム**: [Firebase](https://firebase.google.com/) (Hosting, Firestore, Storage)

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
└── run-public.sh     # 公開デプロイスクリプト (ngrok/bore)
```

---

## 🚦 はじめに

### 事前準備
- [Docker](https://www.docker.com/) & Docker Compose
- [Bun](https://bun.sh/) (ローカル開発用)
- [Rust](https://www.rust-lang.org/) (ローカルバックエンド開発用)

### 1. Docker で実行 (推奨)
システム全体を起動する最も簡単な方法です。

> **注意**: デフォルトでは、データはホストマシンの `~/open-codelabs` フォルダに保存されます。保存場所をカスタマイズするには、ルートディレクトリの `.env` ファイルで `DATA_VOLUME_PATH` を編集してください。

```bash
docker-compose up --build
```
- **フロントエンド**: [http://localhost:5173](http://localhost:5173)
- **バックエンド API**: [http://localhost:8080](http://localhost:8080)

### 2. ローカル開発環境

#### バックエンド
```bash
cd backend
# .env ファイルを作成 (DATABASE_URL=sqlite:data/sqlite.db?mode=rwc)
cargo run
```

#### フロントエンド
```bash
cd frontend
bun install
# .env ファイルを作成 (VITE_API_URL=http://localhost:8080)
bun run dev
```

### 3. Firebase デプロイ (サーバーレスモード)
ローカルの Rust サーバーなしで運営したい場合は、Firebase を使用できます。詳細は [DEPLOY_FIREBASE.md](docs/deploy/DEPLOY_FIREBASE.md) を参照してください。

---

## 🤖 AI コードラボ生成器
Open Codelabs には、コードを構造化されたチュートリアルに変換する AI 生成器が内蔵されています。
1. 設定で Gemini API キーを入力します。
2. ソースコードまたは技術的な説明を入力します。
3. AI が各ステップ、説明、検証手順を自動生成します。

---

## 🌐 外部への公開 (ngrok / bore)
ローカルマシンでワークショップを開催する場合、`run-public.sh` スクリプトを使用して外部アクセスを提供できます。

```bash
chmod +x run-public.sh
./run-public.sh --ngrok  # ngrok を使用
# または
./run-public.sh --bore   # bore を使用 (Rust ベース)
```

---

## 📚 ドキュメント
完全なドキュメントは GitHub Pages で確認できます：
**[📖 Open Codelabs ドキュメントを見る](https://JAICHANGPARK.github.io/open-codelabs/)**

---

## 📄 ライセンス
このプロジェクトは [Apache License 2.0](LICENSE) に基づいてライセンスされています。
