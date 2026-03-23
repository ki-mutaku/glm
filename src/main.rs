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
    io,
    process::Command,
    time::{Duration, Instant},
};

/// `gh auth token` コマンドを実行して GitHub トークンを取得する
fn get_github_token() -> Result<String> {
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context("`gh auth token` の実行に失敗しました。GitHub CLI はインストールされていますか？")?;

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
    /// 取得した Issue のリスト
    issues: Vec<Issue>,
    /// リストの選択状態（どの項目がハイライトされているか）
    list_state: ListState,
}

impl App {
    fn new(issues: Vec<Issue>) -> Self {
        let mut list_state = ListState::default();
        if !issues.is_empty() {
            // 最初の項目を選択状態にする
            list_state.select(Some(0));
        }
        Self { issues, list_state }
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

    let issues = page.items.into_iter().filter_map(|i| {
        Some(i)
    }).collect::<Vec<_>>();

    let app = App::new(issues);

    // 3. ターミナルのセットアップ
    // Raw Mode を有効にし、専用の描画バッファ（Alternate Screen）に切り替える
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. アプリケーションのメインループ実行
    let res = run_app(&mut terminal, app);

    // 5. ターミナルの復元（アプリ終了時）
    // Raw Mode を解除し、元のシェル画面に戻す
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("エラーが発生しました: {:?}", err)
    }

    Ok(())
}

/// メインのイベントループ（キー入力の待機と描画の繰り返し）
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        // 画面を描画
        terminal.draw(|f| ui(f, &mut app))?;

        // 次の描画までの待機時間を計算
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // キーイベントのチェック
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()), // 'q' で終了
                    KeyCode::Char('j') | KeyCode::Down => app.next(),     // 'j' または 下矢印で次へ
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),   // 'k' または 上矢印で前へ
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

/// UI のレイアウトとウィジェットの配置を定義する関数
fn ui(f: &mut ratatui::Frame, app: &mut App) {
    // 画面全体を左右に分割 (30% : 70%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    // 左側のエリアをさらに上下に分割
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(chunks[0]);

    // 左上: カテゴリ表示用のサイドバー
    let sidebar = Paragraph::new(vec![
        Line::from(vec![Span::styled("● My Issues", Style::default().add_modifier(Modifier::BOLD))]),
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

    // 選択状態を保持するウィジェットとして描画
    f.render_stateful_widget(list, left_chunks[1], &mut app.list_state);

    // 右側: 選択中の Issue の詳細表示
    let main_content = if let Some(selected) = app.list_state.selected() {
        if let Some(issue) = app.issues.get(selected) {
            let title = format!("#{} {}", issue.number, issue.title);
            let body = issue.body.as_deref().unwrap_or("（本文なし）");
            format!("{}\n\n{}", title, body)
        } else {
            "Issue を選択してください。".to_string()
        }
    } else {
        "Issue を選択してください。".to_string()
    };

    let main_panel = Paragraph::new(main_content)
        .block(Block::default().borders(Borders::ALL).title("詳細"))
        .wrap(Wrap { trim: true });
    f.render_widget(main_panel, chunks[1]);
}
