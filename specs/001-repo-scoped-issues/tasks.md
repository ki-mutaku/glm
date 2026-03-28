# Tasks: リポジトリスコープの Issue 管理

**Input**: 設計ドキュメント `/specs/001-repo-scoped-issues/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**組織構造**: タスクはユーザーストーリーごとにグループ化され、各ストーリーの独立した実装とテストを可能にします。

## フォーマット: `[ID] [P?] [Story] 説明`

- **[P]**: 並列実行可能（異なるファイル、依存関係なし）
- **[Story]**: このタスクが属するユーザーストーリー（例: US1, US2, US3）
- 説明には正確なファイルパスを含める

---

## Phase 0: 基盤構築（データ構造とAPI）

**目的**: データモデルと GitHub API 統合の基盤実装

**重要度**: この Phase の完了なしに、他のすべての Phase は開始できません。

### データモデルの実装

- [X] P0-T1 [P] `src/models/` ディレクトリを作成
  - **詳細**: 新しいデータモデル用のモジュールディレクトリを作成
  - **ファイル**: `src/models/mod.rs` を作成し、`pub mod repository;` を追加
  - **注意**: `src/main.rs` に `mod models;` を追加してモジュールを認識させる
  - **依存**: なし
  - **完了条件**: `cargo build` が通る、`src/models/` ディレクトリが存在
  - **見積**: 15分

- [X] P0-T2 [P] Repository 構造体を実装 in `src/models/repository.rs`
  - **詳細**: GitHub リポジトリを表現する構造体を定義
  ```rust
  #[derive(Debug, Clone)]
  pub struct Repository {
      pub id: u64,
      pub name: String,        // "owner/repo" 形式
      pub owner: String,
      pub repo: String,
      pub description: Option<String>,
      pub stars: u32,
      pub private: bool,
  }
  ```
  - **ファイル**: `src/models/repository.rs`
  - **依存**: P0-T1
  - **完了条件**: `cargo build` が通る、`Repository` 型が使用可能
  - **見積**: 30分

- [X] P0-T3 [P] Octocrab から Repository への変換を実装 in `src/models/repository.rs`
  - **詳細**: `From<octocrab::models::Repository>` トレイトを実装
  ```rust
  impl From<octocrab::models::Repository> for Repository {
      fn from(repo: octocrab::models::Repository) -> Self {
          let full_name = repo.full_name.clone().unwrap_or_default();
          let parts: Vec<&str> = full_name.split('/').collect();
          Self {
              id: repo.id.0,
              name: full_name,
              owner: parts.get(0).unwrap_or(&"").to_string(),
              repo: parts.get(1).unwrap_or(&"").to_string(),
              description: repo.description.clone(),
              stars: repo.stargazers_count.unwrap_or(0),
              private: repo.private.unwrap_or(false),
          }
      }
  }
  ```
  - **ファイル**: `src/models/repository.rs`
  - **注意**: `owner/repo` の分割処理でエッジケースを考慮（スラッシュなしの場合）
  - **依存**: P0-T2
  - **完了条件**: 変換ロジックが正常に動作、テストが通る
  - **見積**: 45分

### 状態管理の拡張

- [X] P0-T4 Screen enum を追加 in `src/app.rs`
  - **詳細**: 画面遷移を管理する enum を定義
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub enum Screen {
      IssueList,
      RepositorySelector,
      IssueForm,
  }
  ```
  - **ファイル**: `src/app.rs` の先頭に追加
  - **依存**: なし
  - **完了条件**: `cargo build` が通る
  - **見積**: 15分

- [X] P0-T5 IssueFormState 構造体を追加 in `src/app.rs`
  - **詳細**: Issue 作成フォームの状態を管理する構造体
  ```rust
  #[derive(Debug, Clone)]
  pub struct IssueFormState {
      pub title: String,
      pub body: String,
      pub focused_field: FormField,
  }
  
  #[derive(Debug, Clone, PartialEq)]
  pub enum FormField {
      Title,
      Body,
  }
  
  impl Default for IssueFormState {
      fn default() -> Self {
          Self {
              title: String::new(),
              body: String::new(),
              focused_field: FormField::Title,
          }
      }
  }
  ```
  - **ファイル**: `src/app.rs` の Screen enum の後に追加
  - **依存**: なし
  - **完了条件**: `cargo build` が通る
  - **見積**: 30分

- [X] P0-T6 App 構造体に新規フィールドを追加 in `src/app.rs`
  - **詳細**: リポジトリ選択、画面管理、エラー表示のフィールドを追加
  ```rust
  use crate::models::repository::Repository;
  
  pub struct App {
      // === 既存フィールド ===
      pub octocrab: Octocrab,
      pub issues: Vec<Issue>,
      pub list_state: ListState,
      
      // === 新規フィールド ===
      pub current_screen: Screen,
      pub selected_repository: Option<Repository>,
      pub repositories: Vec<Repository>,
      pub repo_list_state: ListState,
      pub issue_form: Option<IssueFormState>,
      pub error_message: Option<String>,
  }
  ```
  - **ファイル**: `src/app.rs` の `App` 構造体
  - **注意**: 既存の3フィールドは変更せず、6つの新規フィールドを追加
  - **依存**: P0-T2, P0-T4, P0-T5
  - **完了条件**: `cargo build` が通る
  - **見積**: 30分

- [X] P0-T7 App::new を更新して新規フィールドを初期化 in `src/app.rs`
  - **詳細**: コンストラクタで新規フィールドをデフォルト値で初期化
  ```rust
  impl App {
      pub fn new(octocrab: Octocrab, issues: Vec<Issue>) -> Self {
          let mut list_state = ListState::default();
          if !issues.is_empty() {
              list_state.select(Some(0));
          }
          Self {
              octocrab,
              issues,
              list_state,
              current_screen: Screen::IssueList,
              selected_repository: None,
              repositories: Vec::new(),
              repo_list_state: ListState::default(),
              issue_form: None,
              error_message: None,
          }
      }
      // ... 既存メソッドは維持
  }
  ```
  - **ファイル**: `src/app.rs` の `App::new` メソッド
  - **注意**: 既存の `next()`, `previous()`, `selected_issue()` メソッドは変更しない
  - **依存**: P0-T6
  - **完了条件**: `cargo build` が通る、既存の動作が正常
  - **見積**: 30分

- [X] P0-T8 [P] App にヘルパーメソッドを追加 in `src/app.rs`
  - **詳細**: エラー管理とリポジトリ選択のヘルパー
  ```rust
  impl App {
      // ... 既存メソッド ...
      
      pub fn set_error(&mut self, msg: String) {
          self.error_message = Some(msg);
      }
      
      pub fn clear_error(&mut self) {
          self.error_message = None;
      }
      
      pub fn select_repository(&mut self, repo: Repository) {
          self.selected_repository = Some(repo);
      }
      
      pub fn next_repo(&mut self) {
          let i = match self.repo_list_state.selected() {
              Some(i) => {
                  if i >= self.repositories.len().saturating_sub(1) {
                      self.repositories.len().saturating_sub(1)
                  } else {
                      i + 1
                  }
              }
              None => 0,
          };
          self.repo_list_state.select(Some(i));
      }
      
      pub fn previous_repo(&mut self) {
          let i = match self.repo_list_state.selected() {
              Some(i) => {
                  if i == 0 {
                      0
                  } else {
                      i - 1
                  }
              }
              None => 0,
          };
          self.repo_list_state.select(Some(i));
      }
      
      pub fn selected_repository_item(&self) -> Option<&Repository> {
          self.repo_list_state.selected()
              .and_then(|i| self.repositories.get(i))
      }
  }
  ```
  - **ファイル**: `src/app.rs` の `impl App` ブロック末尾
  - **依存**: P0-T7
  - **完了条件**: すべてのヘルパーメソッドが実装され、コンパイルが通る
  - **見積**: 1時間

### GitHub API 統合

- [X] P0-T9 [P] fetch_repositories 関数を実装 in `src/gh.rs`
  - **詳細**: 認証ユーザーのプライベートリポジトリ一覧を取得
  ```rust
  use crate::models::repository::Repository;
  use anyhow::Result;
  use octocrab::Octocrab;
  
  pub async fn fetch_repositories(octocrab: &Octocrab) -> Result<Vec<Repository>> {
      let mut all_repos = Vec::new();
      let mut page = octocrab
          .current()
          .list_repos_for_authenticated_user()
          .per_page(100)
          .send()
          .await?;
      
      loop {
          all_repos.extend(
              page.items
                  .into_iter()
                  .filter(|r| r.private == Some(true))
                  .map(Repository::from)
          );
          
          page = match octocrab.get_page(&page.next).await? {
              Some(next_page) => next_page,
              None => break,
          };
      }
      
      Ok(all_repos)
  }
  ```
  - **ファイル**: `src/gh.rs` の末尾に追加
  - **注意**: ページネーションで全リポジトリを取得、プライベートのみフィルタ
  - **依存**: P0-T3
  - **完了条件**: 関数が実装され、コンパイルが通る
  - **見積**: 1時間

- [X] P0-T10 [P] fetch_issues_for_repo 関数を実装 in `src/gh.rs`
  - **詳細**: 特定リポジトリの Issue 一覧を取得
  ```rust
  use octocrab::models::issues::Issue;
  
  pub async fn fetch_issues_for_repo(
      octocrab: &Octocrab,
      owner: &str,
      repo: &str,
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
  - **ファイル**: `src/gh.rs` の末尾に追加
  - **注意**: オープンな Issue のみ取得（既存の動作に合わせる）
  - **依存**: なし（既存の Octocrab インポート利用）
  - **完了条件**: 関数が実装され、コンパイルが通る
  - **見積**: 45分

- [X] P0-T11 [P] create_issue 関数を実装 in `src/gh.rs`
  - **詳細**: 新規 Issue を GitHub に作成
  ```rust
  pub async fn create_issue(
      octocrab: &Octocrab,
      owner: &str,
      repo: &str,
      title: &str,
      body: &str,
  ) -> Result<Issue> {
      let issue = octocrab
          .issues(owner, repo)
          .create(title)
          .body(body)
          .send()
          .await?;
      
      Ok(issue)
  }
  ```
  - **ファイル**: `src/gh.rs` の末尾に追加
  - **依存**: なし
  - **完了条件**: 関数が実装され、コンパイルが通る
  - **見積**: 30分

**Checkpoint (Phase 0)**: 基盤完成。以降の Phase でユーザーストーリーの実装を開始できます。`cargo build` と `cargo test` が通ることを確認してください。

---

## Phase 1: ユーザーストーリー 1 - リポジトリリストの閲覧 (優先度: P1)

**Goal**: アクセス可能なすべてのプライベートリポジトリを表示し、1つを選択できる

**独立したテスト**: アプリを起動し、'r' キーを押して、リポジトリ名、説明、スター数を表示するナビゲート可能なリストが表示されることを確認

### UI 実装

- [X] P1-T1 [US1] render_repo_selector 関数を実装 in `src/ui.rs`
  - **詳細**: リポジトリ選択画面の UI 描画ロジック
  ```rust
  use ratatui::{
      layout::{Constraint, Direction, Layout},
      style::{Color, Modifier, Style},
      text::{Line, Span},
      widgets::{Block, Borders, List, ListItem, Paragraph},
      Frame,
  };
  use crate::app::App;
  
  fn render_repo_selector(f: &mut Frame, app: &mut App) {
      let main_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([
              Constraint::Length(1), // ヘッダー
              Constraint::Min(0),    // メインエリア
              Constraint::Length(1), // ヘルプバー
          ])
          .split(f.size());
      
      // ヘッダー
      let header = Paragraph::new("Select Repository                        [Esc] Cancel")
          .style(Style::default().bg(Color::DarkGray));
      f.render_widget(header, main_chunks[0]);
      
      // メインエリア: 左右分割
      let content_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
          .split(main_chunks[1]);
      
      // 左側: リポジトリリスト
      let items: Vec<ListItem> = app
          .repositories
          .iter()
          .map(|r| ListItem::new(r.name.clone()))
          .collect();
      
      let list = List::new(items)
          .block(Block::default().borders(Borders::ALL).title("Repositories"))
          .highlight_style(
              Style::default()
                  .bg(Color::DarkGray)
                  .add_modifier(Modifier::BOLD)
          )
          .highlight_symbol(">> ");
      f.render_stateful_widget(list, content_chunks[0], &mut app.repo_list_state);
      
      // 右側: リポジトリ詳細
      if let Some(repo) = app.selected_repository_item() {
          let detail_text = vec![
              Line::from(vec![
                  Span::styled("Repository: ", Style::default().add_modifier(Modifier::BOLD)),
                  Span::raw(&repo.name),
              ]),
              Line::from(""),
              Line::from(vec![
                  Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
                  Span::raw(repo.description.as_deref().unwrap_or("N/A")),
              ]),
              Line::from(""),
              Line::from(vec![
                  Span::styled("Stars: ", Style::default().add_modifier(Modifier::BOLD)),
                  Span::raw(format!("⭐ {}", repo.stars)),
              ]),
              Line::from(""),
              Line::from(vec![
                  Span::styled("Private: ", Style::default().add_modifier(Modifier::BOLD)),
                  Span::raw(if repo.private { "Yes" } else { "No" }),
              ]),
          ];
          
          let detail = Paragraph::new(detail_text)
              .block(Block::default().borders(Borders::ALL).title("Details"));
          f.render_widget(detail, content_chunks[1]);
      }
      
      // ヘルプバー
      let help = Paragraph::new("j/k: Navigate | Enter: Select | Esc: Cancel")
          .style(Style::default().bg(Color::Blue).fg(Color::White));
      f.render_widget(help, main_chunks[2]);
  }
  ```
  - **ファイル**: `src/ui.rs` の末尾に追加
  - **注意**: 左側40%にリスト、右側60%に詳細を表示
  - **依存**: P0-T8
  - **完了条件**: 関数が実装され、コンパイルが通る
  - **見積**: 2時間

- [X] P1-T2 [US1] ui 関数を更新して画面分岐を追加 in `src/ui.rs`
  - **詳細**: `app.current_screen` に基づいて適切な描画関数を呼び出す
  ```rust
  pub fn ui(f: &mut Frame, app: &mut App) {
      // エラーバーを最初にチェック（後のタスクで実装）
      if let Some(err) = &app.error_message {
          render_error_bar(f, err);
          return; // エラー表示中は他の画面を描画しない
      }
      
      match app.current_screen {
          crate::app::Screen::IssueList => render_issue_list_original(f, app),
          crate::app::Screen::RepositorySelector => render_repo_selector(f, app),
          crate::app::Screen::IssueForm => {
              // Phase 2 で実装
              render_issue_list_original(f, app); // 暫定
          }
      }
  }
  
  // 既存の ui 関数の中身を rename
  fn render_issue_list_original(f: &mut Frame, app: &mut App) {
      // ... 既存の描画コードをそのまま移動 ...
  }
  ```
  - **ファイル**: `src/ui.rs` の `pub fn ui` 関数
  - **注意**: 既存の UI ロジックを `render_issue_list_original` に名前変更して移動
  - **依存**: P1-T1
  - **完了条件**: 画面分岐が動作、既存の Issue リスト表示が正常
  - **見積**: 1時間

- [X] P1-T3 [US1] 空のリポジトリリスト用の Empty State を実装 in `src/ui.rs`
  - **詳細**: リポジトリが0件の場合のメッセージ表示
  ```rust
  fn render_repo_selector(f: &mut Frame, app: &mut App) {
      // ... 既存コード ...
      
      // 左側: リポジトリリスト or Empty State
      if app.repositories.is_empty() {
          let empty_msg = Paragraph::new(vec![
              Line::from(""),
              Line::from("No private repositories found."),
              Line::from(""),
              Line::from("Please check your GitHub access permissions"),
              Line::from("or run `gh auth login`."),
          ])
          .block(Block::default().borders(Borders::ALL).title("Repositories"))
          .style(Style::default().fg(Color::Yellow));
          
          f.render_widget(empty_msg, content_chunks[0]);
      } else {
          // ... 既存のリスト描画コード ...
      }
  }
  ```
  - **ファイル**: `src/ui.rs` の `render_repo_selector` 関数内
  - **依存**: P1-T1
  - **完了条件**: リポジトリが0件の場合に適切なメッセージが表示される
  - **見積**: 30分

### イベントハンドリング

- [X] P1-T4 [US1] 'r' キーでリポジトリ選択画面に遷移 in `src/main.rs`
  - **詳細**: メインループに `'r'` キーのハンドラーを追加
  ```rust
  // run_app() 関数内の match event 部分に追加
  KeyCode::Char('r') => {
      // リポジトリ選択画面に遷移
      app.current_screen = crate::app::Screen::RepositorySelector;
      
      // リポジトリ一覧を取得
      match gh::fetch_repositories(&app.octocrab).await {
          Ok(repos) => {
              app.repositories = repos;
              if !app.repositories.is_empty() {
                  app.repo_list_state.select(Some(0));
              }
          }
          Err(e) => {
              app.set_error(format!("Failed to fetch repositories: {}", e));
          }
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、`match key.code` ブロック
  - **注意**: 既存の `'q'`, `'j'`, `'k'`, `'e'` キーの処理と並列に追加
  - **依存**: P0-T9, P1-T2
  - **完了条件**: `'r'` キーでリポジトリ選択画面が表示される
  - **見積**: 45分

- [X] P1-T5 [US1] リポジトリリストでの j/k ナビゲーション in `src/main.rs`
  - **詳細**: リポジトリ選択画面で j/k キーが動作するように条件分岐
  ```rust
  // run_app() 関数内の KeyCode::Char('j') と KeyCode::Char('k') 部分を更新
  KeyCode::Char('j') | KeyCode::Down => {
      match app.current_screen {
          crate::app::Screen::IssueList => app.next(),
          crate::app::Screen::RepositorySelector => app.next_repo(),
          crate::app::Screen::IssueForm => {
              // Phase 2 で実装
          }
      }
  }
  KeyCode::Char('k') | KeyCode::Up => {
      match app.current_screen {
          crate::app::Screen::IssueList => app.previous(),
          crate::app::Screen::RepositorySelector => app.previous_repo(),
          crate::app::Screen::IssueForm => {
              // Phase 2 で実装
          }
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内
  - **注意**: 既存の `app.next()`, `app.previous()` を `Screen::IssueList` の場合のみに限定
  - **依存**: P0-T8, P1-T4
  - **完了条件**: リポジトリ選択画面で j/k キーが動作する
  - **見積**: 30分

- [X] P1-T6 [US1] Esc キーでリポジトリ選択をキャンセル in `src/main.rs`
  - **詳細**: リポジトリ選択画面で Esc キーを押すと Issue リストに戻る
  ```rust
  KeyCode::Esc => {
      match app.current_screen {
          crate::app::Screen::RepositorySelector => {
              app.current_screen = crate::app::Screen::IssueList;
              // 選択状態はクリアしない（選択中のリポジトリを維持）
          }
          crate::app::Screen::IssueForm => {
              // Phase 2 で実装
          }
          _ => {}
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、新規に `KeyCode::Esc` ケースを追加
  - **依存**: P1-T4
  - **完了条件**: Esc キーで Issue リストに戻る、選択リポジトリは変更されない
  - **見積**: 20分

- [X] P1-T7 [US1] Enter キーでリポジトリを選択確定 in `src/main.rs`
  - **詳細**: リポジトリ選択画面で Enter キーを押すと選択を確定し、Issue を取得
  ```rust
  KeyCode::Enter => {
      match app.current_screen {
          crate::app::Screen::RepositorySelector => {
              if let Some(repo) = app.selected_repository_item() {
                  let repo = repo.clone();
                  app.select_repository(repo.clone());
                  
                  // 選択リポジトリの Issue を取得
                  match gh::fetch_issues_for_repo(&app.octocrab, &repo.owner, &repo.repo).await {
                      Ok(issues) => {
                          app.issues = issues;
                          if !app.issues.is_empty() {
                              app.list_state.select(Some(0));
                          } else {
                              app.list_state.select(None);
                          }
                          app.current_screen = crate::app::Screen::IssueList;
                      }
                      Err(e) => {
                          app.set_error(format!("Failed to fetch issues: {}", e));
                          // エラー時もリポジトリは選択状態にする（再試行可能）
                          app.current_screen = crate::app::Screen::IssueList;
                      }
                  }
              }
          }
          crate::app::Screen::IssueForm => {
              // Phase 2 で実装
          }
          _ => {}
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、新規に `KeyCode::Enter` ケースを追加
  - **注意**: Issue 取得失敗時もリポジトリ選択は保持し、ユーザーが再試行できるようにする
  - **依存**: P0-T10, P1-T4
  - **完了条件**: Enter キーでリポジトリが選択され、Issue リストが更新される
  - **見積**: 1時間

**Checkpoint (Phase 1)**: ユーザーストーリー 1 完成。`cargo run` でアプリを起動し、'r' キーでリポジトリ選択、j/k でナビゲーション、Enter で選択、Esc でキャンセルが動作することを確認してください。

---

## Phase 2: ユーザーストーリー 2 - リポジトリスコープの Issue 表示 (優先度: P1)

**Goal**: リポジトリを選択し、そのリポジトリに属する Issue のみを表示する

**独立したテスト**: リストからリポジトリを選択し、Enter を押して、そのリポジトリの Issue のみが Issue リストに表示されることを確認

### UI 拡張

- [X] P2-T1 [US2] render_issue_list_original を拡張してヘッダーを追加 in `src/ui.rs`
  - **詳細**: 選択リポジトリ名をヘッダーに表示
  ```rust
  fn render_issue_list_original(f: &mut Frame, app: &mut App) {
      let main_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([
              Constraint::Length(1), // ヘッダー（新規追加）
              Constraint::Min(0),    // メインエリア（既存）
              Constraint::Length(1), // ステータスバー（既存）
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
      
      // メインエリア: 既存の左右分割コードをそのまま使用
      // ... 既存の Issue リスト + 詳細表示コード ...
      // ただし、main_chunks[1] を使用するように変更
      
      // ステータスバー: 既存のヘルプテキスト
      // main_chunks[2] を使用
  }
  ```
  - **ファイル**: `src/ui.rs` の `render_issue_list_original` 関数
  - **注意**: 既存のレイアウトに1行のヘッダーを追加。メインエリアとステータスバーは既存コードを維持
  - **依存**: P1-T7
  - **完了条件**: Issue リスト画面にヘッダーが表示され、選択リポジトリ名が正しく表示される
  - **見積**: 1時間

- [X] P2-T2 [US2] 空の Issue リスト用の Empty State を実装 in `src/ui.rs`
  - **詳細**: リポジトリが選択されているが Issue が0件の場合のメッセージ
  ```rust
  fn render_issue_list_original(f: &mut Frame, app: &mut App) {
      // ... ヘッダー描画 ...
      
      // メインエリア
      let content_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
          .split(main_chunks[1]);
      
      // 左側: Issue リスト or Empty State
      if app.issues.is_empty() {
          let empty_msg = if app.selected_repository.is_some() {
              "No open issues in this repository."
          } else {
              "No issues assigned to you."
          };
          
          let empty = Paragraph::new(vec![
              Line::from(""),
              Line::from(empty_msg),
              Line::from(""),
          ])
          .block(Block::default().borders(Borders::ALL).title("Issues"))
          .style(Style::default().fg(Color::Yellow));
          
          f.render_widget(empty, content_chunks[0]);
      } else {
          // ... 既存の Issue リスト描画 ...
      }
      
      // 右側: 詳細 or Empty
      // ... 既存コード ...
  }
  ```
  - **ファイル**: `src/ui.rs` の `render_issue_list_original` 関数
  - **注意**: リポジトリ選択時と未選択時で異なるメッセージを表示
  - **依存**: P2-T1
  - **完了条件**: Issue が0件の場合に適切なメッセージが表示される
  - **見積**: 30分

- [X] P2-T3 [US2] ヘルプバーに 'n' キーのヒントを追加 in `src/ui.rs`
  - **詳細**: ステータスバーに Issue 作成のキーバインドを追加
  ```rust
  fn render_issue_list_original(f: &mut Frame, app: &mut App) {
      // ... ヘッダー、メインエリア描画 ...
      
      // ステータスバー
      let help_text = if app.selected_repository.is_some() {
          "q: 終了 | e: 編集 | j/k: 移動 | r: リポジトリ選択 | n: 新規Issue"
      } else {
          "q: 終了 | e: 編集 | j/k: 移動 | r: リポジトリ選択"
      };
      let status_bar = Paragraph::new(help_text)
          .style(Style::default().bg(Color::Blue).fg(Color::White));
      f.render_widget(status_bar, main_chunks[2]);
  }
  ```
  - **ファイル**: `src/ui.rs` の `render_issue_list_original` 関数
  - **注意**: リポジトリ選択時のみ 'n' キーのヒントを表示
  - **依存**: P2-T1
  - **完了条件**: ステータスバーに適切なヘルプテキストが表示される
  - **見積**: 20分

**Checkpoint (Phase 2)**: ユーザーストーリー 2 完成。リポジトリ選択後、ヘッダーに "Repository: owner/repo" が表示され、Issue リストがフィルタリングされることを確認してください。

---

## Phase 3: ユーザーストーリー 3 - 選択したリポジトリでの Issue 作成 (優先度: P2)

**Goal**: 現在選択されているリポジトリに新しい Issue を作成する

**独立したテスト**: リポジトリを選択し、'n' を押し、Issue フォーム（タイトルと本文）に記入し、Issue が GitHub に表示されることを確認

### Issue フォーム UI 実装

- [ ] P3-T1 [US3] render_issue_form 関数を実装 in `src/ui.rs`
  - **詳細**: Issue 作成フォームの UI 描画
  ```rust
  fn render_issue_form(f: &mut Frame, app: &mut App) {
      let main_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([
              Constraint::Length(1),  // ヘッダー
              Constraint::Length(3),  // Title フィールド
              Constraint::Length(1),  // "Body:" ラベル
              Constraint::Min(5),     // Body フィールド
              Constraint::Length(1),  // ヘルプバー
          ])
          .split(f.size());
      
      // ヘッダー
      let repo_name = app
          .selected_repository
          .as_ref()
          .map(|r| r.name.as_str())
          .unwrap_or("unknown");
      let header = Paragraph::new(format!("Create Issue: {}                [Esc] Cancel", repo_name))
          .style(Style::default().bg(Color::DarkGray));
      f.render_widget(header, main_chunks[0]);
      
      // Title フィールド
      if let Some(form) = &app.issue_form {
          let title_style = if form.focused_field == crate::app::FormField::Title {
              Style::default().fg(Color::Blue)
          } else {
              Style::default()
          };
          
          let title_block = Block::default()
              .borders(Borders::ALL)
              .title("Title")
              .border_style(title_style);
          let title_input = Paragraph::new(form.title.as_str())
              .block(title_block);
          f.render_widget(title_input, main_chunks[1]);
          
          // "Body:" ラベル
          let body_label = Paragraph::new("Body:");
          f.render_widget(body_label, main_chunks[2]);
          
          // Body フィールド
          let body_style = if form.focused_field == crate::app::FormField::Body {
              Style::default().fg(Color::Blue)
          } else {
              Style::default()
          };
          
          let body_block = Block::default()
              .borders(Borders::ALL)
              .border_style(body_style);
          let body_input = Paragraph::new(form.body.as_str())
              .block(body_block);
          f.render_widget(body_input, main_chunks[3]);
      }
      
      // ヘルプバー
      let help = Paragraph::new("Tab: Switch Field | Enter: Submit | Ctrl+E: External Editor | Esc: Cancel")
          .style(Style::default().bg(Color::Blue).fg(Color::White));
      f.render_widget(help, main_chunks[4]);
  }
  ```
  - **ファイル**: `src/ui.rs` の末尾に追加
  - **注意**: フォーカス中のフィールドは青色の境界で表示
  - **依存**: P0-T5
  - **完了条件**: Issue フォーム画面が描画される、フォーカス表示が正しい
  - **見積**: 2時間

- [ ] P3-T2 [US3] ui 関数の IssueForm 分岐を実装 in `src/ui.rs`
  - **詳細**: `ui()` 関数の `Screen::IssueForm` ケースを更新
  ```rust
  pub fn ui(f: &mut Frame, app: &mut App) {
      // エラーバーチェック
      if let Some(err) = &app.error_message {
          render_error_bar(f, err);
          return;
      }
      
      match app.current_screen {
          crate::app::Screen::IssueList => render_issue_list_original(f, app),
          crate::app::Screen::RepositorySelector => render_repo_selector(f, app),
          crate::app::Screen::IssueForm => render_issue_form(f, app), // 更新
      }
  }
  ```
  - **ファイル**: `src/ui.rs` の `ui` 関数
  - **依存**: P3-T1
  - **完了条件**: `Screen::IssueForm` 時に正しい画面が表示される
  - **見積**: 10分

### Issue フォーム イベントハンドリング

- [ ] P3-T3 [US3] 'n' キーで Issue 作成フォームを開く in `src/main.rs`
  - **詳細**: Issue リスト画面で 'n' キーを押すとフォームを開く
  ```rust
  KeyCode::Char('n') => {
      if app.current_screen == crate::app::Screen::IssueList {
          if app.selected_repository.is_none() {
              app.set_error("Please select a repository first (press 'r').".to_string());
          } else {
              app.current_screen = crate::app::Screen::IssueForm;
              app.issue_form = Some(crate::app::IssueFormState::default());
          }
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、`match key.code` ブロック
  - **注意**: リポジトリが選択されていない場合はエラーメッセージを表示
  - **依存**: P2-T3, P3-T2
  - **完了条件**: 'n' キーでフォームが開く、リポジトリ未選択時はエラーが表示される
  - **見積**: 30分

- [ ] P3-T4 [US3] Tab キーでフォームフィールドを切り替え in `src/main.rs`
  - **詳細**: Title ⇄ Body のフォーカスを切り替え
  ```rust
  KeyCode::Tab => {
      if app.current_screen == crate::app::Screen::IssueForm {
          if let Some(form) = &mut app.issue_form {
              form.focused_field = match form.focused_field {
                  crate::app::FormField::Title => crate::app::FormField::Body,
                  crate::app::FormField::Body => crate::app::FormField::Title,
              };
          }
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、新規に `KeyCode::Tab` ケースを追加
  - **依存**: P3-T3
  - **完了条件**: Tab キーでフォーカスが切り替わる
  - **見積**: 20分

- [ ] P3-T5 [US3] 文字入力とバックスペースの処理 in `src/main.rs`
  - **詳細**: フォーカス中のフィールドに文字を入力、バックスペースで削除
  ```rust
  // Char イベントの処理を拡張
  KeyCode::Char(c) => {
      if app.current_screen == crate::app::Screen::IssueForm {
          if let Some(form) = &mut app.issue_form {
              match form.focused_field {
                  crate::app::FormField::Title => {
                      if c != 'q' && c != 'n' && c != 'r' && c != 'e' {
                          form.title.push(c);
                      }
                  }
                  crate::app::FormField::Body => {
                      if c != 'q' && c != 'n' && c != 'r' && c != 'e' {
                          form.body.push(c);
                      }
                  }
              }
          }
      } else {
          // 既存のキーハンドリング ('q', 'j', 'k', 'e', 'r', 'n')
          match c {
              'q' => return Ok(()),
              'j' => { /* ... 既存コード ... */ }
              // ... その他のキー ...
          }
      }
  }
  
  KeyCode::Backspace => {
      if app.current_screen == crate::app::Screen::IssueForm {
          if let Some(form) = &mut app.issue_form {
              match form.focused_field {
                  crate::app::FormField::Title => {
                      form.title.pop();
                  }
                  crate::app::FormField::Body => {
                      form.body.pop();
                  }
              }
          }
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、`KeyCode::Char` と `KeyCode::Backspace` ケース
  - **注意**: フォーム入力中は通常のキーバインド（q, j, k, e, r, n）を無効化
  - **依存**: P3-T3
  - **完了条件**: フォームに文字入力できる、バックスペースで削除できる
  - **見積**: 45分

- [ ] P3-T6 [US3] Esc キーで Issue フォームをキャンセル in `src/main.rs`
  - **詳細**: Issue フォーム画面で Esc を押すと Issue リストに戻る
  ```rust
  KeyCode::Esc => {
      match app.current_screen {
          crate::app::Screen::RepositorySelector => {
              app.current_screen = crate::app::Screen::IssueList;
          }
          crate::app::Screen::IssueForm => {
              app.current_screen = crate::app::Screen::IssueList;
              app.issue_form = None; // フォームの内容を破棄
          }
          _ => {}
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、既存の `KeyCode::Esc` ケースを更新
  - **依存**: P3-T3
  - **完了条件**: Esc キーでフォームが閉じ、入力内容が破棄される
  - **見積**: 15分

- [ ] P3-T7 [US3] Enter キーで Issue を作成送信 in `src/main.rs`
  - **詳細**: Body フィールドにフォーカスがある状態で Enter を押すと Issue を作成
  ```rust
  KeyCode::Enter => {
      match app.current_screen {
          crate::app::Screen::RepositorySelector => {
              // ... 既存のリポジトリ選択コード ...
          }
          crate::app::Screen::IssueForm => {
              if let Some(form) = &app.issue_form {
                  // Title が空の場合はエラー
                  if form.title.trim().is_empty() {
                      app.set_error("Issue title is required.".to_string());
                      return Ok(());
                  }
                  
                  // Body フィールドにフォーカスがある場合のみ送信
                  if form.focused_field == crate::app::FormField::Body {
                      if let Some(repo) = &app.selected_repository {
                          let owner = repo.owner.clone();
                          let repo_name = repo.repo.clone();
                          let title = form.title.clone();
                          let body = form.body.clone();
                          
                          match gh::create_issue(&app.octocrab, &owner, &repo_name, &title, &body).await {
                              Ok(_new_issue) => {
                                  // Issue リストを再取得
                                  match gh::fetch_issues_for_repo(&app.octocrab, &owner, &repo_name).await {
                                      Ok(issues) => {
                                          app.issues = issues;
                                          if !app.issues.is_empty() {
                                              app.list_state.select(Some(0));
                                          }
                                      }
                                      Err(e) => {
                                          app.set_error(format!("Failed to refresh issues: {}", e));
                                      }
                                  }
                                  
                                  app.current_screen = crate::app::Screen::IssueList;
                                  app.issue_form = None;
                              }
                              Err(e) => {
                                  app.set_error(format!("Failed to create issue: {}", e));
                                  // フォームは保持し、ユーザーが再試行可能
                              }
                          }
                      }
                  } else {
                      // Title フィールドにフォーカスがある場合は Body に移動
                      if let Some(form) = &mut app.issue_form {
                          form.focused_field = crate::app::FormField::Body;
                      }
                  }
              }
          }
          _ => {}
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、既存の `KeyCode::Enter` ケースを更新
  - **注意**: Title が空の場合はバリデーションエラー。Issue 作成失敗時はフォームを保持して再試行可能に
  - **依存**: P0-T11, P0-T10, P3-T3
  - **完了条件**: Enter キーで Issue が作成され、リストが更新される
  - **見積**: 1.5時間

- [ ] P3-T8 [P] [US3] Ctrl+E で外部エディタを起動 in `src/main.rs`
  - **詳細**: Body フィールドの編集を外部エディタで行う（既存関数を再利用）
  ```rust
  // 既存の gh::edit_with_external_editor 関数を使用
  // KeyCode の処理に追加
  use crossterm::event::{KeyCode, KeyModifiers};
  
  // match event 内に追加
  if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('e') {
      if app.current_screen == crate::app::Screen::IssueForm {
          if let Some(form) = &mut app.issue_form {
              match gh::edit_with_external_editor(&form.body) {
                  Ok(edited_body) => {
                      form.body = edited_body;
                  }
                  Err(e) => {
                      app.set_error(format!("Editor failed: {}", e));
                  }
              }
          }
      }
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数内、イベントハンドリング部分
  - **注意**: 既存の `gh::edit_with_external_editor` 関数が body の編集に対応していることを確認
  - **依存**: P3-T3
  - **完了条件**: Ctrl+E で外部エディタが開き、編集内容がフォームに反映される
  - **見積**: 45分

**Checkpoint (Phase 3)**: ユーザーストーリー 3 完成。`cargo run` でアプリを起動し、リポジトリを選択後 'n' キーでフォームが開き、Issue が作成できることを確認してください。

---

## Phase 4: エラーハンドリングとポリッシュ (優先度: P3)

**Goal**: ユーザーフレンドリーなエラー表示と UX 改善

### エラー表示

- [ ] P4-T1 [P] render_error_bar 関数を実装 in `src/ui.rs`
  - **詳細**: 画面上部に赤いエラーバーを表示
  ```rust
  fn render_error_bar(f: &mut Frame, error: &str) {
      let chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Length(1), Constraint::Min(0)])
          .split(f.size());
      
      let error_bar = Paragraph::new(format!("❌ ERROR: {}    [Press any key to dismiss]", error))
          .style(Style::default().bg(Color::Red).fg(Color::White));
      f.render_widget(error_bar, chunks[0]);
      
      // エラー表示中は他の画面を描画しない
      // （ui() 関数で early return する）
  }
  ```
  - **ファイル**: `src/ui.rs` の末尾に追加
  - **注意**: エラー表示中は他の UI 要素を表示しない
  - **依存**: なし
  - **完了条件**: エラーバーが実装され、コンパイルが通る
  - **見積**: 30分

- [ ] P4-T2 任意のキー押下でエラーをクリア in `src/main.rs`
  - **詳細**: エラー表示中に任意のキーを押すとエラーを消去
  ```rust
  // run_app() 関数の最初でチェック
  if app.error_message.is_some() {
      // 任意のキー押下でエラーをクリア
      if let Event::Key(_) = event {
          app.clear_error();
      }
      continue; // エラー表示中は他のイベント処理をスキップ
  }
  ```
  - **ファイル**: `src/main.rs` の `run_app()` 関数、イベントループの最初
  - **依存**: P4-T1
  - **完了条件**: エラー表示中に任意のキーを押すとエラーが消える
  - **見積**: 20分

### エラーメッセージの改善

- [ ] P4-T3 [P] GitHub API エラーメッセージを改善 in `src/main.rs`
  - **詳細**: API エラーをユーザーフレンドリーなメッセージに変換
  ```rust
  fn format_api_error(e: &anyhow::Error) -> String {
      let err_str = e.to_string();
      
      if err_str.contains("401") || err_str.contains("Unauthorized") {
          "Authentication failed. Please run `gh auth login` to re-authenticate.".to_string()
      } else if err_str.contains("403") || err_str.contains("rate limit") {
          "GitHub API rate limit exceeded. Please try again later.".to_string()
      } else if err_str.contains("404") || err_str.contains("Not Found") {
          "Repository or resource not found. Please check your access permissions.".to_string()
      } else if err_str.contains("422") {
          format!("Invalid input: {}", err_str)
      } else if err_str.contains("network") || err_str.contains("connection") {
          "Network error: Could not connect to GitHub. Check your internet connection.".to_string()
      } else {
          format!("GitHub API error: {}", err_str)
      }
  }
  ```
  - **ファイル**: `src/main.rs` の末尾にヘルパー関数として追加
  - **注意**: すべての `app.set_error()` 呼び出しで `format_api_error()` を使用するように更新
  - **依存**: P4-T1
  - **完了条件**: エラーメッセージが分かりやすく、対処法を含む
  - **見積**: 1時間

### UX 改善

- [ ] P4-T4 [P] ローディング表示を追加 in `src/ui.rs`（オプション）
  - **詳細**: API 呼び出し中にローディングメッセージを表示
  ```rust
  // App 構造体に loading フィールドを追加（オプション）
  pub struct App {
      // ... 既存フィールド ...
      pub loading: bool,
      pub loading_message: String,
  }
  
  // ui.rs に render_loading 関数を追加
  fn render_loading(f: &mut Frame, message: &str) {
      let chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Percentage(50), Constraint::Length(3), Constraint::Percentage(50)])
          .split(f.size());
      
      let loading = Paragraph::new(vec![
          Line::from(""),
          Line::from(format!("⏳ {}", message)),
          Line::from(""),
      ])
      .block(Block::default().borders(Borders::ALL))
      .style(Style::default().fg(Color::Yellow))
      .alignment(ratatui::layout::Alignment::Center);
      
      f.render_widget(loading, chunks[1]);
  }
  ```
  - **ファイル**: `src/app.rs` と `src/ui.rs`
  - **注意**: オプション機能。実装する場合は API 呼び出し前に `app.loading = true` を設定
  - **依存**: なし
  - **完了条件**: API 呼び出し中にローディング表示が出る（実装する場合）
  - **見積**: 1時間（オプション）

- [ ] P4-T5 [P] Issue 作成成功メッセージを追加 in `src/app.rs` と `src/ui.rs`（オプション）
  - **詳細**: Issue 作成成功時に緑色のメッセージを3秒間表示
  ```rust
  // App 構造体に success_message フィールドを追加
  pub struct App {
      // ... 既存フィールド ...
      pub success_message: Option<String>,
  }
  
  impl App {
      pub fn set_success(&mut self, msg: String) {
          self.success_message = Some(msg);
      }
      
      pub fn clear_success(&mut self) {
          self.success_message = None;
      }
  }
  
  // ui.rs に成功メッセージの描画を追加
  pub fn ui(f: &mut Frame, app: &mut App) {
      // エラーバーチェック
      if let Some(err) = &app.error_message {
          render_error_bar(f, err);
          return;
      }
      
      // 成功メッセージバー
      if let Some(msg) = &app.success_message {
          let chunks = Layout::default()
              .direction(Direction::Vertical)
              .constraints([Constraint::Length(1), Constraint::Min(0)])
              .split(f.size());
          
          let success_bar = Paragraph::new(format!("✅ {}", msg))
              .style(Style::default().bg(Color::Green).fg(Color::White));
          f.render_widget(success_bar, chunks[0]);
          
          // 残りの画面は通常通り描画（chunks[1] を使用）
      }
      
      // ... 既存の画面分岐コード ...
  }
  ```
  - **ファイル**: `src/app.rs` と `src/ui.rs`
  - **注意**: オプション機能。実装する場合は Issue 作成成功時に `app.set_success("Issue created successfully")` を呼び出す
  - **依存**: P3-T7
  - **完了条件**: Issue 作成成功時に緑色のメッセージが表示される（実装する場合）
  - **見積**: 45分（オプション）

### コード品質

- [ ] P4-T6 [P] Clippy の警告を修正
  - **詳細**: `cargo clippy` を実行し、すべての警告を修正
  - **ファイル**: すべての Rust ファイル
  - **注意**: 不要な clone、unused imports、match の簡素化など
  - **依存**: すべての実装タスク完了後
  - **完了条件**: `cargo clippy` が警告なしで通る
  - **見積**: 1時間

- [ ] P4-T7 [P] コードフォーマットを実行
  - **詳細**: `cargo fmt` を実行してコードを整形
  - **ファイル**: すべての Rust ファイル
  - **依存**: すべての実装タスク完了後
  - **完了条件**: `cargo fmt --check` が通る
  - **見積**: 10分

**Checkpoint (Phase 4)**: エラーハンドリングとポリッシュ完成。すべてのエッジケースで適切なエラーメッセージが表示されることを確認してください。

---

## Phase 5: テストと検証 (優先度: P4)

**Goal**: 機能の品質とテストカバレッジを確保

### 単体テスト

- [ ] P5-T1 [P] Repository 変換ロジックのテスト in `tests/unit/models_test.rs`
  - **詳細**: `From<octocrab::models::Repository>` のテスト
  ```rust
  #[cfg(test)]
  mod tests {
      use crate::models::repository::Repository;
      
      #[test]
      fn test_repository_conversion() {
          // Octocrab の Repository を作成（モック）
          // Repository::from() を呼び出し
          // フィールドが正しく変換されていることを確認
      }
      
      #[test]
      fn test_repository_name_parsing() {
          // owner/repo 形式のパースをテスト
          // エッジケース: スラッシュなし、複数スラッシュ
      }
  }
  ```
  - **ファイル**: `tests/` ディレクトリに `unit/` サブディレクトリを作成、`models_test.rs` を追加
  - **注意**: `tests/` ディレクトリと `unit/` サブディレクトリが存在しない場合は作成
  - **依存**: P0-T3
  - **完了条件**: テストが実装され、`cargo test` が通る
  - **見積**: 1時間

- [ ] P5-T2 [P] App ヘルパーメソッドのテスト in `tests/unit/app_test.rs`
  - **詳細**: `set_error`, `clear_error`, `next_repo`, `previous_repo` のテスト
  ```rust
  #[cfg(test)]
  mod tests {
      use crate::app::App;
      
      #[test]
      fn test_error_management() {
          // set_error と clear_error の動作を確認
      }
      
      #[test]
      fn test_repo_navigation() {
          // next_repo, previous_repo の境界条件をテスト
      }
  }
  ```
  - **ファイル**: `tests/unit/app_test.rs`
  - **依存**: P0-T8
  - **完了条件**: テストが実装され、`cargo test` が通る
  - **見積**: 1時間

### 統合テスト（オプション）

- [ ] P5-T3 [P] リポジトリ選択フローの統合テスト in `tests/integration/workflow_test.rs`（オプション）
  - **詳細**: リポジトリ選択から Issue フィルタリングまでの一連のフロー
  ```rust
  #[tokio::test]
  async fn test_repository_selection_workflow() {
      // 1. リポジトリ一覧を取得（モック API）
      // 2. リポジトリを選択
      // 3. Issue リストが更新されることを確認
  }
  ```
  - **ファイル**: `tests/integration/workflow_test.rs`
  - **注意**: GitHub API のモックが必要。実装が複雑な場合はスキップ可能
  - **依存**: すべての実装タスク完了後
  - **完了条件**: 統合テストが実装され、`cargo test` が通る（実装する場合）
  - **見積**: 2時間（オプション）

### 手動テスト

- [ ] P5-T4 仕様書の受け入れシナリオをすべて手動テスト
  - **詳細**: `spec.md` のすべての受け入れシナリオを実行
  - **テストケース**:
    1. **US1-1**: 'r' キーでリポジトリリストが表示される
    2. **US1-2**: 'j' または下矢印で次のリポジトリに移動
    3. **US1-3**: 'k' または上矢印で前のリポジトリに移動
    4. **US1-4**: 右パネルに選択リポジトリの詳細が表示される
    5. **US1-5**: 'Esc' でメイン Issue 画面に戻る
    6. **US2-1**: Enter でリポジトリが選択され Issue が表示される
    7. **US2-2**: ヘッダーに選択リポジトリ名が表示される
    8. **US2-3**: リポジトリ未選択時は "My Issues" が表示される
    9. **US2-4**: Issue がない場合は空状態メッセージが表示される
    10. **US2-5**: 別のリポジトリを選択すると Issue リストが更新される
    11. **US3-1**: 'n' キーで Issue 作成フォームが表示される
    12. **US3-2**: タイトルと本文を入力して送信すると Issue が作成される
    13. **US3-3**: Ctrl+E で外部エディタが開く
    14. **US3-4**: 'Esc' でフォームが閉じる
    15. **US3-5**: リポジトリ未選択時にエラーが表示される
    16. **US3-6**: Issue 作成後リストが更新される
  - **依存**: すべての実装タスク完了後
  - **完了条件**: すべての受け入れシナリオが成功
  - **見積**: 2時間

- [ ] P5-T5 成功基準 (SC-001〜SC-007) を検証
  - **詳細**: `spec.md` の成功基準をすべて測定
  - **測定項目**:
    1. **SC-001**: リポジトリ選択画面への遷移時間 < 2秒
    2. **SC-002**: 50+ リポジトリのナビゲーション < 10秒
    3. **SC-003**: Issue フィルタリング < 2秒
    4. **SC-004**: Issue 作成 < 30秒
    5. **SC-005**: すべてのキーボードショートカットが動作
    6. **SC-006**: マウス操作なしで全機能を使用可能
    7. **SC-007**: エラーメッセージが 1秒以内に表示
  - **依存**: P5-T4
  - **完了条件**: すべての成功基準を満たす
  - **見積**: 1時間

- [ ] P5-T6 エッジケースのテスト
  - **詳細**: `spec.md` のエッジケースをすべて確認
  - **テストケース**:
    1. アクセス可能なプライベートリポジトリが0件
    2. GitHub API が利用できない（ネットワークエラー）
    3. リポジトリに 100+ Issue がある（ページネーション）
    4. Issue 作成が失敗（ネットワークエラー、API エラー）
    5. GitHub トークンが無効または期限切れ
    6. リポジトリがアーカイブまたは削除された
  - **依存**: P4-T3
  - **完了条件**: すべてのエッジケースで適切なエラーメッセージが表示される
  - **見積**: 1.5時間

**Checkpoint (Phase 5)**: すべてのテストが完了。`cargo test` が通り、手動テストですべての受け入れシナリオと成功基準を満たすことを確認してください。

---

## 依存関係と実行順序

### Phase 依存関係

```
Phase 0 (基盤構築)
  ↓
Phase 1 (リポジトリリストの閲覧) [US1]
  ↓
Phase 2 (リポジトリスコープの Issue 表示) [US2]
  ↓
Phase 3 (Issue 作成) [US3]
  ↓
Phase 4 (エラーハンドリング)
  ↓
Phase 5 (テストと検証)
```

### 重要な依存関係

- **Phase 0 完了 → すべての Phase が開始可能**: データ構造と API が実装されるまで、他の Phase は開始できません
- **Phase 1 完了 → Phase 2 開始可能**: リポジトリ選択機能が動作するまで、Issue フィルタリングはテストできません
- **Phase 2 完了 → Phase 3 開始可能**: リポジトリが選択されている状態が前提なので、Issue 作成はその後に実装
- **Phase 3 完了 → Phase 4 開始**: すべての機能が実装された後、エラーハンドリングを追加
- **Phase 4 完了 → Phase 5 開始**: エラーハンドリングを含むすべての機能が完成した後、テストを実行

### ユーザーストーリーの依存関係

- **US1 → US2**: リポジトリ選択なしに Issue フィルタリングは不可能
- **US2 → US3**: リポジトリが選択されていないと Issue 作成はできない
- すべてのユーザーストーリーは **Phase 0** の完了に依存

### 並列実行の機会

以下のタスクは並列実行可能です（異なるファイル、依存関係なし）:

**Phase 0**:
- P0-T1, P0-T2, P0-T3 (モデル実装)
- P0-T4, P0-T5 (状態管理 enum)
- P0-T8 (ヘルパーメソッド)
- P0-T9, P0-T10, P0-T11 (GitHub API 関数)

**Phase 4**:
- P4-T1 (エラーバー UI)
- P4-T3 (エラーメッセージ改善)
- P4-T4, P4-T5 (UX 改善、オプション)
- P4-T6, P4-T7 (コード品質)

**Phase 5**:
- P5-T1, P5-T2, P5-T3 (すべてのテスト)

---

## 実装戦略

### MVP 優先（最小限の実装）

1. **Phase 0** を完了（基盤構築）
2. **Phase 1** を完了（リポジトリリスト閲覧）
3. **Phase 2** を完了（Issue フィルタリング）
4. **停止して検証**: ここまでで基本機能が動作します。リポジトリ選択と Issue 表示を確認
5. **Phase 3** を完了（Issue 作成）
6. **停止して検証**: フル機能が動作します
7. **Phase 4, 5** を完了（エラーハンドリングとテスト）

### 段階的デリバリー

1. **Phase 0 完了** → 基盤準備完了
2. **Phase 1 完了** → リポジトリ選択が動作 → デモ可能
3. **Phase 2 完了** → Issue フィルタリングが動作 → MVP デモ可能
4. **Phase 3 完了** → Issue 作成が動作 → フル機能デモ可能
5. **Phase 4 完了** → エラーハンドリング完成 → 本番レディ
6. **Phase 5 完了** → テスト完了 → リリース準備完了

### 推奨作業順序

1. **1日目**: Phase 0 完了（基盤構築）
2. **2-3日目**: Phase 1 完了（リポジトリ選択）
3. **4日目**: Phase 2 完了（Issue フィルタリング）→ MVP チェックポイント
4. **5-6日目**: Phase 3 完了（Issue 作成）
5. **7日目**: Phase 4 完了（エラーハンドリング）
6. **8日目**: Phase 5 完了（テスト）

**合計見積**: 8営業日

---

## 注意事項

- **既存機能の保護**: すべての既存機能（My Issues 表示、Issue 編集）は変更せず、動作を維持してください
- **リグレッション防止**: 各 Phase 完了後に `cargo build && cargo run` で既存の動作を確認してください
- **段階的コミット**: 各タスクまたは小さなタスクグループごとにコミットし、ロールバック可能にしてください
- **エラーハンドリング優先**: すべての API 呼び出しには適切なエラーハンドリングを実装してください
- **ユーザーフレンドリー**: エラーメッセージには問題の説明と対処法を含めてください

---

## 検証チェックリスト

### Phase 0 チェックポイント
- [ ] `cargo build` が通る
- [ ] 既存の `cargo run` が正常に動作（My Issues 表示）
- [ ] 新しいデータ構造（Repository, Screen, IssueFormState）が定義されている
- [ ] GitHub API 関数（fetch_repositories, fetch_issues_for_repo, create_issue）が実装されている

### Phase 1 チェックポイント
- [ ] 'r' キーでリポジトリ選択画面が表示される
- [ ] j/k キーでリポジトリリストをナビゲートできる
- [ ] Enter でリポジトリを選択できる
- [ ] Esc でキャンセルできる
- [ ] リポジトリが0件の場合に Empty State が表示される

### Phase 2 チェックポイント
- [ ] リポジトリ選択後、ヘッダーに "Repository: owner/repo" が表示される
- [ ] Issue リストが選択リポジトリの Issue のみを表示する
- [ ] Issue が0件の場合に Empty State が表示される
- [ ] ステータスバーに 'n' キーのヒントが表示される

### Phase 3 チェックポイント
- [ ] 'n' キーで Issue 作成フォームが表示される
- [ ] Tab キーでフィールドを切り替えられる
- [ ] 文字入力とバックスペースが動作する
- [ ] Esc でフォームをキャンセルできる
- [ ] Enter で Issue が作成される
- [ ] Ctrl+E で外部エディタが開く
- [ ] リポジトリ未選択時にエラーが表示される

### Phase 4 チェックポイント
- [ ] すべての API エラーで適切なメッセージが表示される
- [ ] エラーバーが画面上部に赤色で表示される
- [ ] 任意のキー押下でエラーが消える
- [ ] `cargo clippy` が警告なしで通る
- [ ] `cargo fmt --check` が通る

### Phase 5 チェックポイント
- [ ] `cargo test` がすべて通る
- [ ] 仕様書のすべての受け入れシナリオが成功
- [ ] すべての成功基準（SC-001〜SC-007）を満たす
- [ ] すべてのエッジケースが適切に処理される

---

**実装開始前**: このタスクリストを確認し、不明点があれば `spec.md`, `plan.md`, `data-model.md`, `contracts/` を参照してください。

**実装完了後**: すべてのチェックポイントを確認し、`quickstart.md` の手順で動作確認を行ってください。
