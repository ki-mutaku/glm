use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;
use tempfile::Builder;

/// `gh auth token` コマンドを実行して GitHub トークンを取得する
pub fn get_github_token() -> Result<String> {
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

/// Issue の repository_url (https://api.github.com/repos/owner/repo) から owner と repo を抽出する
pub fn parse_repo_owner(url: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = url.split('/').collect();
    let len = parts.len();
    if len >= 2 {
        Some((parts[len - 2].to_string(), parts[len - 1].to_string()))
    } else {
        None
    }
}

/// 外部エディタを起動して文字列を編集する
pub fn edit_with_external_editor(initial_content: &str) -> Result<Option<String>> {
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
