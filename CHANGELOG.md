# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.2.0]: https://github.com/cwatanab/agy-statusline/compare/v0.1.0...v0.2.0
