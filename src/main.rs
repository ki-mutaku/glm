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
                // エラーメッセージ表示中は任意のキーでクリア
                if app.error_message.is_some() {
                    app.clear_error();
                    continue;
                }
                
                // Ctrl+E の処理（Issue フォームでの外部エディタ起動）
                if key.modifiers.contains(event::KeyModifiers::CONTROL) && key.code == KeyCode::Char('e') {
                    if app.current_screen == app::Screen::IssueForm {
                        if let Some(form) = &mut app.issue_form {
                            // ターミナルを一時復元
                            disable_raw_mode()?;
                            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                            
                            match gh::edit_with_external_editor(&form.body) {
                                Ok(Some(edited_body)) => {
                                    form.body = edited_body;
                                }
                                Ok(None) => {
                                    // ユーザーがキャンセルした場合
                                }
                                Err(e) => {
                                    app.set_error(format!("Editor failed: {}", e));
                                }
                            }
                            
                            // TUI を再開
                            enable_raw_mode()?;
                            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                            terminal.clear()?;
                        }
                    }
                    continue;
                }
                
                match key.code {
                    KeyCode::Char(c) => {
                        // Issue フォームでの文字入力
                        if app.current_screen == app::Screen::IssueForm {
                            if let Some(form) = &mut app.issue_form {
                                match form.focused_field {
                                    app::FormField::Title => {
                                        form.title.push(c);
                                    }
                                    app::FormField::Body => {
                                        form.body.push(c);
                                    }
                                }
                            }
                        } else {
                            // 通常のキーバインド
                            match c {
                                'q' => return Ok(()),
                                'r' => {
                                    // リポジトリ選択画面に遷移
                                    app.current_screen = app::Screen::RepositorySelector;
                                    
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
                                'n' => {
                                    if app.current_screen == app::Screen::IssueList {
                                        if app.selected_repository.is_none() {
                                            app.set_error("Please select a repository first (press 'r').".to_string());
                                        } else {
                                            app.current_screen = app::Screen::IssueForm;
                                            app.issue_form = Some(app::IssueFormState::default());
                                        }
                                    }
                                }
                                'e' => {
                                    // 外部エディタでの編集（Issue リスト画面のみ）
                                    if app.current_screen == app::Screen::IssueList {
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
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        match app.current_screen {
                            app::Screen::IssueList => app.next(),
                            app::Screen::RepositorySelector => app.next_repo(),
                            app::Screen::IssueForm => {
                                // Phase 2 で実装
                            }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        match app.current_screen {
                            app::Screen::IssueList => app.previous(),
                            app::Screen::RepositorySelector => app.previous_repo(),
                            app::Screen::IssueForm => {
                                // フォーム編集中は無効
                            }
                        }
                    }
                    KeyCode::Tab => {
                        if app.current_screen == app::Screen::IssueForm {
                            if let Some(form) = &mut app.issue_form {
                                form.focused_field = match form.focused_field {
                                    app::FormField::Title => app::FormField::Body,
                                    app::FormField::Body => app::FormField::Title,
                                };
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if app.current_screen == app::Screen::IssueForm {
                            if let Some(form) = &mut app.issue_form {
                                match form.focused_field {
                                    app::FormField::Title => {
                                        form.title.pop();
                                    }
                                    app::FormField::Body => {
                                        form.body.pop();
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        match app.current_screen {
                            app::Screen::RepositorySelector => {
                                app.current_screen = app::Screen::IssueList;
                            }
                            app::Screen::IssueForm => {
                                app.current_screen = app::Screen::IssueList;
                                app.issue_form = None;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Enter => {
                        match app.current_screen {
                            app::Screen::RepositorySelector => {
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
                                            app.current_screen = app::Screen::IssueList;
                                        }
                                        Err(e) => {
                                            app.set_error(format!("Failed to fetch issues: {}", e));
                                            app.current_screen = app::Screen::IssueList;
                                        }
                                    }
                                }
                            }
                            app::Screen::IssueForm => {
                                if let Some(form) = &app.issue_form {
                                    // Title が空の場合はエラー
                                    if form.title.trim().is_empty() {
                                        app.set_error("Issue title is required.".to_string());
                                    } else if form.focused_field == app::FormField::Body {
                                        // Body フィールドにフォーカスがある場合のみ送信
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
                                                    
                                                    app.current_screen = app::Screen::IssueList;
                                                    app.issue_form = None;
                                                }
                                                Err(e) => {
                                                    app.set_error(format!("Failed to create issue: {}", e));
                                                }
                                            }
                                        }
                                    } else {
                                        // Title フィールドにフォーカスがある場合は Body に移動
                                        if let Some(form) = &mut app.issue_form {
                                            form.focused_field = app::FormField::Body;
                                        }
                                    }
                                }
                            }
                            _ => {}
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
