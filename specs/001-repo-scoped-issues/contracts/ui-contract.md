# User Interface Contract: リポジトリスコープの Issue 管理

**Date**: 2025-01-22  
**Feature**: 001-repo-scoped-issues  
**Contract Type**: TUI (Terminal User Interface)

## Overview

GitHub Life Manager (GLM) は TUI アプリケーションとして、キーボード操作による完全なインタラクティブ体験を提供します。本ドキュメントは、ユーザーに公開される UI コントラクト（画面構成、キーバインディング、状態遷移）を定義します。

---

## Screen Contracts

### 1. IssueList Screen (My Issues)

**Purpose**: ユーザーに割り当てられた Issue、または選択リポジトリの Issue を表示

**Layout**:
```
┌───────────────────────────────────────────────────────┐
│ [Repository: owner/repo]              [r] Select Repo │ ← ヘッダー
├─────────────────┬─────────────────────────────────────┤
│ カテゴリ        │ 詳細                                │
│ ● My Issues     │ #123 Issue Title                    │
│   Inbox         │                                     │
│   Projects      │ Issue body content here...          │
│                 │                                     │
├─────────────────┤                                     │
│ Issue 一覧      │                                     │
│ >> [Open] Issue1│                                     │
│    [Open] Issue2│                                     │
│    [Open] Issue3│                                     │
└─────────────────┴─────────────────────────────────────┘
│ q: 終了 | e: 編集 | j/k: 移動 | r: リポジトリ選択 | n: 新規Issue │
└───────────────────────────────────────────────────────┘
```

**Key Bindings**:
| Key | Action | Condition |
|-----|--------|-----------|
| `q` | アプリを終了 | Always |
| `j` or `↓` | 次の Issue を選択 | Always |
| `k` or `↑` | 前の Issue を選択 | Always |
| `e` | 選択中の Issue を外部エディタで編集 | Issue が選択されている |
| `r` | リポジトリ選択画面に遷移 | Always |
| `n` | Issue 作成フォームに遷移 | リポジトリが選択されている |

**Header Display Rules**:
- リポジトリ選択済み: `Repository: owner/repo` を表示
- リポジトリ未選択: ヘッダーなし、または `All Issues (assigned to you)` を表示

**Error Handling**:
- エラー発生時: ヘッダー下に赤背景の ERROR バー表示
- 任意のキー押下でエラーをクリア

---

### 2. RepositorySelector Screen

**Purpose**: プライベートリポジトリ一覧から選択

**Layout**:
```
┌───────────────────────────────────────────────────────┐
│ Select Repository                        [Esc] Cancel │
├────────────────────┬──────────────────────────────────┤
│ >> owner/repo1     │ Repository: owner/repo1          │
│    owner/repo2     │ Description: A sample repo       │
│    owner/repo3     │ Stars: ⭐ 123                    │
│    ...             │ Private: Yes                     │
│                    │                                  │
│                    │                                  │
│ (50 repositories)  │                                  │
└────────────────────┴──────────────────────────────────┘
│ j/k: Navigate | Enter: Select | Esc: Cancel           │
└───────────────────────────────────────────────────────┘
```

**Key Bindings**:
| Key | Action |
|-----|--------|
| `j` or `↓` | 次のリポジトリを選択 |
| `k` or `↑` | 前のリポジトリを選択 |
| `Enter` | 選択中のリポジトリを確定し、IssueList に戻る |
| `Esc` | キャンセルして IssueList に戻る（選択変更なし） |

**Empty State**:
```
┌───────────────────────────────────────────────────────┐
│ Select Repository                        [Esc] Cancel │
├───────────────────────────────────────────────────────┤
│                                                       │
│          No private repositories found.               │
│                                                       │
│     Please check your GitHub access permissions      │
│              or run `gh auth login`.                  │
│                                                       │
└───────────────────────────────────────────────────────┘
│ Esc: Back                                             │
└───────────────────────────────────────────────────────┘
```

**Error Handling**:
- API エラー時: エラーメッセージを画面中央に表示、Esc で戻る

---

### 3. IssueForm Screen

**Purpose**: 新規 Issue を作成

**Layout**:
```
┌───────────────────────────────────────────────────────┐
│ Create Issue: owner/repo                [Esc] Cancel  │
├───────────────────────────────────────────────────────┤
│ Title:                                                │
│ ┌───────────────────────────────────────────────────┐ │
│ │ [Focused: Type your issue title here]            │ │
│ └───────────────────────────────────────────────────┘ │
│                                                       │
│ Body:                                                 │
│ ┌───────────────────────────────────────────────────┐ │
│ │                                                   │ │
│ │ [Type issue description]                          │ │
│ │                                                   │ │
│ │ Press Ctrl+E to open external editor              │ │
│ │                                                   │ │
│ └───────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────┘
│ Tab: Switch Field | Enter: Submit | Ctrl+E: Editor | Esc: Cancel │
└───────────────────────────────────────────────────────┘
```

**Key Bindings**:
| Key | Action | Condition |
|-----|--------|-----------|
| `Tab` | Title ⇄ Body のフォーカス切替 | Always |
| `Enter` (Title) | Body フィールドに移動 | Title フィールドにフォーカス |
| `Enter` (Body) | Issue を GitHub に作成して送信 | Body フィールドにフォーカス & Title が空でない |
| `Ctrl+E` | 外部エディタで Body を編集 | Always |
| `Esc` | キャンセルして IssueList に戻る（入力破棄） | Always |
| 文字入力 | フォーカス中のフィールドにテキスト追加 | Always |
| `Backspace` | フォーカス中のフィールドの最後の文字を削除 | Always |

**Field Focus Indication**:
- フォーカス中: フィールド境界を青色でハイライト
- 非フォーカス: グレー境界

**Validation**:
- Title が空の場合: Enter 押下時に「Title is required」エラーを表示
- Body は空でも可

**Success Feedback**:
- Issue 作成成功時: IssueList に戻り、画面上部に緑色で「Issue #123 created successfully」を 3 秒間表示

---

## State Transition Contract

### Global Transitions

```
┌─────────────┐
│ IssueList   │ <──────┐
│ (initial)   │        │
└──────┬──────┘        │
       │ 'r'           │ Esc
       ▼               │
┌──────────────────┐   │
│RepositorySelector├───┘
└──────┬───────────┘
       │ Enter
       │ (repo selected)
       ▼
┌─────────────┐
│ IssueList   │
│ (filtered)  │ <──────┐
└──────┬──────┘        │
       │ 'n'           │ Enter (success)
       │               │ or Esc
       ▼               │
┌─────────────┐        │
│ IssueForm   ├────────┘
└─────────────┘
```

### Prohibited Transitions

- **IssueList (no repo selected) → IssueForm**: `n` キー押下時にエラーメッセージ表示
- **Any Screen → IssueList (via 'q')**: `q` は終了のみ、画面遷移には使用しない

---

## Error Message Contract

### Error Display Format

```
┌───────────────────────────────────────────────────────┐
│ ❌ ERROR: [Error message here]       [Press any key] │ ← 赤背景
├───────────────────────────────────────────────────────┤
│ (Normal screen content below)                         │
```

### Standard Error Messages

| Error Condition | Message |
|----------------|---------|
| GitHub API 認証失敗 | `Authentication failed. Run 'gh auth login' to re-authenticate.` |
| ネットワークエラー | `Network error: Could not connect to GitHub. Check your internet connection.` |
| レート制限超過 | `GitHub API rate limit exceeded. Please try again later.` |
| リポジトリが見つからない | `Repository not found or access denied.` |
| Issue 作成失敗 | `Failed to create issue: [API error details]` |
| リポジトリ未選択で Issue 作成試行 | `Please select a repository first (press 'r').` |
| Title が空で送信試行 | `Issue title is required.` |

---

## Color Scheme Contract

| Element | Color | Purpose |
|---------|-------|---------|
| Normal Text | White | 通常のテキスト |
| Highlighted Item | White on DarkGray | 選択中の項目 |
| Border | Gray | ウィジェット境界 |
| Status Bar | White on Blue | ヘルプテキスト |
| Error Bar | White on Red | エラーメッセージ |
| Success Notification | White on Green | 成功メッセージ |
| Focused Field | Blue Border | フォーカス中のフォームフィールド |

---

## Accessibility Contract

- **キーボード専用**: マウス不要で全操作可能
- **明確なフォーカス表示**: 選択項目は `>>` プレフィックスとハイライトで明示
- **ヘルプテキスト常時表示**: 各画面下部にキーバインディングガイドを表示
- **エラーメッセージの明確性**: エラー理由と対処法を含む

---

## Performance Contract

- **画面遷移**: < 100ms (API 呼び出し除く)
- **キー入力レスポンス**: < 50ms
- **API 呼び出し中**: ローディング表示またはブロッキング UI (「Loading...」表示)

---

## Backward Compatibility

**既存機能の保持**:
- リポジトリ未選択時: 従来の「My Issues」動作を維持
- `q`, `j/k`, `e` キーは既存の動作を保持

**Breaking Changes**:
- なし（すべて追加機能）
