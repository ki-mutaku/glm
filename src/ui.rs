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
