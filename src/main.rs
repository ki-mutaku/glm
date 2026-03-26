use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use octocrab::{models::issues::Issue, Octocrab};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{
    io::{self, Write},
    process::Command,
    time::{Duration, Instant},
};
use tempfile::Builder;

/// `gh auth token` コマンドを実行して GitHub トークンを取得する
fn get_github_token() -> Result<String> {
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context(
            "`gh auth token` の実行に失敗しました。GitHub CLI はインストールされていますか？",
        )?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("`gh auth token` が失敗しました: {}", err);
    }

    let token = String::from_utf8(output.stdout)
        .context("トークンが不正な UTF-8 です")?
        .trim()
        .to_string();

    Ok(token)
}

/// アプリケーションの状態を管理する構造体
struct App {
    /// GitHub API クライアント
    octocrab: Octocrab,
    /// 取得した Issue のリスト
    issues: Vec<Issue>,
    /// リストの選択状態（どの項目がハイライトされているか）
    list_state: ListState,
}

impl App {
    fn new(octocrab: Octocrab, issues: Vec<Issue>) -> Self {
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
    fn next(&mut self) {
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
    fn previous(&mut self) {
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
    fn selected_issue(&self) -> Option<&Issue> {
        self.list_state.selected().and_then(|i| self.issues.get(i))
    }

    /// 指定したインデックスの Issue 本文を更新する（メモリ上）
    fn update_issue_body(&mut self, index: usize, new_body: String) {
        if let Some(issue) = self.issues.get_mut(index) {
            issue.body = Some(new_body);
        }
    }
}

/// Issue の repository_url (https://api.github.com/repos/owner/repo) から owner と repo を抽出する
fn parse_repo_owner(url: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = url.split('/').collect();
    let len = parts.len();
    if len >= 2 {
        Some((parts[len - 2].to_string(), parts[len - 1].to_string()))
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. GitHub CLI を使った認証
    println!("`gh auth token` を使用してトークンを取得中...");
    let token = get_github_token()?;

    // 2. Octocrab (GitHub API クライアント) の初期化と Issue 取得
    println!("自分にアサインされたオープンな Issue を取得中...");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Octocrab クライアントの構築に失敗しました")?;

    let page = octocrab
        .search()
        .issues_and_pull_requests("is:issue is:open assignee:@me")
        .send()
        .await
        .context("Issue の検索に失敗しました")?;

    let issues = page.items.into_iter().collect::<Vec<_>>();

    let app = App::new(octocrab, issues);

    // 3. ターミナルのセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. アプリケーションのメインループ実行
    let res = run_app(&mut terminal, app).await;

    // 5. ターミナルの復元
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("エラーが発生しました: {:?}", err)
    }

    Ok(())
}

/// メインのイベントループ
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char('e') => {
                        // 外部エディタでの編集
                        if let Some(index) = app.list_state.selected() {
                            let issue = app.issues[index].clone();

                            // ターミナルを一時復元
                            disable_raw_mode()?;
                            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

                            let new_body =
                                edit_with_external_editor(issue.body.as_deref().unwrap_or(""))?;

                            // API を叩いて GitHub を更新
                            if let Some(body) = new_body {
                                if body != issue.body.as_deref().unwrap_or("") {
                                    if let Some((owner, repo)) =
                                        parse_repo_owner(issue.repository_url.as_str())
                                    {
                                        app.octocrab
                                            .issues(owner, repo)
                                            .update(issue.number)
                                            .body(&body)
                                            .send()
                                            .await
                                            .context("GitHub の Issue 更新に失敗しました")?;

                                        app.update_issue_body(index, body);
                                    }
                                }
                            }

                            // TUI を再開
                            enable_raw_mode()?;
                            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                            terminal.clear()?;
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

/// 外部エディタを起動して文字列を編集する
fn edit_with_external_editor(initial_content: &str) -> Result<Option<String>> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let mut temp_file = Builder::new()
        .suffix(".md")
        .tempfile()
        .context("一時ファイルの作成に失敗しました")?;

    temp_file.write_all(initial_content.as_bytes())?;
    temp_file.flush()?;

    let status = Command::new(editor)
        .arg(temp_file.path())
        .status()
        .context("エディタの起動に失敗しました")?;

    if status.success() {
        let content = std::fs::read_to_string(temp_file.path())
            .context("一時ファイルの読み込みに失敗しました")?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

/// UI の描画
fn ui(f: &mut ratatui::Frame, app: &mut App) {
    // 画面全体を上下に分割 (メインエリア : ステータス行)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.size());

    // メインエリアを左右に分割 (30% : 70%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[0]);

    // 左側のエリアをさらに上下に分割
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(chunks[0]);

    // 左上: カテゴリ表示
    let sidebar = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "● My Issues",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Inbox"),
        Line::from("  Projects"),
    ])
    .block(Block::default().borders(Borders::ALL).title("カテゴリ"));
    f.render_widget(sidebar, left_chunks[0]);

    // 左下: Issue タイトルの一覧リスト
    let items: Vec<ListItem> = app
        .issues
        .iter()
        .map(|i| {
            let title = i.title.clone();
            let state = format!("{:?}", i.state);
            let content = vec![Line::from(Span::raw(format!("[{}] {}", state, title)))];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Issue 一覧"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, left_chunks[1], &mut app.list_state);

    // 右側: 選択中の Issue の詳細表示
    let main_content = if let Some(issue) = app.selected_issue() {
        let title = format!("#{} {}", issue.number, issue.title);
        let body = issue.body.as_deref().unwrap_or("（本文なし）");
        format!("{}\n\n{}", title, body)
    } else {
        "Issue を選択してください。".to_string()
    };

    let main_panel = Paragraph::new(main_content)
        .block(Block::default().borders(Borders::ALL).title("詳細"))
        .wrap(Wrap { trim: true });
    f.render_widget(main_panel, chunks[1]);

    // ステータス行 (下部)
    let help_text = " q: 終了 | e: 編集 | j/k: 移動 ";
    let status_bar =
        Paragraph::new(help_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(status_bar, main_chunks[1]);
}
