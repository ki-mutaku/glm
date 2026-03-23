# GitHub Life Manager (GLM) 設計書

## 1. 概要
`ratatui` を使用した、`lazygit` 風のマルチパネル TUI (Terminal User Interface) ツール。
GitHub の Issue を効率的に管理し、詳細な編集が必要な場合のみ環境変数 `$EDITOR`（Neovim, Vim 等）を外部プロセスとして起動するハイブリッド設計。

## 2. システムアーキテクチャ
- **UI Layer:** `ratatui` + `crossterm` による 3 パネル構成。
- **Data Layer:** `octocrab` による GitHub API 連携。認証は `gh auth token` を利用。
- **Editor Bridge:** 一時ファイルを作成し、外部エディタをサブプロセスとして起動。
- **Runtime:** `tokio` による非同期実行環境。
- **Error Handling:** `anyhow` による一貫したエラー処理とコンテキスト提供。

## 3. UI レイアウト設計
画面を左右に 3:7 の比率で分割し、左側をさらに上下に分割。

| パネル | 内容 | 操作 |
| :--- | :--- | :--- |
| **Sidebar (左上)** | カテゴリ (My Issues / Inbox / Projects) | 将来的に切り替えを実装予定 |
| **List (左下)** | Issue タイトルの一覧 | `j`/`k` または矢印キーで移動 |
| **Main (右側)** | Issue 本文 (Markdown) | リスト選択に合わせてリアルタイム表示 |
| **Status (下部)** | 操作ヘルプ・同期状態 | `q` で終了、`e` で編集（今後実装） |

## 4. エディタ ($EDITOR) 連携の仕組み
以下のフローを実装予定：
1. **一時ファイル作成:** `tempfile` クレート等を使用し、Issue の現在の本文を `.md` ファイルとして書き出す。
2. **エディタ起動:** `std::process::Command` を使用し、子プロセスとして `$EDITOR` を起動。
3. **API 更新:** エディタ終了後、一時ファイルの内容を読み取り、GitHub API へ `PATCH` リクエストを送信。
4. **UI 復帰:** ターミナルの「Raw mode」を再有効化し、TUI の描画を再開。

## 5. 実装ステータス (MVP 完了)
- [x] `gh auth token` コマンドからのトークン自動取得。
- [x] 自分にアサインされたオープンな Issue の一覧表示。
- [x] 3 パネルレイアウトの構築と基本ナビゲーション。
- [x] エラー処理の `anyhow` 統合と日本語コメント化。

## 6. 今後の展望
- **Editor Bridge の実装:** 実際に `e` キーでエディタを立ち上げる機能。
- **Sync Manager:** GitHub API との定期的な同期や更新処理の抽象化。
- **Pomo-tui 連携:** ステータスラインにポモドーロタイマーの残り時間を表示。
- **ローカルキャッシュ:** `sqlx` 等を用いた Issue データのローカルキャッシュ化。
