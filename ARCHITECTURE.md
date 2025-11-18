# windows-drivers-rs アーキテクチャ

## 概要

windows-drivers-rsは、RustでWindowsカーネルモードおよびユーザーモードドライバーを開発するための包括的なクレートコレクションです。このプロジェクトは、WDM（Windows Driver Model）、KMDF（Kernel-Mode Driver Framework）、UMDF（User-Mode Driver Framework）の3つの主要なWindowsドライバーモデルをサポートしています。

### プロジェクト情報

- **リポジトリ**: https://github.com/microsoft/windows-drivers-rs
- **ライセンス**: MIT OR Apache-2.0
- **Rust Edition**: 2024
- **MSRV (最小サポートRustバージョン)**: 1.85.0

---

## ワークスペース構成

このプロジェクトは、以下の7つのクレートで構成されるCargoワークスペースです：

```
crates/
├── cargo-wdk/         # Cargo拡張コマンドラインツール
├── wdk/               # 安全な高レベルAPI
├── wdk-sys/           # WDK FFIバインディング
├── wdk-build/         # ビルドスクリプトユーティリティ
├── wdk-macros/        # プロシージャルマクロ
├── wdk-alloc/         # グローバルアロケータ実装
└── wdk-panic/         # パニックハンドラ実装
```

---

## クレート詳細

### 1. wdk-sys (v0.5.1)

**役割**: WDK（Windows Driver Kit）APIへの直接的なFFI（Foreign Function Interface）バインディングを提供する基盤レイヤー。

#### 主要機能

- **bindgenによる自動生成**: WDKヘッダーファイルから自動生成されたRust FFIバインディング
- **マニュアルバインディング**: bindgenで生成できないマクロやインラインヘルパーの手動実装
- **WDF関数テーブル**: Windows Driver Foundation関数へのアクセス機構
- **NTSTATUS処理**: カーネルモード戻り値の型安全な処理

#### モジュール構成

- `ntddk`: WDM/KMDFカーネルモードAPI
- `wdf`: Windows Driver FrameworkのコアAPI
- `windows`: UMDF用Windows API
- デバイス固有モジュール（feature-gated）:
  - `gpio`: GPIOデバイスAPI
  - `hid`: HID（Human Interface Device）API
  - `parallel_ports`: パラレルポートAPI
  - `spb`: SPB（Simple Peripheral Bus）API
  - `storage`: ストレージデバイスAPI
  - `usb`: USBデバイスAPI

#### 依存関係

**ランタイム依存**:
- `wdk-macros` (=0.5.1) - ロックステップバージョニング
- `rustversion` - Rustバージョンに応じた条件付きコンパイル

**ビルド依存**:
- `bindgen` - C/C++バインディング自動生成
- `cc` - Cコンパイラ統合
- `wdk-build` - WDK設定検出
- `cargo_metadata` - Cargoメタデータ解析

#### 特徴

- `#![no_std]` クレート（カーネルモード環境対応）
- ドライバータイプに応じた条件付きコンパイル（WDM/KMDF/UMDF）
- Cargoフィーチャーによる追加APIサポート
- 安全でないFFI関数の明確な`unsafe`マーキング

#### 公開API例

```rust
// NTSTATUS処理
pub fn NT_SUCCESS(status: NTSTATUS) -> bool;
pub fn NT_ERROR(status: NTSTATUS) -> bool;

// WDF関数呼び出し
call_unsafe_wdf_function_binding!(
    WdfDriverCreate,
    driver_globals,
    driver_object,
    // ...
);

// ページングコードマーカー
PAGED_CODE!();
```

---

### 2. wdk (v0.4.1)

**役割**: `wdk-sys`の上に構築された、安全でイディオマティックなRustラッパーを提供する高レベルAPI。

#### 主要機能

- **安全な抽象化**: `wdk-sys`のunsafe関数に対する型安全なラッパー
- **WDF統合**: KMDF/UMDF用の高レベルAPI
- **診断マクロ**: カーネル/ユーザーモードでの出力サポート（`alloc`フィーチャー有効時）

#### モジュール構成

- `wdf::spinlock`: スピンロックの安全な抽象化
- `wdf::timer`: タイマーオブジェクトの安全な抽象化
- `print`: `println!`/`print!`マクロのカーネルモード実装

#### 依存関係

- `wdk-sys` (workspace) - FFI基盤
- `cfg-if` - 条件付きコンパイルユーティリティ

#### 特徴

- **ドライバーモード依存の`std`サポート**:
  - WDM/KMDF: `#![no_std]`
  - UMDF: 標準ライブラリ使用可能
- デフォルトで`alloc`フィーチャー有効（動的メモリ割り当てサポート）
- メモリ安全性とスレッド安全性の保証

#### 公開API例

```rust
// wdk-sysからの再エクスポート
pub use wdk_sys::{nt_success, paged_code};

// アーキテクチャ固有のブレークポイント
pub fn dbg_break();

// WDFスピンロックの安全な使用
use wdk::wdf::spinlock::SpinLock;
```

---

### 3. wdk-build (v0.5.1)

**役割**: WDK依存のビルドをCargoビルドスクリプト（`build.rs`）内で設定するためのライブラリ。

#### 主要機能

- **WDK環境検出**: WDKインストールパスとバージョンの自動検出
- **bindgen設定**: WDKヘッダー用のbindgen設定構築
- **リンカー設定**: ドライバータイプ別のリンカーフラグ生成
- **cargo-make統合**: ドライバーパッケージング用のMakefileタスク提供
- **メタデータ解析**: `Cargo.toml`の`[package.metadata.wdk]`セクション解析

#### 主要型

```rust
pub struct Config {
    // WDKビルド設定
}

pub enum DriverConfig {
    Wdm(),
    Kmdf(KmdfConfig),
    Umdf(UmdfConfig),
}

pub struct KmdfConfig {
    pub kmdf_version_major: u8,
    pub target_kmdf_version_minor: u8,
    // ...
}

pub enum CpuArchitecture {
    AMD64,
    ARM64,
}

pub enum ApiSubset {
    Base, Wdf, Gpio, Hid, ParallelPorts,
    Spb, Storage, Usb
}
```

#### 公開関数

```rust
// バイナリ（.sysドライバー）のビルド設定
pub fn configure_wdk_binary_build();

// ライブラリクレートのビルド設定
pub fn configure_wdk_library_build(
    exported_apis: impl IntoIterator<Item = ApiSubset>
);

// カスタム処理付きライブラリビルド
pub fn configure_wdk_library_build_and_then(
    exported_apis: impl IntoIterator<Item = ApiSubset>,
    configure_callback: impl FnOnce(Config)
);

// WDKビルド番号の検出
pub fn detect_wdk_build_number() -> Result<u32>;
```

#### 特徴

- WDK環境変数の検出（`WDKContentRoot`など）
- cargo-makeマクロの提供（`rust-driver-makefile.toml`）
- WDKライブラリパスの自動解決
- プリプロセッサ定義の自動注入

#### 使用例（`build.rs`）

```rust
use wdk_build::{configure_wdk_binary_build, ApiSubset};

fn main() {
    // ドライバーバイナリのビルド設定
    configure_wdk_binary_build();

    // または、ライブラリのビルド設定
    // configure_wdk_library_build([
    //     ApiSubset::Base,
    //     ApiSubset::Usb,
    // ]);
}
```

---

### 4. wdk-macros (v0.5.1)

**役割**: `wdk-sys`のFFIバインディングとの対話を容易にするプロシージャルマクロコレクション。

#### 主要機能

- **WDF関数呼び出しマクロ**: 型安全なWDF関数ディスパッチ
- **関数テーブル情報のキャッシング**: コンパイル高速化のためのメタデータキャッシュ
- **型情報の検証**: コンパイル時の型チェック

#### プロシージャルマクロ

```rust
// WDF関数を名前で呼び出す
call_unsafe_wdf_function_binding!(
    function_name,
    arg1,
    arg2,
    // ...
)
```

#### 依存関係

- `syn`, `quote`, `proc-macro2` - プロシージャルマクロ基盤
- `serde`, `serde_json` - メタデータシリアライゼーション
- `itertools` - イテレータユーティリティ
- `fs4` - ファイルシステムロック
- `scratch` - キャッシュディレクトリ管理

#### 特徴

- `proc-macro` クレート（`[lib] proc-macro = true`）
- **ロックステップバージョニング**: `wdk-sys`と厳密に同期（=0.5.1）
- 直接依存を避け、`wdk-sys`を通じて再エクスポート
- 関数シグネチャ情報のファイルキャッシュによるビルド高速化

#### 内部動作

1. WDF関数名を解析
2. scratchディレクトリから関数シグネチャ情報を読み込み
3. 関数テーブルインデックスを使用した実行時ディスパッチコードを生成
4. 型安全性を検証

---

### 5. wdk-alloc (v0.4.1)

**役割**: WDKカーネルモードバイナリ用のグローバルアロケータ実装。

#### 主要機能

- `#[global_allocator]`で使用可能なアロケータ
- WDKメモリプールAPI（`ExAllocatePool2`/`ExFreePool`）の使用
- 非ページプールからの割り当て

#### 公開型

```rust
pub struct WdkAllocator;

unsafe impl GlobalAlloc for WdkAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
}
```

#### 使用例

```rust
#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;
```

#### 依存関係

- `wdk-sys` - WDKメモリ管理API

#### 特徴

- `#![no_std]` クレート
- **WDM/KMDFモードでのみ有効**（UMDFは標準アロケータ使用）
- **IRQL制約**: `IRQL <= DISPATCH_LEVEL`でのみ安全
- **メモリプール**: `POOL_FLAG_NON_PAGED`（非ページプール）から割り当て
- **メモリタグ**: `"rust"`（リトルエンディアンで`0x74737572`）

#### 制約と注意事項

- 高いIRQLレベル（`DISPATCH_LEVEL`以上）では使用不可
- ページングされない非ページプールを使用（メモリ効率に注意）
- 将来的にはIRQL対応の改善予定

---

### 6. wdk-panic (v0.4.1)

**役割**: WDKでビルドされたプログラム用のデフォルトパニックハンドラ実装。

#### 主要機能

- シンプルな無限ループパニックハンドラ
- デバッグ/リリースビルドで一貫した動作

#### 実装

```rust
#[cfg(not(test))]
#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}
```

#### 依存関係

なし（完全に独立）

#### 特徴

- `#![no_std]` クレート
- 最小限の依存関係（完全に自己完結）
- テスト時には無効化（`#[cfg(not(test))]`）
- **将来の改善計画**: `KeBugCheckEx`によるバグチェック実装（コメント参照）

#### 使用方法

```rust
#[cfg(not(test))]
extern crate wdk_panic;
```

単に依存関係に追加するだけで、パニックハンドラが自動的に登録されます。

---

### 7. cargo-wdk (v0.1.1)

**役割**: Rustでのドライバー開発ワークフローを自動化するCargo拡張コマンドラインツール。

#### 主要機能

##### `new`コマンド - プロジェクト生成

```bash
cargo wdk new <PATH> --kmdf|--umdf|--wdm
```

テンプレートから新しいドライバープロジェクトを作成します。

**オプション**:
- `--kmdf`: Kernel-Mode Driver Frameworkプロジェクト
- `--umdf`: User-Mode Driver Frameworkプロジェクト
- `--wdm`: Windows Driver Modelプロジェクト

##### `build`コマンド - ビルドとパッケージング

```bash
cargo wdk build [OPTIONS]
```

ドライバーのコンパイル、署名、パッケージングを自動実行します。

**オプション**:
- `--profile <PROFILE>`: ビルドプロファイル（dev/release）
- `--target-arch <ARCH>`: ターゲットアーキテクチャ（x64/arm64）
- `--verify-signature`: 署名検証の実行

**実行される処理**:
1. Rustコードのコンパイル（`.sys`ファイル生成）
2. `stampinf`: INFファイルへのバージョン情報埋め込み
3. `inf2cat`: カタログファイル（`.cat`）生成
4. `signtool`: デジタル署名（WDRLocalTestCert使用）

#### 依存関係

- `wdk-build` - WDK設定とビルドロジック
- `clap` - コマンドライン引数解析
- `cargo_metadata` - Cargoワークスペース情報
- `anyhow`, `thiserror` - エラーハンドリング
- `tracing`, `tracing-subscriber` - ログとトレース

#### 特徴

- **ワークスペース対応**: モノレポ構成でも動作
- **自動署名**: テスト用自己署名証明書（WDRLocalTestCert）の自動生成と使用
- **パッケージ生成**: `.sys`, `.inf`, `.cat`, `.cer`ファイルを含む完全なドライバーパッケージ
- **テンプレートシステム**: `templates/`ディレクトリからプロジェクトテンプレートを提供

#### 出力構造

```
target/
└── <arch>-pc-windows-msvc/
    └── <profile>/
        ├── driver_name.sys      # ドライバーバイナリ
        ├── driver_name.inf      # インストール情報
        ├── driver_name.cat      # カタログファイル
        └── WDRLocalTestCert.cer # テスト証明書
```

---

## クレートの分類

### ツール層
- **cargo-wdk**: 開発ワークフロー自動化CLIツール

### ビルドツール層
- **wdk-build**: ビルドスクリプトライブラリ、cargo-make統合

### API層
- **wdk**: 安全な高レベルAPI（推奨使用）
- **wdk-sys**: FFI低レベルバインディング（高度な用途）

### ランタイムサポート層
- **wdk-alloc**: メモリアロケータ
- **wdk-panic**: パニックハンドラ

### メタプログラミング層
- **wdk-macros**: プロシージャルマクロ

---

## クレート間の依存関係

```
┌─────────────────┐
│   cargo-wdk     │ (CLI tool)
│  (開発ツール)   │
└────────┬────────┘
         │
         ▼
    ┌─────────┐
    │wdk-build│
    └────┬────┘
         │
         ▼ (build dependency)
    ┌─────────┐           ┌────────────┐
    │   wdk   │◄──────────│  wdk-alloc │
    │(安全API)│           └─────┬──────┘
    └────┬────┘                 │
         │                      │
         ▼                      ▼
    ┌─────────┐           ┌─────────┐
    │ wdk-sys │◄──────────│wdk-panic│
    │  (FFI)  │           │(独立)    │
    └────┬────┘           └─────────┘
         │
         ▼ (lockstep =0.5.1)
    ┌──────────┐
    │wdk-macros│
    └──────────┘
```

### 依存関係の重要なポイント

1. **ロックステップバージョニング**:
   - `wdk-sys` ⇔ `wdk-macros`: 厳密な同期（=0.5.1）
   - FFIバインディングとマクロの整合性を保証

2. **最小依存原則**:
   - `wdk-panic`: 依存なし（完全自己完結）
   - `wdk-alloc`: `wdk-sys`のみに依存

3. **レイヤー分離**:
   - 上位レイヤーは下位レイヤーにのみ依存
   - 循環依存なし

4. **ビルド時依存**:
   - ほぼ全クレートが`wdk-build`をビルド依存として使用
   - 一貫したビルド設定の共有

---

## アーキテクチャ設計原則

### 1. レイヤードアーキテクチャ

```
┌──────────────────────────────────────┐
│  アプリケーション層                  │
│  (cargo-wdk: 開発ツール)            │
└──────────────────────────────────────┘
              ▼
┌──────────────────────────────────────┐
│  抽象化層                            │
│  (wdk: 安全でイディオマティックなAPI)│
└──────────────────────────────────────┘
              ▼
┌──────────────────────────────────────┐
│  FFI層                               │
│  (wdk-sys: 直接バインディング)      │
│  (wdk-macros: メタプログラミング)   │
└──────────────────────────────────────┘
              ▼
┌──────────────────────────────────────┐
│  ビルド層                            │
│  (wdk-build: 設定・リンカー設定)    │
└──────────────────────────────────────┘
              ▼
┌──────────────────────────────────────┐
│  ランタイム層                        │
│  (wdk-alloc, wdk-panic)             │
└──────────────────────────────────────┘
```

### 2. `no_std`優先設計

カーネルモード環境（WDM/KMDF）では標準ライブラリが利用できないため：

- **カーネルモードクレート**: `#![no_std]`属性を使用
- **標準機能の最小化**: 標準ライブラリへの依存を排除
- **`alloc`クレート**: `wdk-alloc`により動的メモリ割り当てを任意で有効化
- **UMDF例外**: ユーザーモードドライバーは標準ライブラリ使用可能

### 3. 条件付きコンパイル

異なるドライバーモデルとプラットフォームをサポートするため：

```rust
// ドライバータイプによる分岐
#[cfg(feature = "wdk-sys/wdm")]
// WDM固有コード

#[cfg(feature = "wdk-sys/kmdf")]
// KMDF固有コード

#[cfg(feature = "wdk-sys/umdf")]
// UMDF固有コード

// 機能別API
#[cfg(feature = "wdk-sys/usb")]
pub mod usb;

#[cfg(feature = "wdk-sys/gpio")]
pub mod gpio;
```

### 4. 型安全性の追求

Rustの型システムを最大限活用：

- **`wdk-macros`**: WDF関数呼び出しの型チェック
- **`wdk`の抽象化**: 型安全なラッパーによるメモリ安全性保証
- **明示的な`unsafe`**: 危険な操作の可視化と文書化義務
- **ライフタイム管理**: リソースの適切なライフタイム追跡

### 5. バージョン管理戦略

```toml
# ロックステップバージョニング（厳密同期）
wdk-macros = "=0.5.1"  # wdk-sysと完全に同期

# セマンティックバージョニング
wdk = "0.4.1"
wdk-alloc = "0.4.1"
wdk-panic = "0.4.1"
```

**ロックステップの理由**:
- FFIバインディング（`wdk-sys`）とマクロ（`wdk-macros`）の整合性が必須
- 関数シグネチャの変更にマクロが即座に対応する必要がある

### 6. ビルドシステム統合

#### cargo-make統合

```toml
# rust-driver-makefile.toml
[tasks.driver-build]
description = "ドライバーのビルド"
command = "cargo"
args = ["build", "--release"]

[tasks.driver-package]
dependencies = ["driver-build"]
# INF処理、署名など
```

#### WDK環境検出

```rust
// 自動検出される環境変数
WDKContentRoot
WindowsSdkDir
WindowsSDKVersion
```

#### bindgen自動化

```rust
// build.rsでの使用
use wdk_build::Config;

let config = Config::from_env_auto()?;
config.configure_bindings_builder(bindgen::Builder::default())
    .generate()?
    .write_to_file(out_path)?;
```

### 7. 開発体験の最適化

#### プロジェクト開始の簡素化

```bash
# 1コマンドでプロジェクト作成
cargo wdk new my_driver --kmdf

# テンプレートに含まれるもの：
# - Cargo.toml (メタデータ設定済み)
# - src/lib.rs (エントリーポイント)
# - build.rs (WDK設定)
# - driver.inf (INFテンプレート)
```

#### 詳細なエラーメッセージ

```rust
// tracingによる詳細ログ
tracing::info!("WDK detected at: {}", wdk_root);
tracing::warn!("KMDF version {} not found, using {}",
               requested, available);
```

### 8. テスト可能性

#### WDF関数のスタブ化

```toml
[features]
test-stubs = []  # テスト用のWDF関数スタブを有効化
```

#### 条件付きテストコード

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator() {
        // テスト用アロケータは標準アロケータを使用
    }
}
```

#### 独立したテスト

各クレートは独立してテスト可能：

```bash
# 個別クレートのテスト
cd crates/wdk-sys && cargo test
cd crates/wdk && cargo test

# ワークスペース全体のテスト
cargo test --workspace
```

---

## 技術的特徴

### bindgen統合

#### カスタムbindgen設定

```rust
// wdk-build::BuilderExt トレイトによる拡張
trait BuilderExt {
    fn configure_for_wdk(self, config: &Config) -> Self;
}

impl BuilderExt for bindgen::Builder {
    fn configure_for_wdk(self, config: &Config) -> Self {
        self
            .clang_args(&config.get_include_paths())
            .clang_args(&config.get_preprocessor_definitions())
            // ...
    }
}
```

#### 自動ヘッダー解析

- WDKヘッダーファイルの自動検出
- プリプロセッサ定義の自動注入（`_AMD64_`, `_WIN64`など）
- 型エイリアスの自動生成

### WDF関数ディスパッチメカニズム

#### マクロ展開

```rust
// 使用例
call_unsafe_wdf_function_binding!(
    WdfDriverCreate,
    driver_globals,
    driver_object,
    registry_path,
    driver_attributes,
    driver_config,
    &mut driver_handle
);

// 展開後（簡略版）
unsafe {
    let func_table = (*driver_globals).func_table;
    let func = (*func_table).pfnWdfDriverCreate;
    (func)(
        driver_globals,
        driver_object,
        registry_path,
        driver_attributes,
        driver_config,
        driver_handle
    )
}
```

#### 関数テーブルインデックス

- 実行時に関数テーブルから関数ポインタを取得
- バージョン互換性を動的に処理
- コンパイル時型チェックによる安全性確保

### メタデータ駆動設定

#### Cargo.tomlでの設定

```toml
[package.metadata.wdk.driver-model]
driver-type = "KMDF"
kmdf-version-major = 1
target-kmdf-version-minor = 33

[package.metadata.wdk.infverif]
version = "latest"
```

#### ビルド時解析

```rust
// wdk-buildが自動的に解析
let metadata = cargo_metadata::MetadataCommand::new()
    .exec()?;
let driver_config = metadata
    .get_package_metadata::<WdkMetadata>("wdk")?
    .driver_model;
```

### リンカー設定自動化

#### ドライバータイプ別フラグ

```rust
// WDMドライバー
println!("cargo:rustc-link-arg=/SUBSYSTEM:NATIVE");
println!("cargo:rustc-link-arg=/DRIVER");
println!("cargo:rustc-link-arg=/ENTRY:DriverEntry");

// KMDFドライバー
println!("cargo:rustc-link-arg=/SUBSYSTEM:NATIVE");
println!("cargo:rustc-link-arg=/DRIVER:WDF");
println!("cargo:rustc-link-arg=/ENTRY:FxDriverEntry");
```

#### WDKライブラリリンク

```rust
// 自動的にリンクされるライブラリ
println!("cargo:rustc-link-lib=ntoskrnl");
println!("cargo:rustc-link-lib=hal");
println!("cargo:rustc-link-lib=wdfldr");
println!("cargo:rustc-link-lib=BufferOverflowFastFailK");
```

---

## ドライバーモデル別の特性

### WDM (Windows Driver Model)

**特徴**:
- 最も低レベルなAPI
- 完全な制御と柔軟性
- 複雑で実装が困難

**使用クレート**:
```toml
[dependencies]
wdk-sys = { version = "0.5.1", features = ["wdm"] }
wdk = { version = "0.4.1", default-features = false }
wdk-alloc = "0.4.1"
wdk-panic = "0.4.1"
```

**メタデータ**:
```toml
[package.metadata.wdk.driver-model]
driver-type = "WDM"
```

### KMDF (Kernel-Mode Driver Framework)

**特徴**:
- オブジェクト指向フレームワーク
- WDMより高レベルで使いやすい
- 自動的なエラーハンドリングと電源管理

**使用クレート**:
```toml
[dependencies]
wdk-sys = { version = "0.5.1", features = ["kmdf-1.33"] }
wdk = "0.4.1"
wdk-alloc = "0.4.1"
wdk-panic = "0.4.1"
```

**メタデータ**:
```toml
[package.metadata.wdk.driver-model]
driver-type = "KMDF"
kmdf-version-major = 1
target-kmdf-version-minor = 33
```

### UMDF (User-Mode Driver Framework)

**特徴**:
- ユーザーモードで実行
- システムクラッシュのリスクが低い
- デバッグが容易

**使用クレート**:
```toml
[dependencies]
wdk-sys = { version = "0.5.1", features = ["umdf-2.33"] }
wdk = "0.4.1"  # 標準ライブラリ使用可能
```

**メタデータ**:
```toml
[package.metadata.wdk.driver-model]
driver-type = "UMDF"
umdf-version-major = 2
target-umdf-version-minor = 33
```

---

## 使用方法

### 新規プロジェクトの作成

```bash
# KMDFドライバーの作成
cargo wdk new my_kmdf_driver --kmdf

# UMDFドライバーの作成
cargo wdk new my_umdf_driver --umdf

# WDMドライバーの作成
cargo wdk new my_wdm_driver --wdm
```

### ビルド

```bash
# 開発ビルド
cargo wdk build

# リリースビルド
cargo wdk build --profile release

# 特定アーキテクチャ向けビルド
cargo wdk build --target-arch x64
cargo wdk build --target-arch arm64

# 署名検証付きビルド
cargo wdk build --verify-signature
```

### 典型的なプロジェクト構造

```
my_driver/
├── Cargo.toml           # パッケージ設定とメタデータ
├── build.rs             # ビルドスクリプト
├── src/
│   └── lib.rs          # ドライバーエントリーポイント
├── driver.inf           # インストール情報ファイル
└── target/
    └── x86_64-pc-windows-msvc/
        └── release/
            ├── my_driver.sys
            ├── my_driver.inf
            ├── my_driver.cat
            └── WDRLocalTestCert.cer
```

### エントリーポイントの実装例（KMDF）

```rust
#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

use wdk_sys::{
    DRIVER_OBJECT,
    NTSTATUS,
    PCUNICODE_STRING,
    PWDF_DRIVER_CONFIG,
    STATUS_SUCCESS,
    call_unsafe_wdf_function_binding,
};

#[export_name = "DriverEntry"]
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    let mut driver_config = core::mem::zeroed::<PWDF_DRIVER_CONFIG>();
    let mut driver_handle = core::ptr::null_mut();

    call_unsafe_wdf_function_binding!(
        WdfDriverCreate,
        driver as *mut _ as *mut _,
        registry_path,
        core::ptr::null_mut(),
        &mut driver_config,
        &mut driver_handle
    );

    STATUS_SUCCESS
}
```

---

## まとめ

windows-drivers-rsプロジェクトは、7つの緊密に統合されたクレートで構成され、RustでのWindowsドライバー開発を実現する包括的なエコシステムです。

### 設計の核心

1. **安全性**: FFI層の上に型安全な抽象化を構築
2. **柔軟性**: 3つのドライバーモデル（WDM/KMDF/UMDF）をサポート
3. **モジュール性**: 明確な責任分離と最小依存原則
4. **使いやすさ**: cargo-wdkによる開発ワークフローの自動化
5. **保守性**: ロックステップバージョニングと厳格な型チェック

### アーキテクチャの利点

- **Rustの安全性保証をカーネルモードに**: メモリ安全性とスレッド安全性
- **既存WDKツールチェーンとのシームレス統合**: INF処理、署名、パッケージング
- **段階的な学習曲線**: 高レベルAPI（`wdk`）から低レベルFFI（`wdk-sys`）まで
- **テスト容易性**: 各レイヤーを独立してテスト可能

### 今後の拡張性

このアーキテクチャは以下の拡張に対応可能です：

- 新しいデバイスカテゴリのサポート（Bluetooth、ネットワークなど）
- より高度な安全抽象化（RAII、型状態パターン）
- 追加のビルドツール統合
- パフォーマンス最適化とメモリ効率の改善

---

## 参考情報

- **リポジトリ**: https://github.com/microsoft/windows-drivers-rs
- **Windows Driver Kit (WDK)**: https://docs.microsoft.com/windows-hardware/drivers/
- **Rust FFI**: https://doc.rust-lang.org/nomicon/ffi.html
- **bindgen**: https://rust-lang.github.io/rust-bindgen/
