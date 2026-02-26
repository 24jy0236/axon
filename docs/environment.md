# AXON 環境構築

## ランタイムのインストール
### インストール済みか確認
#### rust
```bash
rustc --version
```
#### node.js
```bash
node --version
```
### 無ければインストール
#### rust
https://win.rustup.rs/x86_64 からインストールする  
か、
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# or
curl https://sh.rustup.rs -sSf | sh
```
Linker(.exeファイル生成)はMicrosoft C++ Build Tools https://aka.ms/vs/17/release/vs_BuildTools.exe  
これを実行し、_C++によるデスクトップ開発_をインストールする
#### node.js
https://nodejs.org/ja/download からインストールする  
か、
```bash
winget install OpenJS.NodeJS.LTS
```

## プロジェクトの作成
#### rust
```bash
# cd axon/
cargo new backend --bin
```
```bash
# cd backend/
cargo add tokio -F full
cargo add axum
cargo add serde -F derive
cargo add serde_json
cargo add tower-http -F "cors"
```
#### node.js
```bash
# cd axon/
npx create-next-app@latest frontend
```

## VSCode拡張機能(必須)
- rust-analizer
- CodeLLDB

## 開発環境で起動
```bash
# cd axon/frontend/
npm run dev
```
```bash
# cd axon/backend/
cargo run
```

---

### postgre docker
#### 起動
```bash
docker compose build --no-cache
docker compose up -d
```

#### 状態確認
```bash
docker compose ps
```

#### 削除
```bash
docker compose down -v
```
(`-v`はボリュームごと消去する！要注意！)

---

### migration実行
```bash
cd backend/
sqlx migrate run
```

#### migrateを一からやりなおすには、dockerをボリュームごと初期化する
```bash
docker compose down -v
docker compose up -d
```

#### DBのスキーマ大掃除
```bash
cd axon/
docker compose down -v
docker compose build --no-cache
docker compose up -d
cd backend/
sqlx migrate run
cargo test export_bindings # TODO: このts型定義エクスポートの順番を検討
```
```bash
docker compose down -v && docker compose build --no-cache && docker compose up -d && cd backend/ && sqlx migrate run && cargo test export_bindings
```

---

### psql
```bash
docker exec -it axon-db psql -U postgres -d axon-db
```

---

### RustのStructからTSへの型定義エクスポート
```bash
cd backend/
cargo test export_bindings
```

---

### 2. 開発環境をセットするまでの手順

```bash
# 1. 開発用環境を起動 (ビルドなしでいきなり立ち上がる！)
docker compose -f docker-compose.dev.yaml up -d

# 2. フロントエンドの依存関係をコンテナ内でインストール
# (初回や package.json を変えた時だけ実行)
docker exec -it axon-frontend-dev npm install

# 3. データベースのマイグレーションと型生成
docker exec -it axon-backend-dev cargo install sqlx-cli --no-default-features --features postgres
docker exec -it axon-backend-dev sqlx migrate run
docker exec -it axon-backend-dev cargo test export_bindings

# 4. ログを監視してホットリロードを楽しむ
docker compose -f docker-compose.dev.yaml logs -f
```