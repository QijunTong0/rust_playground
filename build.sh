#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# build.sh - Rust → WebAssembly ビルドスクリプト
#
# このスクリプト自体が学習教材です。各ステップのコメントを読んでください。
# ═══════════════════════════════════════════════════════════════════
set -e

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║   Rust + WebAssembly 学習アプリ ビルドスクリプト      ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

# ───────────────────────────────────────────────────────────────────
# STEP 1: wasm-pack のインストール確認
#
# wasm-pack は Rust → WebAssembly のビルドを自動化するツールです。
# 内部でやること:
#   1. cargo build --target wasm32-unknown-unknown --release
#   2. wasm-bindgen CLI で JS グルーコードを生成
#   3. (オプション) wasm-opt でバイナリを最適化
# ───────────────────────────────────────────────────────────────────
echo "STEP 1: wasm-pack を確認..."
if ! command -v wasm-pack &>/dev/null; then
  echo "  wasm-pack が見つかりません。インストールします..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  echo "  ✓ wasm-pack インストール完了"
else
  echo "  ✓ wasm-pack: $(wasm-pack --version)"
fi

# ───────────────────────────────────────────────────────────────────
# STEP 2: wasm32-unknown-unknown ターゲットの追加
#
# Rust のクロスコンパイルには「ターゲットトリプル」が必要です。
#   wasm32-unknown-unknown の意味:
#     wasm32       = 32bit WebAssembly アーキテクチャ
#     unknown      = OS なし（ブラウザの VM 上で動く）
#     unknown      = ABI なし（libc を使わない）
#
# これにより:
#   - std::fs, std::net など OS 依存の機能は使えない
#   - JS ホストから提供された機能だけを使う（= imports）
#   - バイナリサイズが小さくなる
# ───────────────────────────────────────────────────────────────────
echo ""
echo "STEP 2: wasm32 コンパイルターゲットを追加..."
rustup target add wasm32-unknown-unknown 2>/dev/null || true
echo "  ✓ wasm32-unknown-unknown ターゲット: 準備完了"

# ───────────────────────────────────────────────────────────────────
# STEP 3: Rust → WebAssembly コンパイル
#
# wasm-pack build オプションの説明:
#   --target web
#     ES module 形式で出力（import/export を使う）。
#     バンドラー（Webpack/Vite）不要で直接ブラウザで動く。
#     --target bundler にすると Node.js/バンドラー向けになる。
#
#   --release
#     最適化ビルド。opt-level="s" が Cargo.toml で設定済み。
#     デバッグビルドは .wasm が数倍大きくなり遅い。
#
#   --out-dir ../www/pkg
#     生成ファイルの出力先。www/ から直接 import できるよう配置。
# ───────────────────────────────────────────────────────────────────
echo ""
echo "STEP 3: Rust を WebAssembly にコンパイル..."
cd "$(dirname "$0")/wasm"
wasm-pack build --target web --release --out-dir ../www/pkg
cd ..
echo "  ✓ コンパイル完了"

# ───────────────────────────────────────────────────────────────────
# STEP 4: 生成ファイルの確認（学習ポイント）
#
# wasm-pack が生成するファイル:
#   fib_wasm_bg.wasm  - 実際の WebAssembly バイナリ
#   fib_wasm.js       - wasm-bindgen の JS グルーコード（必読！）
#   fib_wasm.d.ts     - TypeScript 型定義（export 一覧がわかる）
#   package.json      - npm パッケージメタデータ
# ───────────────────────────────────────────────────────────────────
echo ""
echo "STEP 4: 生成ファイルの確認"
echo ""
echo "  ファイル一覧:"
ls -lh www/pkg/ | grep -v "^total" | awk '{printf "    %-40s %s\n", $9, $5}'
echo ""

WASM_SIZE=$(stat -f%z www/pkg/fib_wasm_bg.wasm 2>/dev/null || stat -c%s www/pkg/fib_wasm_bg.wasm 2>/dev/null || echo "?")
echo "  Wasm バイナリサイズ: ${WASM_SIZE} バイト"
echo ""
echo "  ─────────────────────────────────────────────────────"
echo "  [学習推奨] 以下のファイルを読んでみましょう:"
echo ""
echo "  1. www/pkg/fib_wasm.js"
echo "     → wasm-bindgen が生成した JS グルーコード"
echo "       型変換（i32 ↔ Number, i64 ↔ BigInt, &str → JS String）の"
echo "       実装が読める。このファイルを理解すれば Wasm interop が分かる。"
echo ""
echo "  2. www/pkg/fib_wasm.d.ts"
echo "     → TypeScript 型定義。export された関数の型シグネチャ一覧。"
echo ""
echo "  3. WAT テキスト形式（要 wabt）:"
echo "     sudo apt install wabt"
echo "     wasm2wat www/pkg/fib_wasm_bg.wasm | less"
echo "     → .wasm バイナリの人間が読める形式。Panel 3 のカードと照合できる。"
echo "  ─────────────────────────────────────────────────────"
echo ""

# ───────────────────────────────────────────────────────────────────
# STEP 5: HTTP サーバーの起動
#
# なぜ file:// URL で開けないのか？
#   ブラウザは .wasm ファイルを application/wasm MIME type として
#   サーブするサーバーからのみ受け付ける（セキュリティ制約）。
#   file:// URL からは CORS エラーで fetch() が失敗する。
#   → 必ず HTTP サーバー経由でアクセスすること。
# ───────────────────────────────────────────────────────────────────
echo "STEP 5: HTTP サーバーを起動します"
echo ""
echo "  ブラウザで以下を開いてください:"
echo "  → http://localhost:8080"
echo ""
echo "  終了するには Ctrl+C を押してください。"
echo ""
python3 -m http.server 8080 --directory www
