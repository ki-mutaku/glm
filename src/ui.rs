use crate::app::{App, Screen};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// UI の描画
pub fn ui(f: &mut Frame, app: &mut App) {
    let current_screen = app.current_screen.clone();
    match current_screen {
        Screen::IssueList => render_issue_list(f, app),
        Screen::RepositorySelector => render_repo_selector(f, app),
        Screen::IssueTitleInput { .. } => {
            // 背景を描画してから、その上にフローティングUIを描画
            render_issue_list(f, app);
            render_title_input_floating(f, app);
        }
        Screen::IssueDraft { .. } => render_issue_draft(f, app),
    }

    // エラーメッセージは最前面に表示
    if let Some(err) = &app.error_message {
        render_error_bar(f, err);
    }
}

/// エラーバーを描画
fn render_error_bar(f: &mut Frame, error_message: &str) {
    let area = centered_rect(80, 20, f.size());
    f.render_widget(Clear, area); // 背景をクリア

    let error_text = format!("❌ エラー:\n\n{}\n\n任意のキーを押して続行...", error_message);
    let error_paragraph = Paragraph::new(error_text)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title("エラー")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Rgb(50, 0, 0))),
        );
    f.render_widget(error_paragraph, area);
}

/// Issue リスト画面を描画
fn render_issue_list(f: &mut Frame, app: &mut App) {
    // 画面全体を3分割 (ヘッダー : メインエリア : ステータス行)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // ヘッダー
            Constraint::Min(0),    // メインエリア
            Constraint::Length(1), // ステータスバー
        ])
        .split(f.size());
    
    // ヘッダー: リポジトリ名
    let header_text = if let Some(repo) = &app.selected_repository {
        format!("リポジトリ: {}                [r] リポジトリ選択", repo.name)
    } else {
        "全Issue (自分にアサイン)         [r] リポジトリ選択".to_string()
    };
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(header, main_chunks[0]);

    // メインエリアを左右に分割 (30% : 70%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[1]);

    // 左下: Issue タイトルの一覧リスト or Empty State
    if app.issues.is_empty() {
        let empty_msg = if app.selected_repository.is_some() {
            "このリポジトリにオープンなIssueはありません。"
        } else {
            "あなたにアサインされたIssueはありません。"
        };
        
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(empty_msg),
            Line::from(""),
        ])
        .block(Block::default().borders(Borders::ALL).title("Issue 一覧"))
        .style(Style::default().fg(Color::Yellow));
        
        f.render_widget(empty, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .issues
            .iter()
            .map(|i| {
                let state_color = match i.state {
                    octocrab::models::IssueState::Open => Color::Green,
                    octocrab::models::IssueState::Closed => Color::Red,
                    _ => Color::Gray,
                };
                let state = Span::styled(format!("[{:?}]", i.state), Style::default().fg(state_color));
                let title = Span::raw(format!(" {}", i.title));
                ListItem::new(Line::from(vec![state, title]))
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

        f.render_stateful_widget(list, chunks[0], &mut app.list_state);
    }

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
    let help_text = if app.selected_repository.is_some() {
        " q: 終了 | e: 編集 | j/k: 移動 | r: リポジトリ選択 | n: 新規Issue "
    } else {
        " q: 終了 | e: 編集 | j/k: 移動 | r: リポジトリ選択 "
    };
    let status_bar =
        Paragraph::new(help_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(status_bar, main_chunks[2]);
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
    let header = Paragraph::new("リポジトリ選択                        [Esc] キャンセル")
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
            Line::from("リポジトリが見つかりません。"),
            Line::from(""),
            Line::from("GitHub のアクセス権限を確認するか、"),
            Line::from("`gh auth login` を実行してください。"),
        ])
        .block(Block::default().borders(Borders::ALL).title("リポジトリ一覧"))
        .style(Style::default().fg(Color::Yellow));
        
        f.render_widget(empty_msg, content_chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .repositories
            .iter()
            .map(|r| ListItem::new(r.name.clone()))
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("リポジトリ一覧"))
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
                Span::styled("リポジトリ: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&repo.name),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("説明: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(repo.description.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("スター数: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("⭐ {}", repo.stars)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("プライベート: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if repo.private { "はい" } else { "いいえ" }),
            ]),
        ];
        
        let detail = Paragraph::new(detail_text)
            .block(Block::default().borders(Borders::ALL).title("詳細"));
        f.render_widget(detail, content_chunks[1]);
    }
    
    // ヘルプバー
    let help = Paragraph::new("j/k: 移動 | Enter: 選択 | Esc: キャンセル")
        .style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(help, main_chunks[2]);
}

/// Issue タイトル入力用のフローティングUIを描画
fn render_title_input_floating(f: &mut Frame, app: &mut App) {
    let title = if let Screen::IssueTitleInput { title } = &app.current_screen {
        title.as_str()
    } else {
        return; // このUIは IssueTitleInput 状態でのみ描画される
    };

    let width = f.size().width * 60 / 100;
    let height = 3;
    let x = (f.size().width - width) / 2;
    let y = (f.size().height - height) / 2;
    let area = Rect::new(x, y, width, height);
    f.render_widget(Clear, area); // 背景をクリアして重ね描き

    let input_text = format!("{}▋", title); // カーソル表示
    
    let paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Issue のタイトルを入力 (Enterで確定, Escでキャンセル) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
    
    f.render_widget(paragraph, area);
}

/// Issue ドラフト画面を描画
fn render_issue_draft(f: &mut Frame, app: &mut App) {
    let (title, body) = if let Screen::IssueDraft { title, body } = &app.current_screen {
        (title.as_str(), body.as_str())
    } else {
        return;
    };
    
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // ヘッダー
            Constraint::Length(3),  // Title フィールド
            Constraint::Min(5),     // Body フィールド
            Constraint::Length(1),  // ヘルプバー
        ])
        .split(f.size());

    // ヘッダー
    let repo_name = app.selected_repository.as_ref().map(|r| r.name.as_str()).unwrap_or("unknown");
    let header = Paragraph::new(format!("Issue 作成: {}                [Esc] キャンセル", repo_name))
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(header, main_chunks[0]);

    // Title フィールド
    let title_block = Block::default().borders(Borders::ALL).title("タイトル");
    f.render_widget(Paragraph::new(title).block(title_block), main_chunks[1]);
    
    // Body フィールド
    let body_block = Block::default().borders(Borders::ALL).title("詳細");
    let display_body = if body.is_empty() {
        "（本文なし）\n\n'e' キーを押して外部エディタで編集してください。"
    } else {
        body
    };
    f.render_widget(Paragraph::new(display_body).block(body_block).wrap(Wrap { trim: true }), main_chunks[2]);
    
    // ヘルプバー
    let help = Paragraph::new(" e: 詳細を編集 | Enter: この内容で作成 | Esc: キャンセル ")
        .style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(help, main_chunks[3]);
}

/// 画面中央に指定したパーセンテージの Rect を作成するヘルパー関数
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
