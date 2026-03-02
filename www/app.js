// ═══════════════════════════════════════════════════════════════════
// app.js - WebAssembly 学習アプリのメイン JS
//
// LEARNING NOTE: type="module" が必要な理由
//   wasm-pack が生成する fib_wasm.js は ES module (import/export を使う)。
//   ブラウザでは <script type="module"> からのみ import できる。
//   また、ES module は自動的に Strict Mode で動作する。
// ═══════════════════════════════════════════════════════════════════

// LEARNING NOTE: Wasm モジュールの読み込みは非同期。
//   init() が内部でやること:
//     1. fetch("fib_wasm_bg.wasm") - バイナリを HTTP で取得
//     2. WebAssembly.compile(bytes) - ブラウザの Wasm エンジンでコンパイル
//     3. WebAssembly.instantiate(module, imports) - 実行可能インスタンスを作成
//   これが完了して初めて fibonacci() などの関数が呼べるようになる。
import init, {
  fibonacci,
  fill_fibonacci_sequence,
  get_module_info,
} from './pkg/fib_wasm.js';

// ── JavaScript 版フィボナッチ（公平な比較のため同じアルゴリズム）────
// JS では数値型は実行時に決まる。エンジンは毎回「これは数値か？」を確認する。
function fibonacci_js(n) {
  if (n <= 1) return n;
  return fibonacci_js(n - 1) + fibonacci_js(n - 2);
}

// ── ベンチマーク実行ヘルパー ────────────────────────────────────────
function runJsBenchmark(n) {
  const t0 = performance.now();
  const result = fibonacci_js(n);
  const elapsed = performance.now() - t0;
  return { result, elapsed };
}

function runWasmBenchmark(n) {
  const t0 = performance.now();
  // LEARNING NOTE: fibonacci() は Rust/Wasm から export された関数。
  //   wasm-bindgen が JS Number を Wasm の i32 に変換して渡す。
  //   Wasm は i64 で結果を返すが、i64 は Number の安全整数範囲を超えることがある。
  //   そのため wasm-bindgen は BigInt で返す。Number() で変換して表示する。
  const result = fibonacci(n);
  const elapsed = performance.now() - t0;
  return { result: Number(result), elapsed };
}

// ── Panel 1: ベンチマーク UI ────────────────────────────────────────
function setupBenchmarkPanel() {
  const fibNInput = document.getElementById('fib-n');
  const fibNDisplay = document.getElementById('fib-n-display');

  fibNInput.addEventListener('input', () => {
    fibNDisplay.textContent = fibNInput.value;
  });

  document.getElementById('btn-run-js').addEventListener('click', () => {
    const n = parseInt(fibNInput.value);
    const { result, elapsed } = runJsBenchmark(n);
    document.getElementById('js-result').textContent = result;
    document.getElementById('js-time').textContent = elapsed.toFixed(1) + ' ms';
  });

  document.getElementById('btn-run-wasm').addEventListener('click', () => {
    const n = parseInt(fibNInput.value);
    const { result, elapsed } = runWasmBenchmark(n);
    document.getElementById('wasm-result').textContent = result;
    document.getElementById('wasm-time').textContent = elapsed.toFixed(1) + ' ms';
  });

  document.getElementById('btn-run-both').addEventListener('click', () => {
    const n = parseInt(fibNInput.value);

    // JS を先に実行（遅いため）
    const js = runJsBenchmark(n);
    const wasm = runWasmBenchmark(n);

    document.getElementById('js-result').textContent = js.result;
    document.getElementById('js-time').textContent = js.elapsed.toFixed(1) + ' ms';
    document.getElementById('wasm-result').textContent = wasm.result;
    document.getElementById('wasm-time').textContent = wasm.elapsed.toFixed(1) + ' ms';

    const speedupEl = document.getElementById('speedup');
    if (wasm.elapsed > 0) {
      const speedup = js.elapsed / wasm.elapsed;
      speedupEl.textContent = speedup.toFixed(1);
    } else {
      speedupEl.textContent = '∞';
    }
  });
}

// ── Panel 2: Linear Memory ビューア ─────────────────────────────────
function setupMemoryPanel(wasmModule) {
  document.getElementById('btn-fill').addEventListener('click', () => {
    const len = Math.min(50, Math.max(1, parseInt(document.getElementById('seq-len').value) || 16));

    // LEARNING NOTE: new Uint32Array(len) で作ったバッファを Wasm に渡す。
    //   wasm-bindgen は (pointer, length) ペアを Wasm 側に渡す。
    //   Rust では &mut [u32] として受け取り、同じメモリ領域に直接書き込む。
    //   コピーは一切発生しない。これがゼロコピーの本質。
    const buffer = new Uint32Array(len);
    fill_fibonacci_sequence(buffer);

    // バッファをそのまま読む（Rust が書き込んだ内容がそのまま見える）
    const grid = document.getElementById('memory-display');
    grid.innerHTML = '';
    buffer.forEach((val, idx) => {
      const cell = document.createElement('div');
      cell.className = 'memory-cell';
      cell.innerHTML =
        `<span class="cell-idx">fib(${idx})</span>` +
        `<span class="cell-val">${val.toLocaleString()}</span>`;
      grid.appendChild(cell);
    });

    // LEARNING NOTE: WebAssembly.Memory は実体が SharedArrayBuffer に近い ArrayBuffer。
    //   .buffer.byteLength で Linear Memory の現在のバイト数がわかる。
    //   1 ページ = 65,536 バイト。Wasm モジュールは必要に応じてページを追加できる。
    const memory = wasmModule.memory;
    const byteLen = memory.buffer.byteLength;
    const pages = byteLen / 65536;
    document.getElementById('memory-info').textContent =
      `Wasm Linear Memory: ${byteLen.toLocaleString()} バイト ` +
      `（${pages} ページ × 64 KiB）| ` +
      `バッファは Wasm メモリのアドレス空間内に配置され、JS と Rust が共有しています`;
  });
}

// ── Panel 3: Module Anatomy - セクションカードのクリック展開 ─────────
function setupAnatomyPanel() {
  let expandedCard = null;

  document.querySelectorAll('.section-card').forEach(card => {
    card.addEventListener('click', () => {
      if (expandedCard && expandedCard !== card) {
        expandedCard.classList.remove('expanded');
      }
      card.classList.toggle('expanded');
      expandedCard = card.classList.contains('expanded') ? card : null;
    });
  });
}

// ── メイン: Wasm 初期化と全パネルのセットアップ ──────────────────────
async function main() {
  const statusEl = document.getElementById('module-status');

  try {
    // LEARNING NOTE: init() は wasm-pack が生成した関数。
    //   内部では fetch → compile → instantiate を行い、
    //   WebAssembly.Instance オブジェクト（≈ wasmModule）を返す。
    //   このオブジェクトには .memory（WebAssembly.Memory）が含まれる。
    const wasmModule = await init();

    // Wasm から情報取得（これ自体が import の実証: Rust が console.log を呼ぶ）
    const info = get_module_info();
    statusEl.textContent = '✓ Wasm 準備完了 | ' + info;
    statusEl.className = 'status-bar ready';

    // 各パネルのセットアップ
    setupBenchmarkPanel();
    setupMemoryPanel(wasmModule);
    setupAnatomyPanel();

    // ボタンを有効化
    document.querySelectorAll('button').forEach(btn => {
      btn.disabled = false;
    });

  } catch (err) {
    statusEl.textContent = '✗ Wasm の読み込みに失敗: ' + err.message;
    statusEl.className = 'status-bar error';
    console.error('Wasm 初期化エラー:', err);
    console.error('ヒント: python3 -m http.server 8080 でサーブしていますか？');
    console.error('file:// URL では .wasm ファイルは読み込めません（MIME type 制約）');
  }
}

main();
