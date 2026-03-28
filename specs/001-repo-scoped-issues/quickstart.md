# Quickstart Guide: リポジトリスコープの Issue 管理

**Feature**: 001-repo-scoped-issues  
**Date**: 2025-01-22  
**Target Audience**: 開発者・実装者

## Overview

本ガイドは、リポジトリスコープの Issue 管理機能を実装する開発者向けのクイックスタートです。設計ドキュメント全体を読まなくても、基本的な実装方針と開始手順を理解できます。

---

## 5-Minute Summary

### What We're Building

GitHub Life Manager (GLM) に以下 3 つの機能を追加：

1. **リポジトリ選択画面** (`r` キー) → プライベートリポジトリ一覧を表示
2. **リポジトリスコープの Issue 表示** → 選択リポジトリの Issue のみフィルタリング
3. **Issue 作成フォーム** (`n` キー) → 選択リポジトリに新規 Issue を作成

### Key Technical Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| 状態管理 | `Screen` enum + `App` 構造体拡張 | シンプル、Rust パターンマッチ活用 |
| API クライアント | Octocrab 0.38 (既存) | 追加依存なし |
| UI フレームワーク | Ratatui 0.26 (既存) | 既存レイアウトを維持 |
| エラーハンドリング | トースト形式 (`App.error_message`) | ユーザーフレンドリー |
| テスト戦略 | 単体テスト + 統合テスト | GitHub API モック + E2E |

---

## Architecture Overview

### Before (既存)

```
main.rs
  └─> App (issues: Vec<Issue>, list_state: ListState)
       └─> ui.rs (単一画面: Issue リスト)
```

### After (新規)

```
main.rs
  └─> App
       ├─> current_screen: Screen (IssueList | RepositorySelector | IssueForm)
       ├─> selected_repository: Option<Repository>
       ├─> repositories: Vec<Repository>
       ├─> issue_form: Option<IssueFormState>
       └─> error_message: Option<String>
  
ui.rs
  ├─> render_issue_list() (既存を拡張)
  ├─> render_repo_selector() (新規)
  └─> render_issue_form() (新規)
```

---

## Implementation Phases

### Phase 1: Data Models & State Management (P0)

**Goal**: 基盤となるデータ構造と状態管理を実装

**Tasks**:
1. `src/models/repository.rs` を作成
   ```rust
   pub struct Repository {
       pub id: u64,
       pub name: String,
       pub owner: String,
       pub repo: String,
       pub description: Option<String>,
       pub stars: u32,
       pub private: bool,
   }
   ```

2. `src/app.rs` に新規フィールドを追加
   ```rust
   pub enum Screen {
       IssueList,
       RepositorySelector,
       IssueForm,
   }
   
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

3. `src/app.rs` にヘルパーメソッド追加
   ```rust
   impl App {
       pub fn set_error(&mut self, msg: String) { ... }
       pub fn clear_error(&mut self) { ... }
       pub fn select_repository(&mut self, repo: Repository) { ... }
   }
   ```

**Success Criteria**: コンパイルエラーなし、既存テストが通る

---

### Phase 2: GitHub API Integration (P0)

**Goal**: リポジトリ一覧取得と Issue フィルタリング API を実装

**Tasks**:
1. `src/gh.rs` に `fetch_repositories()` 関数を追加
   ```rust
   pub async fn fetch_repositories(octocrab: &Octocrab) -> Result<Vec<Repository>> {
       let mut page = octocrab
           .current()
           .list_repos_for_authenticated_user()
           .per_page(100)
           .send()
           .await?;
       
       let mut repos = Vec::new();
       loop {
           repos.extend(
               page.items.into_iter()
                   .filter(|r| r.private == Some(true))
                   .map(Repository::from)
           );
           page = match octocrab.get_page(&page.next).await? {
               Some(p) => p,
               None => break,
           };
       }
       Ok(repos)
   }
   ```

2. `src/gh.rs` に `fetch_issues_for_repo()` 関数を追加
   ```rust
   pub async fn fetch_issues_for_repo(
       octocrab: &Octocrab,
       owner: &str,
       repo: &str
   ) -> Result<Vec<Issue>> {
       let page = octocrab
           .issues(owner, repo)
           .list()
           .state(octocrab::params::State::Open)
           .per_page(100)
           .send()
           .await?;
       Ok(page.items)
   }
   ```

**Success Criteria**: 単体テスト (モック使用) でリポジトリ・Issue 取得が成功

---

### Phase 3: Repository Selector UI (P1)

**Goal**: リポジトリ選択画面の実装

**Tasks**:
1. `src/ui.rs` に `render_repo_selector()` 関数を追加
2. 左側: リポジトリリスト (`List` ウィジェット)
3. 右側: 選択リポジトリの詳細 (`Paragraph` ウィジェット)
4. ヘルプバー: "j/k: Navigate | Enter: Select | Esc: Cancel"

**Key Code**:
```rust
fn render_repo_selector(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(f.size());
    
    // 左: リポジトリリスト
    let items: Vec<ListItem> = app.repositories.iter()
        .map(|r| ListItem::new(r.name.clone()))
        .collect();
    let list = List::new(items)
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[0], &mut app.repo_list_state);
    
    // 右: 詳細
    if let Some(idx) = app.repo_list_state.selected() {
        let repo = &app.repositories[idx];
        let detail = format!(
            "Name: {}\nDescription: {}\nStars: {}",
            repo.name,
            repo.description.as_deref().unwrap_or("N/A"),
            repo.stars
        );
        let paragraph = Paragraph::new(detail);
        f.render_widget(paragraph, chunks[1]);
    }
}
```

**Main Loop Integration**:
```rust
// main.rs の run_app() 内
match key.code {
    KeyCode::Char('r') => {
        app.current_screen = Screen::RepositorySelector;
        // リポジトリリストを取得
        app.repositories = gh::fetch_repositories(&app.octocrab).await?;
    }
    KeyCode::Enter if app.current_screen == Screen::RepositorySelector => {
        if let Some(idx) = app.repo_list_state.selected() {
            let repo = app.repositories[idx].clone();
            app.select_repository(repo.clone());
            app.issues = gh::fetch_issues_for_repo(&app.octocrab, &repo.owner, &repo.repo).await?;
            app.current_screen = Screen::IssueList;
        }
    }
    KeyCode::Esc if app.current_screen == Screen::RepositorySelector => {
        app.current_screen = Screen::IssueList;
    }
    // ...
}
```

**Success Criteria**: `r` キーでリポジトリリストが表示され、Enter で選択・Esc でキャンセルが動作

---

### Phase 4: Scoped Issue Display (P1)

**Goal**: 選択リポジトリの Issue のみ表示

**Tasks**:
1. `src/ui.rs` の `render_issue_list()` を拡張
2. ヘッダーに選択リポジトリ名を表示

**Key Code**:
```rust
fn render_issue_list(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // ヘッダー
            Constraint::Min(0),    // メインエリア
            Constraint::Length(1), // ステータスバー
        ])
        .split(f.size());
    
    // ヘッダー: リポジトリ名
    let header_text = if let Some(repo) = &app.selected_repository {
        format!("Repository: {}                [r] Select Repo", repo.name)
    } else {
        "All Issues (assigned to you)         [r] Select Repo".to_string()
    };
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(header, main_chunks[0]);
    
    // ... 既存の Issue リスト描画コード ...
}
```

**Success Criteria**: リポジトリ選択後、ヘッダーに "Repository: owner/repo" が表示され、Issue リストがフィルタリングされる

---

### Phase 5: Issue Creation Form (P2)

**Goal**: Issue 作成フォームの実装

**Tasks**:
1. `src/app.rs` に `IssueFormState` 構造体を追加
   ```rust
   pub struct IssueFormState {
       pub title: String,
       pub body: String,
       pub focused_field: FormField,
   }
   
   pub enum FormField {
       Title,
       Body,
   }
   ```

2. `src/ui.rs` に `render_issue_form()` 関数を追加
3. `src/gh.rs` に `create_issue()` 関数を追加
   ```rust
   pub async fn create_issue(
       octocrab: &Octocrab,
       owner: &str,
       repo: &str,
       title: &str,
       body: &str
   ) -> Result<Issue> {
       octocrab.issues(owner, repo)
           .create(title)
           .body(body)
           .send()
           .await
           .context("Failed to create issue")
   }
   ```

4. `main.rs` のイベントループに `n` キー, `Tab` キー, テキスト入力処理を追加

**Success Criteria**: `n` キーでフォーム表示、タイトル・本文入力後に Enter で Issue 作成成功

---

### Phase 6: Error Handling & Polish (P2)

**Goal**: エラーハンドリングと UX 改善

**Tasks**:
1. すべての API 呼び出しに `match` または `?` でエラーハンドリング
2. エラー時に `app.set_error()` でメッセージ設定
3. `src/ui.rs` にエラーバー描画を追加
4. ローディング表示の追加 (オプション)

**Key Code**:
```rust
// エラーバー描画
if let Some(err) = &app.error_message {
    let error_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(f.size());
    
    let error_bar = Paragraph::new(format!("❌ ERROR: {}  [Press any key]", err))
        .style(Style::default().bg(Color::Red).fg(Color::White));
    f.render_widget(error_bar, error_chunks[0]);
}
```

**Success Criteria**: ネットワークエラー・認証エラー時にユーザーフレンドリーなメッセージが表示される

---

## Testing Strategy

### Unit Tests

**Target**: `src/gh.rs` の API 呼び出し関数

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fetch_repositories_filters_private() {
        // Mock Octocrab レスポンス
        // Assert: private == true のみ返される
    }
}
```

### Integration Tests

**Target**: 画面遷移フロー

```bash
$ cargo test --test workflow_tests
```

**Test Cases**:
1. リポジトリ選択 → Issue リストフィルタリング
2. Issue 作成 → リスト更新
3. エラーハンドリング

---

## Development Environment Setup

### Prerequisites

```bash
# GitHub CLI インストール & 認証
$ brew install gh
$ gh auth login

# Rust ツールチェーン (既にインストール済みと仮定)
$ rustc --version
```

### Build & Run

```bash
# 開発ビルド
$ cargo build

# 実行
$ cargo run

# テスト
$ cargo test

# Clippy (リンター)
$ cargo clippy
```

---

## Common Issues & Troubleshooting

### Issue: "gh auth token failed"

**Solution**: `gh auth login` を再実行

### Issue: "Repository not found"

**Possible Causes**:
1. リポジトリがプライベートで、トークンに `repo` スコープがない
2. リポジトリ名のタイポ

**Solution**: `gh auth refresh -s repo` でスコープを追加

### Issue: UI が乱れる

**Solution**: ターミナルサイズを確認 (最低 80x24)

---

## Next Steps

1. **Phase 1-2**: データモデルと API 統合 (1-2 日)
2. **Phase 3-4**: リポジトリ選択と Issue フィルタリング UI (2-3 日)
3. **Phase 5**: Issue 作成フォーム (2-3 日)
4. **Phase 6**: エラーハンドリングと polish (1 日)

**Total Estimate**: 6-9 営業日

---

## References

- **仕様書**: `specs/001-repo-scoped-issues/spec.md`
- **データモデル**: `specs/001-repo-scoped-issues/data-model.md`
- **API コントラクト**: `specs/001-repo-scoped-issues/contracts/api-contract.md`
- **UI コントラクト**: `specs/001-repo-scoped-issues/contracts/ui-contract.md`
- **リサーチ**: `specs/001-repo-scoped-issues/research.md`

---

## Quick Commands Cheat Sheet

| Command | Purpose |
|---------|---------|
| `cargo run` | アプリを起動 |
| `cargo test` | すべてのテストを実行 |
| `cargo clippy` | コード品質チェック |
| `cargo fmt` | コードフォーマット |
| `gh auth status` | 認証状態を確認 |
| `gh auth token` | トークンを表示 |

---

**Happy Coding! 🚀**
