use octocrab::{models::issues::Issue, Octocrab};
use ratatui::widgets::ListState;

pub struct App {
    /// GitHub API クライアント
    pub octocrab: Octocrab,
    /// 取得した Issue のリスト
    pub issues: Vec<Issue>,
    /// リストの選択状態（どの項目がハイライトされているか）
    pub list_state: ListState,
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
}
