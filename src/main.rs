mod app;
mod config;
mod gh;
mod models;
mod ui;

use anyhow::{Context, Result};
use app::{App, Screen};
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

    // 2. Octocrab (GitHub API クライアント) の初期化
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Octocrab クライアントの構築に失敗しました")?;

    // 3. 設定の読み込みと App の初期化
    let config = config::load_config();
    let mut app = App::new(octocrab.clone(), vec![]); // まず空の Issue リストで初期化

    // 最後に開いたリポジトリがあれば、その Issue を取得
    if let Some(repo) = config.last_repository {
        println!(
            "最後に開いたリポジトリ ({}/{}) の Issue を取得中...",
            repo.owner, repo.name
        );
        app.select_repository(repo.clone());
        match gh::fetch_issues_for_repo(&octocrab, &repo.owner, &repo.name).await {
            Ok(issues) => {
                app.issues = issues;
                if !app.issues.is_empty() {
                    app.list_state.select(Some(0));
                }
            }
            Err(e) => {
                println!(
                    "Issueの取得に失敗しました: {}. 自分にアサインされたIssueを取得します。",
                    e
                );
                // 失敗した場合はフォールバック
                let page = octocrab
                    .search()
                    .issues_and_pull_requests("is:issue is:open assignee:@me")
                    .send()
                    .await?;
                app.issues = page.items.into_iter().collect();
                if !app.issues.is_empty() {
                    app.list_state.select(Some(0));
                }
            }
        }
    } else {
        println!("自分にアサインされたオープンな Issue を取得中...");
        let page = octocrab
            .search()
            .issues_and_pull_requests("is:issue is:open assignee:@me")
            .send()
            .await?;
        app.issues = page.items.into_iter().collect();
        if !app.issues.is_empty() {
            app.list_state.select(Some(0));
        }
    }

    // 4. ターミナルのセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 5. アプリケーションのメインループ実行
    let res = run_app(&mut terminal, app).await;

    // 6. ターミナルの復元
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
                // エラーメッセージ表示中は任意のキーでクリア
                if app.error_message.is_some() {
                    app.clear_error();
                    continue;
                }

                // 現在の画面に応じてキーイベントを処理
                let current_screen = app.current_screen.clone();
                match current_screen {
                    Screen::IssueList => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('r') => {
                            app.current_screen = Screen::RepositorySelector;
                            match gh::fetch_repositories(&app.octocrab).await {
                                Ok(repos) => {
                                    app.repositories = repos;
                                    if !app.repositories.is_empty() {
                                        app.repo_list_state.select(Some(0));
                                    }
                                }
                                Err(e) => {
                                    app.set_error(format!("リポジトリの取得に失敗しました: {}", e));
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            if app.selected_repository.is_none() {
                                app.set_error(
                                    "リポジトリを先に選択してください ('r'キー)".to_string(),
                                );
                            } else {
                                app.current_screen = Screen::IssueTitleInput {
                                    title: String::new(),
                                };
                            }
                        }
                        KeyCode::Char('e') => {
                            if let Some(index) = app.list_state.selected() {
                                let issue = app.issues[index].clone();
                                disable_raw_mode()?;
                                execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                                let new_body = gh::edit_with_external_editor(
                                    issue.body.as_deref().unwrap_or(""),
                                )?;
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
                                enable_raw_mode()?;
                                execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                                terminal.clear()?;
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.next(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous(),
                        _ => {}
                    },
                    Screen::RepositorySelector => {
                        match key.code {
                            KeyCode::Esc => app.current_screen = Screen::IssueList,
                            KeyCode::Char('j') | KeyCode::Down => app.next_repo(),
                            KeyCode::Char('k') | KeyCode::Up => app.previous_repo(),
                            KeyCode::Enter => {
                                if let Some(repo) = app.selected_repository_item() {
                                    let repo = repo.clone();
                                    app.select_repository(repo.clone());

                                    // 設定を保存
                                    config::save_config(&config::AppConfig {
                                        last_repository: Some(repo.clone()),
                                    });

                                    match gh::fetch_issues_for_repo(
                                        &app.octocrab,
                                        &repo.owner,
                                        &repo.name,
                                    )
                                    .await
                                    {
                                        Ok(issues) => {
                                            app.issues = issues;
                                            app.list_state.select(if app.issues.is_empty() {
                                                None
                                            } else {
                                                Some(0)
                                            });
                                            app.current_screen = Screen::IssueList;
                                        }
                                        Err(e) => {
                                            app.set_error(format!("Issueの取得に失敗しました: {}", e));
                                            app.current_screen = Screen::IssueList;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Screen::IssueTitleInput { .. } => match key.code {
                        KeyCode::Esc => app.current_screen = Screen::IssueList,
                        KeyCode::Enter => {
                            if let Screen::IssueTitleInput { title } = &app.current_screen {
                                if title.trim().is_empty() {
                                    app.set_error("タイトルは必須です。".to_string());
                                } else {
                                    app.current_screen = Screen::IssueDraft {
                                        title: title.clone(),
                                        body: String::new(),
                                    };
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Screen::IssueTitleInput { ref mut title } =
                                &mut app.current_screen
                            {
                                title.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if let Screen::IssueTitleInput { ref mut title } =
                                &mut app.current_screen
                            {
                                title.pop();
                            }
                        }
                        _ => {}
                    },
                    Screen::IssueDraft { .. } => {
                        match key.code {
                            KeyCode::Esc => app.current_screen = Screen::IssueList,
                            KeyCode::Char('e') => {
                                let mut body_clone = String::new();
                                if let Screen::IssueDraft { body, .. } = &app.current_screen {
                                    body_clone = body.clone();
                                }

                                disable_raw_mode()?;
                                execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                                let new_body_opt = gh::edit_with_external_editor(&body_clone);
                                enable_raw_mode()?;
                                execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                                terminal.clear()?;

                                match new_body_opt {
                                    Ok(Some(edited_body)) => {
                                        if let Screen::IssueDraft { ref mut body, .. } =
                                            &mut app.current_screen
                                        {
                                            *body = edited_body;
                                        }
                                    }
                                    Ok(None) => {} // ユーザーがキャンセル
                                    Err(e) => app.set_error(format!(
                                        "エディタでエラーが発生しました: {}",
                                        e
                                    )),
                                }
                            }
                            KeyCode::Enter => {
                                if let Screen::IssueDraft { title, body } = &app.current_screen {
                                    if let Some(repo) = &app.selected_repository {
                                        let owner = repo.owner.clone();
                                        let repo_name = repo.name.clone();
                                        let title_clone = title.clone();
                                        let body_clone = body.clone();

                                        match gh::create_issue(
                                            &app.octocrab,
                                            &owner,
                                            &repo_name,
                                            &title_clone,
                                            &body_clone,
                                        )
                                        .await
                                        {
                                            Ok(_new_issue) => {
                                                // Issue リストを再取得
                                                match gh::fetch_issues_for_repo(
                                                    &app.octocrab,
                                                    &owner,
                                                    &repo_name,
                                                )
                                                .await
                                                {
                                                    Ok(issues) => {
                                                        app.issues = issues;
                                                        app.list_state.select(
                                                            if app.issues.is_empty() {
                                                                None
                                                            } else {
                                                                Some(0)
                                                            },
                                                        );
                                                    }
                                                    Err(e) => app.set_error(format!(
                                                        "Issueの再取得に失敗: {}",
                                                        e
                                                    )),
                                                }
                                                app.current_screen = Screen::IssueList;
                                            }
                                            Err(e) => {
                                                app.set_error(format!("Issueの作成に失敗: {}", e))
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}
