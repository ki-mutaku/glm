use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// UI の描画
pub fn ui(f: &mut Frame, app: &mut App) {
    // エラーバーのチェック（後で実装）
    if let Some(err) = &app.error_message {
        render_error_bar(f, err);
        return;
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

/// エラーバーを描画
fn render_error_bar(f: &mut Frame, error_message: &str) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    
    let error_text = format!("❌ Error: {}\n\nPress any key to continue...", error_message);
    let error_bar = Paragraph::new(error_text)
        .style(Style::default().bg(Color::Red).fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Error"));
    f.render_widget(error_bar, main_chunks[0]);
}

/// Issue リスト画面を描画（既存の ui 関数の中身）
fn render_issue_list_original(f: &mut Frame, app: &mut App) {
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

/// リポジトリ選択画面を描画
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
    }
    
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
