# GitHub Actions 自動コードレビュー設定

このリポジトリには、PR作成時に自動的にコード品質チェックを実行し、レビューコメントを投稿するワークフローが設定されています。

## 🚀 機能

PRを作成すると、以下の自動チェックが実行されます：

### ✅ 自動実行される項目

1. **Clippy 静的解析** - Rustコードの問題を検出
2. **コードフォーマットチェック** - rustfmtによるスタイル確認
3. **ビルドチェック** - コンパイルエラーの検出
4. **変更ファイル一覧** - PRで変更されたファイルのリスト

### 📊 レビュー結果

すべてのチェック結果は自動的にPRコメントとして投稿されます：
- ✅ 問題なし：緑のチェックマーク
- ⚠️  警告あり：詳細を折りたたみ表示
- ❌ エラー：修正が必要な箇所を表示

## 💡 GitHub Copilotとの併用（推奨）

自動チェックに加えて、GitHub Copilotでより詳細なレビューができます：

### 1. GitHub.com上でのレビュー（最も簡単）
```
PRページ → "Files changed"タブ → 右上のCopilotアイコン → "Review changes"
```

### 2. ローカルでのレビュー
```bash
# PRをチェックアウト
gh pr checkout <PR番号>

# VSCodeでCopilot Chatを開く（Ctrl/Cmd+I）
# 「このコードをレビューして」と質問
```

### 3. CLI経由
```bash
gh copilot explain "このPRの変更内容を説明して"
gh copilot suggest "このコードの改善方法を教えて"
```

## 🔧 ワークフロー詳細

### トリガー
- PRの作成（opened）
- PRへの新しいコミット（synchronize）
- PRの再オープン（reopened）

### 実行内容
```yaml
jobs:
  code-analysis:
    - Rustツールチェーンのセットアップ
    - キャッシュの利用（高速化）
    - Clippy実行
    - rustfmtチェック
    - cargo check実行
    - 結果の集計とコメント投稿
```

## ⚙️ カスタマイズ

### 警告レベルの調整

`copilot-review.yml`のClippyオプションを変更：

```yaml
# 警告をエラーとして扱わない
cargo clippy --all-targets --all-features

# 特定のlintを無効化
cargo clippy -- -A clippy::style
```

### 特定のファイルのみをチェック

トリガー条件にパスフィルタを追加：

```yaml
on:
  pull_request:
    paths:
      - 'src/**/*.rs'
      - 'Cargo.toml'
```

### セキュリティチェックを追加

```yaml
- name: Security Audit
  run: |
    cargo install cargo-audit
    cargo audit
```

## 📝 使用方法

1. **コードを変更してコミット**
   ```bash
   git add .
   git commit -m "Add new feature"
   ```

2. **ブランチをプッシュ**
   ```bash
   git push origin feature-branch
   ```

3. **GitHubでPRを作成**
   - GitHub Actionsが自動実行
   - 数分後、PRにレビューコメントが投稿される

4. **結果を確認**
   - PRのActionsタブで詳細を確認
   - コメントの警告を修正
   - 必要に応じてCopilotで詳細レビュー

## 🐛 トラブルシューティング

### Actionsが実行されない
- リポジトリのSettings → Actions → General
- "Allow all actions and reusable workflows"を確認

### コメントが投稿されない
- Settings → Actions → General → Workflow permissions
- "Read and write permissions"を有効化

### Clippyエラーが表示される
```bash
# ローカルで事前確認
cargo clippy --all-targets --all-features -- -D warnings

# 警告を修正
cargo clippy --fix

# フォーマット修正
cargo fmt
```

## 🎓 学生プランユーザーへ

このワークフローは、GitHub学生プランで**完全に無料**で利用できます：
- ✅ GitHub Actions: 無料
- ✅ Clippy/rustfmt: オープンソース
- ✅ GitHub Copilot: 学生は無料

より高度な自動修正（Copilot Autofix）はBusiness/Enterpriseプラン限定ですが、このワークフローでも十分実用的なコードレビューが可能です！
