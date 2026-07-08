# agy-statusline

Antigravity CLI 向けの高度なステータスラインプラグイン。オリジナルの [antigravity-cli-statusline](https://github.com/weby-homelab/antigravity-cli-statusline) を Rust に移植し、Nerd Font アイコン、クォータバー表示、クロスプラットフォーム対応を強化しました。

## レイアウト

ターミナル幅に応じて自動的に3つのレイアウトを切り替えます：

| レイアウト | 幅 | 説明 |
|---|---|---|
| Wide | ≥ 180 cols | 左右寄せ1行ですべての情報を表示 |
| Medium | ≥ 90 cols | 2行の枠付きレイアウト |
| Narrow | < 90 cols | コンパクト2行表示 |

## 機能

- **レスポンシブレイアウト** — ターミナル幅に応じて自動切替
- **Nerd Font アイコン** — デフォルトで Nerd Font アイコン使用、`--classic` で ASCII 互換モード
- **クォータバー** — 5時間/週間の残量を20セグメントバーで可視化（<20% 赤、<50% 黄、≥50% 緑）
- **リセット時間表示** — クォータリセットまでの残り時間を表示（例: `⌛️ 3h 30m`）
- **Git 直接取得** — JSON ではなく `git` コマンドからリアルタイムのブランチ・変更状態を取得
- **トークン使用量** — 入出力トークンを人間可読形式（K/M）で表示
- **サンドボックス状態** — ON (net) / ON (no-net) / OFF
- **ホスト情報** — ホスト名・Tailscale IP 表示
- **電源状態** — AC接続/バッテリー駆動と残量表示
- **ユーザー情報** — プラン・メールアドレス表示

## インストール

### ビルド済みバイナリ

[Releases](https://github.com/cwatanab/agy-statusline/releases) から各プラットフォームのバイナリをダウンロードしてください：

- `statusline-windows-x86_64.exe` — Windows (x64)
- `statusline-linux-x86_64` — Linux (x64)
- `statusline-linux-arm64` — Linux (ARM64)

### ソースからビルド

```bash
git clone https://github.com/cwatanab/agy-statusline.git
cd agy-statusline
cargo build --release
```

### 設定

`~/.agy/settings.json` に以下を追加：

```json
{
  "statusLine": {
    "type": "",
    "command": "/path/to/statusline",
    "enabled": true
  }
}
```

クラシックモード（Nerd Font 不要）を使用する場合：

```json
{
  "statusLine": {
    "type": "",
    "command": "/path/to/statusline --classic",
    "enabled": true
  }
}
```

## コマンドラインオプション

| オプション | 説明 |
|---|---|
| `--classic` | ASCII 互換モード（Nerd Font 不要） |
| `--no-nerdfont` | `--classic` のエイリアス |
| `--compatibility` | `--classic` のエイリアス |

## 謝辞

このプロジェクトは [weby-homelab/antigravity-cli-statusline](https://github.com/weby-homelab/antigravity-cli-statusline) をベースにしています。
オリジナルの作者である Weby Homelab に感謝します。

> Built in Ukraine under air raid sirens & blackouts ⚡
> © 2026 Weby Homelab
