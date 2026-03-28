mod app;
mod gh;
mod models;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use octocrab::Octocrab;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. GitHub CLI を使った認証
    println!("`gh auth token` を使用してトークンを取得中...");
    let token = gh::get_github_token()?;

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
        terminal.draw(|f| ui::ui(f, &mut app))?;

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
                                gh::edit_with_external_editor(issue.body.as_deref().unwrap_or(""))?;

                            // API を叩いて GitHub を更新
                            if let Some(body) = new_body {
                                if body != issue.body.as_deref().unwrap_or("") {
                                    if let Some((owner, repo)) =
                                        gh::parse_repo_owner(issue.repository_url.as_str())
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
