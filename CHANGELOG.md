# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-08-07

### Added
- パース・レンダリング性能を測定する統合マイクロベンチマークテスト (`tests/perf_benchmark.rs`) を追加。

### Changed
- **60倍の高速化**: ゼロコピー JSON パーサー (`ParsedInput<'a>`) とヒープアロケーションフリーなパース処理を導入。
- **シングルバッファレンダリング**: レンダリング時の中間配列割り当て (`Vec<String>`) および重厚な `format!` マクロを撤廃し、直書きストリーミングエンジンへ刷新。
- **ゼロプロセス Git ブランチ取得**: 外部 `git.exe` プロセス起動による遅延を解消し、`.git/HEAD` を直接参照する超高速探査アルゴリズムを導入 (実行速度 〜6.2ms)。
- **バイナリサイズ削減**: プロファイル設定およびリンク最適化によりバイナリサイズを約 30% 削減 (`287KB` → `203KB`)。

### Fixed
- JSON 文字列（パスやダブルクォート等）の `\` エスケープシーケンス（`\\`, `\"`, `\n` 等）がパース時にアンエスケープされずそのまま出力される不具合を修正。`Cow<'a, str>` によるスマートアンエスケープによりエスケープ非検出時は Zero-Allocation 高速性を維持。

## [0.2.1] - 2026-07-29

### Fixed
- Windows環境において、Gitコマンド実行時に Clink が自動インジェクトされ失敗する問題（`Unable to inject Clink.` エラー）を防ぐため、`CLINK_NOINJECT=1` を付与して無効化・抑止する処理を追加。

## [0.2.0] - 2026-07-08

### Added
- WindowsホストからLinux (musl) へのクロスビルド用設定（.cargo/config.toml）を追加。
- バーチャート（コンテキストおよびAPIクォータ）にUnicode縦分割ブロック要素（▏▎▍▌▋▊▉█）を導入し、1/8ステップ（1.25%刻み）の繊細で滑らかなプログレス描画に対応。

### Changed
- ワイド表示時も含めて右パディングを廃止し、すべての情報を左詰めで一列に表示するシンプルなレイアウトに統一。
- ゲージ表示（コンテキストおよびAPIクォータ）の長さを20文字から10文字に半減。
- クォータバー（Limit）とコンテキストサイズ（使用率）の色分け規則を、割合（使用量）に応じた警告色（赤・黄・緑）に変更。
- VCSブランチ名（vcs_str）の表示位置をモデル名の右隣に固定。

### Removed
- 電源ステータス表示（バッテリーおよびAC接続情報）を完全に削除。
- ホスト名（hostname）およびTailscale IPアドレス（tailscale_ip）の表示と取得処理を完全に削除。
- バージョン情報、アカウント名、プラン名、カレントディレクトリ、会話IDの表示を完全に削除。
- 画面幅によるナロー/ワイドの分岐およびモデル名切り詰め処理を完全に廃止し、常にすべての詳細情報を左詰めで一列に表示するレイアウトに統一。
- 各表示要素（モデル名、VCS等）の装飾カラーコードを削除し、デフォルトテキスト色に統一。

[0.2.2]: https://github.com/cwatanab/agy-statusline/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/cwatanab/agy-statusline/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/cwatanab/agy-statusline/compare/v0.1.0...v0.2.0
