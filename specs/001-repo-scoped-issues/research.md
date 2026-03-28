# Research: リポジトリスコープの Issue 管理

**Date**: 2025-01-22  
**Feature**: 001-repo-scoped-issues

## Overview

このドキュメントは、リポジトリスコープの Issue 管理機能を実装するための技術調査結果をまとめたものです。

## Research Tasks

### 1. Ratatui での複数画面状態管理パターン

**Decision**: Screen enum + match ベースの描画分岐パターンを採用

**Rationale**:
- Ratatui は React のような状態管理ライブラリを持たないため、Rust の enum を使った画面管理が一般的
- 既存の `App` 構造体に `current_screen: Screen` フィールドを追加し、`ui.rs` で match 文により描画を分岐
- シンプルで Rust のパターンマッチを活用でき、コンパイル時に画面遷移の妥当性をチェック可能

**Alternatives considered**:
- **Option 1**: 各画面を trait で抽象化する → 過剰設計、3 画面程度では不要
- **Option 2**: 外部状態管理クレート (e.g., `druid`, `iced`) → TUI 用ではなく GUI 向け、依存増加

**Implementation Pattern**:
```rust
pub enum Screen {
    IssueList,
    RepositorySelector,
    IssueForm,
}

// ui.rs 内
pub fn ui(f: &mut Frame, app: &mut App) {
    match app.current_screen {
        Screen::IssueList => render_issue_list(f, app),
        Screen::RepositorySelector => render_repo_selector(f, app),
        Screen::IssueForm => render_issue_form(f, app),
    }
}
```

---

### 2. Octocrab でのリポジトリ一覧取得

**Decision**: `octocrab.current().list_repos_for_authenticated_user()` を使用し、プライベートリポジトリをフィルタリング

**Rationale**:
- Octocrab 0.38 は `repos()` ハンドラーと `current()` (認証ユーザー) ハンドラーを提供
- `current().list_repos_for_authenticated_user()` で認証ユーザーのリポジトリを取得可能
- 返される `Repository` 型には `private: Option<bool>` フィールドがあり、プライベート判定が可能
- ページネーションは `octocrab::Page<T>` で提供され、`next` メソッドで次ページを取得

**API Usage**:
```rust
use octocrab::models::Repository;

let mut page = octocrab
    .current()
    .list_repos_for_authenticated_user()
    .per_page(100)
    .send()
    .await?;

let mut repos = Vec::new();
loop {
    repos.extend(page.items.into_iter().filter(|r| r.private == Some(true)));
    page = match octocrab.get_page(&page.next).await? {
        Some(next_page) => next_page,
        None => break,
    };
}
```

**Alternatives considered**:
- **Option 1**: GraphQL API を使う → Octocrab は REST API 中心、GraphQL は複雑化
- **Option 2**: Search API (`is:private`) → 検索クエリベースで不安定、公式の repos エンドポイントの方が信頼性高い

---

### 3. リポジトリスコープでの Issue 取得

**Decision**: `octocrab.issues(owner, repo).list()` で特定リポジトリの Issue を取得

**Rationale**:
- 既存コードは `octocrab.search().issues_and_pull_requests("is:issue is:open assignee:@me")` を使用
- リポジトリを選択した場合は、`issues(owner, repo).list()` API に切り替え
- このエンドポイントは特定リポジトリの Issue のみを返すため、フィルタリングが不要

**API Usage**:
```rust
let page = octocrab
    .issues(owner, repo)
    .list()
    .state(octocrab::params::State::Open)
    .per_page(100)
    .send()
    .await?;

let issues = page.items;
```

**State Management**:
```rust
pub struct App {
    pub octocrab: Octocrab,
    pub issues: Vec<Issue>,
    pub list_state: ListState,
    pub current_screen: Screen,
    pub selected_repository: Option<Repository>, // 新規追加
}
```

**Alternatives considered**:
- **Option 1**: 引き続き Search API を使い `repo:owner/repo` クエリを追加 → 非効率、Issue リストAPIの方が直接的
- **Option 2**: すべての Issue を一度取得してクライアント側でフィルタ → API 呼び出し回数増加、非効率

---

### 4. Issue 作成 API 統合

**Decision**: `octocrab.issues(owner, repo).create(title).body(body).send()` を使用

**Rationale**:
- Octocrab の `IssueHandler` は `create()` ビルダーメソッドを提供
- タイトルと本文のみ必須、ラベル・マイルストーン・担当者は v1 でスコープ外
- 既存の外部エディタ統合 (`gh::edit_with_external_editor`) を再利用可能

**API Usage**:
```rust
let new_issue = octocrab
    .issues(owner, repo)
    .create(title)
    .body(body)
    .send()
    .await?;
```

**Form State Management**:
```rust
pub struct IssueFormState {
    pub title: String,
    pub body: String,
    pub focused_field: FormField, // Title or Body
}

pub enum FormField {
    Title,
    Body,
}
```

**Alternatives considered**:
- **Option 1**: GitHub CLI (`gh issue create`) をラップする → API 直接呼び出しの方が柔軟、エラーハンドリングが容易
- **Option 2**: 全フィールドを一度に入力 → 段階的入力（タイトル→本文→送信）の方がUX良好

---

### 5. エラーハンドリング戦略

**Decision**: エラーを App 構造体に保持し、UI にトーストとして表示

**Rationale**:
- API エラー (ネットワーク、認証、レート制限) は `anyhow::Error` でキャプチャ
- `App` に `error_message: Option<String>` フィールドを追加
- UI 描画時にエラーがあればステータスバーまたは専用エリアに赤色で表示
- ユーザーが任意のキーを押すとエラーをクリア

**Implementation**:
```rust
pub struct App {
    // ... existing fields
    pub error_message: Option<String>,
}

impl App {
    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
}
```

**UI Rendering**:
```rust
// ui.rs 内、ステータスバーの上にエラーバーを追加
if let Some(err) = &app.error_message {
    let error_bar = Paragraph::new(err.as_str())
        .style(Style::default().bg(Color::Red).fg(Color::White));
    f.render_widget(error_bar, error_area);
}
```

**Alternatives considered**:
- **Option 1**: エラーごとに panic → ユーザー体験が悪い、アプリが終了
- **Option 2**: ログファイルに記録 → ユーザーがエラーに気づかない可能性
- **Option 3**: モーダルダイアログ → TUI では実装が複雑、トーストで十分

---

### 6. キーボードショートカット設計

**Decision**: 既存パターンを踏襲し、画面ごとに有効なキーを制限

**Rationale**:
- 既存: `q` (終了), `j/k` (上下), `e` (編集)
- 新規追加: `r` (リポジトリ選択画面を開く), `n` (Issue 作成), `Esc` (前の画面に戻る), `Enter` (選択/送信)
- 各画面で無効なキーは無視し、ヘルプテキストに表示しない

**Key Mapping by Screen**:

| Screen | Available Keys |
|--------|----------------|
| IssueList | `q`, `j/k`, `e`, `r`, `n` (repo 選択時のみ) |
| RepositorySelector | `j/k`, `Enter`, `Esc` |
| IssueForm | `Tab` (フィールド切替), `Enter` (送信), `Esc` (キャンセル), `Ctrl+E` (外部エディタ) |

**Alternatives considered**:
- **Option 1**: Vim ライクなモード (Normal/Insert) → TUI では過剰、シンプルな直接操作で十分
- **Option 2**: 数字キーでジャンプ → 50+ リポジトリでは不十分、j/k で十分

---

### 7. UI レイアウト設計

**Decision**: 既存の左右分割レイアウトを維持し、リポジトリ選択と Issue 作成は全画面モーダル風に表示

**Rationale**:
- **IssueList 画面**: 既存の 30%/70% 左右分割を維持。ヘッダーに選択リポジトリ名を追加
- **RepositorySelector 画面**: 全画面を使い、左側にリポジトリリスト、右側に選択リポジトリの詳細（説明、スター数）を表示
- **IssueForm 画面**: 全画面フォーム、タイトルフィールド (1行) と本文フィールド (複数行) を上下配置

**Layout Examples**:

#### IssueList (変更箇所のみ)
```
┌─────────────────────────────────────────────┐
│ Repository: owner/repo         [r] Select   │ ← 新規ヘッダー行
├─────────────────────────────────────────────┤
│ カテゴリ  │ 詳細                            │
│ ...       │ ...                             │
└─────────────────────────────────────────────┘
```

#### RepositorySelector
```
┌─────────────────────────────────────────────┐
│ Select Repository               [Esc] Back  │
├──────────────────┬──────────────────────────┤
│ >> owner/repo1   │ Name: owner/repo1        │
│    owner/repo2   │ Description: ...         │
│    owner/repo3   │ Stars: 123               │
│ ...              │ Private: Yes             │
└──────────────────┴──────────────────────────┘
│ j/k: Navigate | Enter: Select | Esc: Cancel│
└─────────────────────────────────────────────┘
```

#### IssueForm
```
┌─────────────────────────────────────────────┐
│ Create Issue: owner/repo       [Esc] Cancel │
├─────────────────────────────────────────────┤
│ Title: [                                  ] │
│                                             │
│ Body:                                       │
│ ┌─────────────────────────────────────────┐ │
│ │                                         │ │
│ │                                         │ │
│ │ [Press Ctrl+E for external editor]     │ │
│ │                                         │ │
│ └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│ Tab: Switch | Enter: Submit | Esc: Cancel   │
└─────────────────────────────────────────────┘
```

**Alternatives considered**:
- **Option 1**: すべて左右分割維持 → リポジトリリストが狭くなり視認性低下
- **Option 2**: タブベースの画面切替 → 状態管理が複雑化、モーダル風の方がシンプル

---

## Summary

本研究により、以下の技術的方針が確立されました：

1. **状態管理**: Screen enum + App 構造体の拡張
2. **API 統合**: Octocrab の標準メソッドを使用（リポジトリ一覧、Issue 一覧、Issue 作成）
3. **エラーハンドリング**: トースト形式でユーザーフレンドリーなメッセージ表示
4. **UI レイアウト**: 既存レイアウト維持 + 新画面はモーダル風全画面表示
5. **キーボード操作**: 既存パターン踏襲 (`j/k`, `Enter`, `Esc`) + 新規ショートカット (`r`, `n`)

すべての NEEDS CLARIFICATION が解決され、Phase 1 (設計) に進む準備が整いました。
