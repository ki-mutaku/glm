# Data Model: リポジトリスコープの Issue 管理

**Date**: 2025-01-22  
**Feature**: 001-repo-scoped-issues

## Overview

この機能で使用するデータモデルとその関係性を定義します。

## Core Entities

### 1. Repository

**Purpose**: GitHub リポジトリを表現するエンティティ

**Fields**:

```rust
pub struct Repository {
    pub id: u64,                    // GitHub の内部 ID
    pub name: String,               // "owner/repo" 形式のフルネーム
    pub owner: String,              // オーナー名 (owner)
    pub repo: String,               // リポジトリ名 (repo)
    pub description: Option<String>, // リポジトリの説明
    pub stars: u32,                 // スター数
    pub private: bool,              // プライベートリポジトリか
}
```

**Validation Rules**:

- `name` は `owner/repo` 形式である必要がある
- `private` は常に `true` (プライベートリポジトリのみ表示)
- `stars` は非負整数

**Source**: Octocrab の `octocrab::models::Repository` から変換

---

### 2. Issue

**Purpose**: GitHub Issue を表現するエンティティ（既存）

**Fields**:

```rust
// octocrab::models::issues::Issue をそのまま使用
pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub repository_url: String,
    pub user: User,
    // ... その他の標準フィールド
}
```

**Relationship**: `repository_url` フィールドにより Repository と関連付けられる

---

### 3. App State

**Purpose**: アプリケーション全体の状態を管理

**Fields**:

```rust
pub struct App {
    // 既存フィールド
    pub octocrab: Octocrab,
    pub issues: Vec<Issue>,
    pub list_state: ListState,

    // 新規フィールド
    pub current_screen: Screen,
    pub selected_repository: Option<Repository>,
    pub repositories: Vec<Repository>,
    pub repo_list_state: ListState,
    pub issue_form: Option<IssueFormState>,
    pub error_message: Option<String>,
}
```

**State Transitions**: 「State Transitions」セクションを参照

---

### 4. Screen (Enum)

**Purpose**: 現在表示している画面を識別

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    IssueList,           // My Issues 画面 (デフォルト)
    RepositorySelector,  // リポジトリ選択画面
    IssueForm,           // Issue 作成フォーム画面
}
```

**Initial State**: `Screen::IssueList`

---

### 5. IssueFormState

**Purpose**: Issue 作成フォームの入力状態

```rust
pub struct IssueFormState {
    pub title: String,           // Issue タイトル
    pub body: String,            // Issue 本文
    pub focused_field: FormField, // フォーカス中のフィールド
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    Title,
    Body,
}
```

**Validation Rules**:

- `title` は空でない文字列（最低 1 文字）
- `body` は空でも可（GitHub API の制約に準拠）

**State Transitions**:

- `Tab` キー: `FormField::Title` ⇄ `FormField::Body`
- `Enter` キー (Title フィールド): `FormField::Body` に移動
- `Enter` キー (Body フィールド): フォーム送信

---

## Relationships

### Repository → Issues (1:N)

- 1 つの Repository は複数の Issue を持つ
- `App.selected_repository` が設定されている場合、`App.issues` にはそのリポジトリの Issue のみが格納される
- `App.selected_repository` が `None` の場合、`App.issues` にはユーザーに割り当てられた全 Issue が格納される（既存動作）

### App → Screen (1:1)

- `App` は常に 1 つの `Screen` を持つ
- `current_screen` フィールドにより現在の画面が決定される

### App → Repository (0..1)

- `App` は 0 または 1 つの選択済み Repository を持つ
- `selected_repository: Option<Repository>` で表現

---

## State Transitions

### Screen Transitions

```
┌─────────────┐
│ IssueList   │◄────────────┐
└──────┬──────┘             │
       │ 'r'                │ Esc
       ▼                    │
┌──────────────────┐        │
│RepositorySelector├────────┘
└──────┬───────────┘
       │ Enter
       │ (repo selected)
       ▼
┌─────────────┐
│ IssueList   │
│ (filtered)  │
└──────┬──────┘
       │ 'n'
       ▼
┌─────────────┐
│ IssueForm   │
└──────┬──────┘
       │ Enter (submit)
       │ or Esc (cancel)
       ▼
┌─────────────┐
│ IssueList   │
│ (refresh)   │
└─────────────┘
```

### Data Flow

#### 1. リポジトリ選択フロー

```
User presses 'r'
  → App.current_screen = Screen::RepositorySelector
  → Fetch repositories from GitHub API
  → App.repositories = fetched repos (filtered by private)
  → Render repository list

User selects a repo (Enter)
  → App.selected_repository = Some(selected_repo)
  → App.current_screen = Screen::IssueList
  → Fetch issues from selected repo
  → App.issues = fetched issues
  → Render issue list with repo header
```

#### 2. Issue 作成フロー

```
User presses 'n' (on IssueList with repo selected)
  → Check: App.selected_repository.is_some()
  → If None: App.error_message = "Please select a repository first"
  → If Some:
      → App.current_screen = Screen::IssueForm
      → App.issue_form = Some(IssueFormState::default())
      → Render issue form

User fills title and body
  → Update App.issue_form.title and .body

User presses Enter (submit)
  → Call octocrab.issues(owner, repo).create(title).body(body).send()
  → If success:
      → App.current_screen = Screen::IssueList
      → Refresh App.issues (re-fetch from API)
  → If error:
      → App.error_message = error details
      → Stay on Screen::IssueForm (user can retry)

User presses Esc (cancel)
  → App.current_screen = Screen::IssueList
  → App.issue_form = None
```

#### 3. エラーハンドリングフロー

```
API call fails (network, auth, rate limit)
  → Capture error with anyhow::Error
  → App.error_message = Some(user_friendly_message)
  → Render error toast at top of screen

User presses any key
  → App.error_message = None
  → Error toast disappears
```

---

## Validation Rules Summary

| Entity         | Field                 | Rule                                  |
| -------------- | --------------------- | ------------------------------------- |
| Repository     | `name`                | Must match `^[^/]+/[^/]+$`            |
| Repository     | `private`             | Must be `true` (filter applied)       |
| Repository     | `stars`               | Must be `>= 0`                        |
| IssueFormState | `title`               | Must not be empty (`len() > 0`)       |
| IssueFormState | `body`                | Can be empty                          |
| App            | `selected_repository` | Must be `Some(_)` when creating issue |

---

## API Mapping

### Octocrab → Internal Model

#### Repository

```rust
// Octocrab の Repository を内部 Repository に変換
fn from_octocrab_repo(repo: octocrab::models::Repository) -> Repository {
    Repository {
        id: repo.id.0,
        name: repo.full_name.clone().unwrap_or_default(),
        owner: repo.owner.clone().map(|u| u.login).unwrap_or_default(),
        repo: repo.name.clone(),
        description: repo.description.clone(),
        stars: repo.stargazers_count.unwrap_or(0),
        private: repo.private.unwrap_or(false),
    }
}
```

#### Issue

```rust
// Octocrab の Issue をそのまま使用 (変換不要)
// octocrab::models::issues::Issue
```

---

## Index & Query Patterns

### 1. Repository Lookup by Name

**Use Case**: Issue 作成時に owner/repo を指定

**Query**:

```rust
app.repositories.iter()
    .find(|r| r.name == "owner/repo")
```

**Optimization**: 小規模リスト (<100 repos) のため線形探索で十分。大規模化時は `HashMap<String, Repository>` を検討。

---

### 2. Issue Filtering by Repository

**Use Case**: 選択リポジトリの Issue のみ表示

**Query**:

```rust
// API レベルでフィルタリング (クライアント側不要)
octocrab.issues(owner, repo).list().send().await
```

---

## Memory Considerations

- **repositories**: 100 repos × 200 bytes ≈ 20 KB (許容範囲)
- **issues**: 100 issues × 500 bytes ≈ 50 KB (許容範囲)
- **Total App State**: < 100 KB (TUI アプリとして十分軽量)

ページネーションは API レベルで実装済み（研究フェーズで確認済み）。
