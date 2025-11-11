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
