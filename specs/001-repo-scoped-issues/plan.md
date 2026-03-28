# Implementation Plan: リポジトリスコープの Issue 管理

**Branch**: `001-repo-scoped-issues` | **Date**: 2025-01-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-repo-scoped-issues/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

GitHub Life Manager (GLM) に、リポジトリ選択機能とリポジトリスコープでの Issue 表示・作成機能を追加します。ユーザーはプライベートリポジトリ一覧から特定のリポジトリを選択し、そのリポジトリに属する Issue のみをフィルタリングして閲覧できるようになります。また、選択したリポジトリに新規 Issue を作成する機能も提供します。

技術的アプローチ：
- 状態管理に新しい Screen enum を導入し、画面遷移を明確化
- Repository 構造体を追加してリポジトリ情報を保持
- Octocrab API を使用してリポジトリ一覧と Issue 一覧を取得
- 既存の外部エディタ統合を Issue 作成フォームで再利用

## Technical Context

**Language/Version**: Rust Edition 2021  
**Primary Dependencies**: Ratatui 0.26 (TUI framework), Crossterm 0.27 (event handling), Octocrab 0.38 (GitHub API), Tokio 1.x (async runtime)  
**Storage**: N/A (メモリ内状態管理のみ)  
**Testing**: cargo test (Rust 標準テストフレームワーク)  
**Target Platform**: macOS/Linux/Windows ターミナル環境
**Project Type**: CLI/TUI アプリケーション  
**Performance Goals**: リポジトリ選択画面への遷移 <2秒、Issue フィルタリング <2秒、Issue 作成 <30秒（API レスポンス時間含む）  
**Constraints**: 100% キーボード操作、マウス不要、GitHub API レート制限内で動作  
**Scale/Scope**: 3 画面（My Issues、リポジトリ選択、Issue 作成）、50+ リポジトリ対応

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: ✅ PASS

**Rationale**: プロジェクトには明示的な constitution.md が存在しないため、デフォルトゲートを適用します。本機能は：
- 既存のシンプルな TUI アーキテクチャを維持
- 新しい外部依存を追加せず、既存の Octocrab、Ratatui、Crossterm のみを使用
- 状態管理を App 構造体内に保持し、複雑な状態管理ライブラリを導入しない
- テストは cargo test で標準的な単体テスト・統合テストを記述

**Complexity Tracking**: 複雑性違反なし

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # エントリーポイント、イベントループ (既存)
├── app.rs               # アプリケーション状態、画面管理 (拡張)
├── gh.rs                # GitHub API 統合 (拡張)
├── ui.rs                # UI 描画ロジック (拡張)
├── models/              # 新規追加
│   └── repository.rs    # Repository 構造体
└── screens/             # 新規追加 (オプション: 画面が複雑化した場合)
    ├── issues.rs        # My Issues 画面
    ├── repo_list.rs     # リポジトリ選択画面
    └── issue_form.rs    # Issue 作成フォーム画面

tests/
├── integration/         # 統合テスト (新規)
│   └── workflow_tests.rs
└── unit/                # 単体テスト (新規)
    └── gh_tests.rs
```

**Structure Decision**: 
初期実装では `src/models/` のみ追加し、UI ロジックは `ui.rs` に集約します。画面が複雑化した場合は Phase 2 で `screens/` モジュールへのリファクタリングを検討します。これにより、既存コードへの変更を最小限に抑えつつ、段階的な拡張が可能になります。

## Complexity Tracking

**Status**: 複雑性違反なし

本機能は既存のシンプルな設計を維持し、新たな複雑性を導入しません。

---

## Phase 0: Research & Clarification ✅

**Status**: COMPLETE

すべての技術的疑問点が解決され、実装に必要な情報が揃いました。

**主な成果**:
- Ratatui での複数画面管理パターンの確立 (Screen enum)
- Octocrab API の使用方法の確認 (リポジトリ一覧、Issue 取得、Issue 作成)
- エラーハンドリング戦略の策定 (トースト形式)
- UI レイアウト設計の完成

詳細は [research.md](./research.md) を参照。

---

## Phase 1: Design & Contracts ✅

**Status**: COMPLETE

**成果物**:
- ✅ **Data Model**: [data-model.md](./data-model.md) - Repository, IssueFormState, Screen enum, App 状態拡張
- ✅ **API Contract**: [contracts/api-contract.md](./contracts/api-contract.md) - GitHub API 統合仕様
- ✅ **UI Contract**: [contracts/ui-contract.md](./contracts/ui-contract.md) - 画面レイアウトとキーバインディング
- ✅ **Quickstart Guide**: [quickstart.md](./quickstart.md) - 実装者向けクイックリファレンス

**Constitution Re-Check**: ✅ PASS (Phase 1 完了後も複雑性違反なし)

---

## Phase 2: Implementation Tasks

**Note**: Phase 2 の詳細なタスク分解は `/speckit.tasks` コマンドで別途生成されます。
本セクションでは高レベルの実装フェーズのみを記載します。

### 実装優先順位

#### P0: 基盤構築 (1-2 日)

**Goal**: データモデルと API 統合の基盤を実装

1. **Data Models**
   - [ ] `src/models/repository.rs` を作成
   - [ ] `Repository` 構造体の実装
   - [ ] `From<octocrab::models::Repository>` トレイト実装

2. **State Management**
   - [ ] `src/app.rs` に `Screen` enum を追加
   - [ ] `App` 構造体に新規フィールドを追加
   - [ ] ヘルパーメソッド (`set_error`, `clear_error`, `select_repository`) を実装

3. **API Integration**
   - [ ] `src/gh.rs` に `fetch_repositories()` を実装
   - [ ] `src/gh.rs` に `fetch_issues_for_repo()` を実装
   - [ ] エラーハンドリングの統合

**Acceptance Criteria**:
- コンパイルエラーなし
- 既存の動作が正常 (リグレッションなし)
- 単体テストが通る

---

#### P1: リポジトリ選択機能 (2-3 日)

**Goal**: ユーザーストーリー 1 と 2 を完全に実装

1. **Repository Selector UI**
   - [ ] `src/ui.rs` に `render_repo_selector()` を実装
   - [ ] リポジトリリストの描画 (左側 40%)
   - [ ] リポジトリ詳細の描画 (右側 60%)
   - [ ] ヘルプバーの追加

2. **Event Handling**
   - [ ] `main.rs` の `run_app()` に `'r'` キーハンドラー追加
   - [ ] `j/k` でリポジトリリストをナビゲート
   - [ ] `Enter` でリポジトリ選択
   - [ ] `Esc` でキャンセル

3. **Scoped Issue Display**
   - [ ] `src/ui.rs` の `render_issue_list()` を拡張
   - [ ] ヘッダーに選択リポジトリ名を表示
   - [ ] リポジトリ選択時に Issue リストを再取得

**Acceptance Criteria**:
- SC-001: リポジトリ選択画面への遷移 < 2秒
- SC-002: 50+ リポジトリのナビゲーション < 10秒
- SC-003: Issue フィルタリング < 2秒
- 仕様書の受け入れシナリオ 1.1〜1.5, 2.1〜2.5 をすべて満たす

---

#### P2: Issue 作成機能 (2-3 日)

**Goal**: ユーザーストーリー 3 を実装

1. **Issue Form State**
   - [ ] `src/app.rs` に `IssueFormState` 構造体を追加
   - [ ] `FormField` enum の実装

2. **Issue Form UI**
   - [ ] `src/ui.rs` に `render_issue_form()` を実装
   - [ ] Title フィールドの描画 (1行)
   - [ ] Body フィールドの描画 (複数行)
   - [ ] フォーカス表示の実装

3. **Event Handling**
   - [ ] `'n'` キーで Issue 作成フォームを開く
   - [ ] `Tab` でフィールド切替
   - [ ] テキスト入力の処理
   - [ ] `Ctrl+E` で外部エディタ起動 (既存関数を再利用)
   - [ ] `Enter` で Issue 作成 API 呼び出し
   - [ ] `Esc` でキャンセル

4. **API Integration**
   - [ ] `src/gh.rs` に `create_issue()` を実装
   - [ ] 成功時に Issue リストを更新
   - [ ] エラーハンドリング

**Acceptance Criteria**:
- SC-004: Issue 作成 < 30秒
- SC-005: 100% キーボード操作で動作
- 仕様書の受け入れシナリオ 3.1〜3.6 をすべて満たす

---

#### P3: エラーハンドリング & Polish (1 日)

**Goal**: ユーザーフレンドリーなエラー表示と UX 改善

1. **Error Display**
   - [ ] `src/ui.rs` にエラーバー描画を追加
   - [ ] すべての API 呼び出しに適切なエラーメッセージを設定
   - [ ] エラークリア機能の実装

2. **Edge Cases**
   - [ ] 空リポジトリリストの処理 (Empty State)
   - [ ] 空 Issue リストの処理
   - [ ] リポジトリ未選択で Issue 作成試行時のエラー
   - [ ] Title が空で Issue 作成試行時のバリデーション

3. **UX Improvements**
   - [ ] ローディング表示 (オプション)
   - [ ] 成功メッセージの表示 (Issue 作成後)
   - [ ] キーバインディングヘルプの改善

**Acceptance Criteria**:
- SC-007: エラーメッセージが 1秒以内に表示
- 仕様書のエッジケースすべてに対応
- ユーザーフレンドリーなエラーメッセージ

---

#### P4: Testing (1-2 日)

**Goal**: テストカバレッジの確保

1. **Unit Tests**
   - [ ] `src/gh.rs` の API 関数のテスト (モック使用)
   - [ ] `src/app.rs` のヘルパーメソッドのテスト
   - [ ] `src/models/repository.rs` の変換ロジックのテスト

2. **Integration Tests**
   - [ ] リポジトリ選択 → Issue フィルタリングのフロー
   - [ ] Issue 作成 → リスト更新のフロー
   - [ ] エラーハンドリングのフロー

3. **Manual Testing**
   - [ ] 仕様書のすべての受け入れシナリオを手動テスト
   - [ ] 成功基準 SC-001〜SC-007 を検証

**Acceptance Criteria**:
- すべての単体テストが通る
- すべての統合テストが通る
- 仕様書のすべての受け入れシナリオが成功

---

## Architecture Details

### State Management

```rust
// src/app.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    IssueList,
    RepositorySelector,
    IssueForm,
}

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
    
    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }
    
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
    
    pub fn select_repository(&mut self, repo: Repository) {
        self.selected_repository = Some(repo);
    }
}
```

### UI Rendering Flow

```rust
// src/ui.rs

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.current_screen {
        Screen::IssueList => render_issue_list(f, app),
        Screen::RepositorySelector => render_repo_selector(f, app),
        Screen::IssueForm => render_issue_form(f, app),
    }
    
    // エラーバーは全画面で共通
    if let Some(err) = &app.error_message {
        render_error_bar(f, err);
    }
}

fn render_issue_list(f: &mut Frame, app: &mut App) {
    // ヘッダー、Issue リスト、詳細、ステータスバーを描画
    // ヘッダーに選択リポジトリ名を表示
}

fn render_repo_selector(f: &mut Frame, app: &mut App) {
    // リポジトリリスト (左) と詳細 (右) を描画
}

fn render_issue_form(f: &mut Frame, app: &mut App) {
    // Title フィールド、Body フィールド、ヘルプバーを描画
}

fn render_error_bar(f: &mut Frame, error: &str) {
    // 画面上部に赤いエラーバーを描画
}
```

### Event Flow

```
User Input (KeyCode)
  ↓
main.rs: run_app()
  ↓
match app.current_screen + key.code
  ↓
  ├─ 'r' → Screen::RepositorySelector + fetch_repositories()
  ├─ 'n' → Screen::IssueForm (if repo selected)
  ├─ Enter (on RepoSelector) → select_repository() + fetch_issues_for_repo()
  ├─ Enter (on IssueForm) → create_issue() + refresh issues
  └─ Esc → 前の画面に戻る
  ↓
App 状態更新
  ↓
terminal.draw(|f| ui::ui(f, &mut app))
  ↓
画面描画
```

---

## API Call Patterns

### 1. アプリ起動時

```rust
// main.rs
let token = gh::get_github_token()?;
let octocrab = Octocrab::builder().personal_token(token).build()?;

// 既存: "My Issues" を取得
let page = octocrab.search()
    .issues_and_pull_requests("is:issue is:open assignee:@me")
    .send()
    .await?;
let issues = page.items;

let app = App::new(octocrab, issues);
```

### 2. リポジトリ選択時

```rust
// 'r' キー押下
app.current_screen = Screen::RepositorySelector;
app.repositories = gh::fetch_repositories(&app.octocrab).await?;
app.repo_list_state.select(Some(0)); // 最初の項目を選択
```

### 3. リポジトリ確定時

```rust
// Enter キー押下 (on RepositorySelector)
if let Some(idx) = app.repo_list_state.selected() {
    let repo = app.repositories[idx].clone();
    app.select_repository(repo.clone());
    
    // 選択リポジトリの Issue を取得
    app.issues = gh::fetch_issues_for_repo(
        &app.octocrab,
        &repo.owner,
        &repo.repo
    ).await?;
    
    app.list_state.select(Some(0)); // 最初の Issue を選択
    app.current_screen = Screen::IssueList;
}
```

### 4. Issue 作成時

```rust
// 'n' キー押下
if app.selected_repository.is_none() {
    app.set_error("Please select a repository first (press 'r')".to_string());
    return;
}

app.current_screen = Screen::IssueForm;
app.issue_form = Some(IssueFormState {
    title: String::new(),
    body: String::new(),
    focused_field: FormField::Title,
});
```

### 5. Issue 送信時

```rust
// Enter キー押下 (on IssueForm, Body フィールド)
if let Some(form) = &app.issue_form {
    if form.title.is_empty() {
        app.set_error("Issue title is required".to_string());
        return;
    }
    
    if let Some(repo) = &app.selected_repository {
        match gh::create_issue(
            &app.octocrab,
            &repo.owner,
            &repo.repo,
            &form.title,
            &form.body
        ).await {
            Ok(_new_issue) => {
                // Issue リストを再取得
                app.issues = gh::fetch_issues_for_repo(
                    &app.octocrab,
                    &repo.owner,
                    &repo.repo
                ).await?;
                
                app.current_screen = Screen::IssueList;
                app.issue_form = None;
                
                // 成功メッセージ (オプション)
                // app.set_success("Issue created successfully");
            }
            Err(e) => {
                app.set_error(format!("Failed to create issue: {}", e));
                // フォームは保持し、ユーザーが再試行可能
            }
        }
    }
}
```

---

## Testing Strategy

### Unit Test Coverage

| Module | Test Target | Test Type |
|--------|-------------|-----------|
| `gh.rs` | `fetch_repositories()` | API モック |
| `gh.rs` | `fetch_issues_for_repo()` | API モック |
| `gh.rs` | `create_issue()` | API モック |
| `app.rs` | `set_error()`, `clear_error()` | 状態変更 |
| `app.rs` | `select_repository()` | 状態変更 |
| `models/repository.rs` | `From<octocrab::Repository>` | 変換ロジック |

### Integration Test Scenarios

1. **リポジトリ選択フロー**
   - 'r' → リポジトリリスト表示 → Enter → Issue リストフィルタリング

2. **Issue 作成フロー**
   - リポジトリ選択 → 'n' → タイトル入力 → Body 入力 → Enter → Issue 作成 → リスト更新

3. **エラーハンドリング**
   - API エラー → エラーメッセージ表示 → 任意のキー → エラークリア

4. **エッジケース**
   - リポジトリ未選択で 'n' → エラーメッセージ
   - Title 空で Enter → バリデーションエラー

### Manual Test Checklist

- [ ] 仕様書の受け入れシナリオ 1.1〜1.5 (リポジトリリスト)
- [ ] 仕様書の受け入れシナリオ 2.1〜2.5 (リポジトリスコープ Issue 表示)
- [ ] 仕様書の受け入れシナリオ 3.1〜3.6 (Issue 作成)
- [ ] 成功基準 SC-001〜SC-007 の検証
- [ ] エッジケースすべてのテスト

---

## Performance Considerations

### API Call Optimization

- **リポジトリリスト**: アプリ起動時に 1 回取得、メモリにキャッシュ
- **Issue リスト**: リポジトリ選択時またはリフレッシュ時のみ取得
- **ページネーション**: API レベルで実装済み (100件/ページ)

### Memory Footprint

- リポジトリ: ~200 bytes × 100 = ~20 KB
- Issue: ~500 bytes × 100 = ~50 KB
- **Total**: < 100 KB (十分軽量)

### Rendering Performance

- Ratatui は効率的な差分レンダリングを実装
- 通常のキー入力 → レンダリング: < 16ms (60 FPS 相当)

---

## Security Considerations

### Token Handling

- Token は `gh auth token` 経由で取得、メモリ上のみ保持
- ログやファイルへの Token 出力を禁止
- エラーメッセージに Token を含めない

### API Communication

- すべての通信は HTTPS (Octocrab がデフォルトで強制)
- TLS 証明書検証を有効化

---

## Migration & Rollback

### Backward Compatibility

- リポジトリ未選択時: 既存の「My Issues」動作を完全に維持
- 既存のキーバインディング (`q`, `j/k`, `e`) は変更なし
- **Breaking Changes**: なし (すべて追加機能)

### Rollback Plan

1. `git checkout main` で既存バージョンに戻す
2. 新規ファイル (`src/models/repository.rs`) を削除
3. `src/app.rs`, `src/gh.rs`, `src/ui.rs` の変更を revert

---

## Future Enhancements (Out of Scope for v1)

以下の機能は v1 のスコープ外ですが、将来の拡張候補として記録します：

- **ラベル・マイルストーン・担当者の設定** (Issue 作成時)
- **Issue 検索・フィルタリング** (タイトル、状態、ラベルでフィルタ)
- **パブリックリポジトリのサポート** (現在はプライベートのみ)
- **Issue コメント機能** (閲覧・追加)
- **Pull Request 管理** (Issue と同様の操作)
- **リアルタイム更新** (Webhooks 経由)
- **設定ファイル** (デフォルトリポジトリ、キーバインディングのカスタマイズ)

---

## Conclusion

本実装計画は Phase 0 (Research) と Phase 1 (Design) を完了し、Phase 2 (Implementation) に進む準備が整いました。

**次のステップ**:
1. `/speckit.tasks` コマンドで詳細なタスク分解を生成
2. P0 タスク (基盤構築) から実装を開始
3. 各 Phase の Acceptance Criteria を満たしながら段階的に実装

**推定期間**: 6-9 営業日  
**リスク**: 低 (既存技術スタックの範囲内、新規依存なし)
