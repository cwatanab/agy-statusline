---
name: statusline-release
description: >-
  agy-statusline のアップストリーム同期、性能最適化、テスト検証、および GitHub Releases 発行（各OS向けバイナリビルド＆デプロイ）の一連の手順を実行するスキル。
---

# statusline-release

`agy-statusline` の新バージョン取り込み、最適化、テスト検証、および GitHub Releases 発行の標準ワークフロー。

## 概要フロー

1. **アップストリーム更新の調査と取り込み**
2. **コードの最適化と性能検証（ゼロアロケーション指向）**
3. **ローカルテスト & ベンチマーク実行**
4. **Git コミット & プッシュ & タギング**
5. **GitHub Release 作成 & CI（各OS向けバイナリ）追跡**
6. **リリースノートの調整**

---

## 詳細手順

### 1. アップストリームの差分調査と取り込み

1. リモート設定の確認:
   ```bash
   git remote -v
   # upstream: https://github.com/weby-homelab/antigravity-cli-statusline.git
   ```
2. 対象リリースタグのリリースノートおよびコミット差分の取得:
   ```bash
   gh release view <TAG> --repo weby-homelab/antigravity-cli-statusline
   git fetch upstream --tags
   git log HEAD..upstream/<BRANCH> --oneline
   ```
3. 変更箇所のコードポーティング:
   - バージョン番号 (`Cargo.toml`, `src/main.rs`, `README.md`, `CHANGELOG.md`)
   - 構造体およびパーサー拡張 (`src/parse.rs`)
   - レンダラー調整・表示改善 (`src/render.rs`, `src/format.rs`)
   - テストの追従・追加 (`tests/*.rs`)

---

### 2. パフォーマンス最適化 (Rust / Zero-Dependency)

`agy-statusline` は毎フレーム呼び出されるため、**ゼロ依存・低アロケーション・高速性** を徹底する:

- **パース最適化 (`src/parse.rs`)**:
  - `read_u64` / `read_u32` / `read_i64` などの数値解析は文字列変換や `parse()` を介さず、ASCII バイト直接走査（`b - b'0'`）でゼロアロケーション化。
- **レンダリング最適化 (`src/render.rs`)**:
  - セグメント描画（Powerline 区切り色など）で `String::replace` のような動的ヒープ確保を避け、バッファ直書きや文字列スライスのプレフィックス判定を活用。
- **ベンチマークによる効果測定**:
  ```bash
  cargo test --release --test perf_benchmark -- --nocapture
  ```
  - 10,000回イテレーションでのマイクロ秒/回を確認し、性能向上を検証する。

---

### 3. テストとリリースビルド確認

```bash
# 全テスト実行
cargo test

# リリースバイナリビルド
cargo build --release
```

---

### 4. Git コミット・プッシュ・タギング

1. 作業内容をコミット:
   ```bash
   git add -A
   git commit -m "chore: release <TAG>"
   git push origin main
   ```
2. リリースタグの作成とプッシュ:
   - 既存のローカルタグが古い参照を指している場合は一度削除して再作成:
   ```bash
   git tag -d <TAG> 2>/dev/null || true
   git tag -a <TAG> -m "Release <TAG>"
   git push origin <TAG>
   ```

---

### 5. GitHub Release 発行 & 各OS向けバイナリ（CI）自動デプロイ

1. リリースを作成:
   ```bash
   gh release create <TAG> --repo cwatanab/agy-statusline --title "<TAG>" --notes "オリジナルの <TAG> を反映"
   ```
2. タグプッシュを契機に GitHub Actions (`.github/workflows/release.yml`) が起動し、以下の5種類のバイナリがビルドされて Release に自動アップロードされる:
   - `statusline-linux-x86_64`
   - `statusline-linux-arm64`
   - `statusline-macos-x86_64`
   - `statusline-macos-arm64`
   - `statusline-windows-x86_64.exe`
3. ワークフローの完了確認:
   ```bash
   gh run list --repo cwatanab/agy-statusline --limit 1
   gh run watch <RUN_ID> --repo cwatanab/agy-statusline
   ```
4. リリースアセットの確認:
   ```bash
   gh release view <TAG> --repo cwatanab/agy-statusline
   ```

---

### 6. リリースノートの調整

- リリースノートを簡潔にする場合（例: 「オリジナルの vX.Y.Z を反映」）:
  ```bash
  gh release edit <TAG> --repo cwatanab/agy-statusline --notes "オリジナルの <TAG> を反映"
  ```
