use octocrab::{models::issues::Issue, Octocrab};
use ratatui::widgets::ListState;

use crate::models::repository::Repository;

/// 現在表示している画面を表す enum
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// My Issues 画面 (デフォルト)
    IssueList,
    /// リポジトリ選択画面
    RepositorySelector,
    /// Issue 作成フォーム画面
    IssueForm,
}

/// Issue 作成フォームの入力状態
#[derive(Debug, Clone)]
pub struct IssueFormState {
    /// Issue タイトル
    pub title: String,
    /// Issue 本文
    pub body: String,
    /// フォーカス中のフィールド
    pub focused_field: FormField,
}

/// フォーム内のフィールドを識別する enum
#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    /// タイトルフィールド
    Title,
    /// 本文フィールド
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

pub struct App {
    // === 既存フィールド ===
    /// GitHub API クライアント
    pub octocrab: Octocrab,
    /// 取得した Issue のリスト
    pub issues: Vec<Issue>,
    /// リストの選択状態（どの項目がハイライトされているか）
    pub list_state: ListState,
    
    // === 新規フィールド ===
    /// 現在表示している画面
    pub current_screen: Screen,
    /// 選択中のリポジトリ
    pub selected_repository: Option<Repository>,
    /// 取得したリポジトリのリスト
    pub repositories: Vec<Repository>,
    /// リポジトリリストの選択状態
    pub repo_list_state: ListState,
    /// Issue 作成フォームの状態
    pub issue_form: Option<IssueFormState>,
    /// エラーメッセージ（表示用）
    pub error_message: Option<String>,
}

impl App {
    pub fn new(octocrab: Octocrab, issues: Vec<Issue>) -> Self {
        let mut list_state = ListState::default();
        if !issues.is_empty() {
            // 最初の項目を選択状態にする
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

    /// 次の項目を選択する (j キー)
    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.issues.len().saturating_sub(1) {
                    self.issues.len().saturating_sub(1)
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// 前の項目を選択する (k キー)
    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// 現在選択されている Issue を取得する
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.list_state.selected().and_then(|i| self.issues.get(i))
    }

    /// 指定したインデックスの Issue 本文を更新する（メモリ上）
    pub fn update_issue_body(&mut self, index: usize, new_body: String) {
        if let Some(issue) = self.issues.get_mut(index) {
            issue.body = Some(new_body);
        }
    }
    
    /// エラーメッセージを設定
    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }
    
    /// エラーメッセージをクリア
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
    
    /// リポジトリを選択
    pub fn select_repository(&mut self, repo: Repository) {
        self.selected_repository = Some(repo);
    }
    
    /// リポジトリリストの次の項目を選択
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
    
    /// リポジトリリストの前の項目を選択
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
    
    /// 現在選択されているリポジトリを取得
    pub fn selected_repository_item(&self) -> Option<&Repository> {
        self.repo_list_state.selected()
            .and_then(|i| self.repositories.get(i))
    }
}
