use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;
use tempfile::Builder;

use crate::models::Repository;
use octocrab::{models::issues::Issue, Octocrab};

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

/// 認証ユーザーのリポジトリ一覧を取得する
pub async fn fetch_repositories(octocrab: &Octocrab) -> Result<Vec<Repository>> {
    let mut all_repos = Vec::new();
    let mut page = octocrab
        .current()
        .list_repos_for_authenticated_user()
        .per_page(100)
        .send()
        .await
        .context("リポジトリ一覧の取得に失敗しました")?;

    loop {
        all_repos.extend(page.items.into_iter().map(Repository::from));

        page = match octocrab.get_page(&page.next).await? {
            Some(next_page) => next_page,
            None => break,
        };
    }

    Ok(all_repos)
}

/// 特定リポジトリの Issue 一覧を取得する
pub async fn fetch_issues_for_repo(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
) -> Result<Vec<Issue>> {
    let page = octocrab
        .issues(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(100)
        .send()
        .await
        .context("Issue 一覧の取得に失敗しました")?;

    Ok(page.items)
}

/// 新規 Issue を GitHub に作成する
pub async fn create_issue(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
) -> Result<Issue> {
    let issue = octocrab
        .issues(owner, repo)
        .create(title)
        .body(body)
        .send()
        .await
        .context("Issue の作成に失敗しました")?;

    Ok(issue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_owner_valid_url() {
        // 正常系: 正しい形式のURLをテスト
        let url = "https://api.github.com/repos/octocat/Hello-World";
        let result = parse_repo_owner(url);

        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn test_parse_repo_owner_trailing_slash() {
        // エッジケース: 末尾にスラッシュがある場合
        let url = "https://api.github.com/repos/octocat/Hello-World/";
        let result = parse_repo_owner(url);

        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "Hello-World");
        assert_eq!(repo, "");
    }

    #[test]
    fn test_parse_repo_owner_short_url() {
        // 正常系: 短いURLでも owner/repo を抽出
        let url = "owner/repo";
        let result = parse_repo_owner(url);

        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_repo_owner_single_part() {
        // エッジケース: 単一のパスのみの場合
        let url = "singlepart";
        let result = parse_repo_owner(url);

        // スラッシュがないので、最後の2つを取得できず None を返す
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_repo_owner_empty_string() {
        // エッジケース: 空文字列の場合
        let url = "";
        let result = parse_repo_owner(url);

        // 空文字列は有効な owner/repo として解析されず None を返す
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_repo_owner_with_dash_and_underscore() {
        // 正常系: ダッシュやアンダースコアを含むリポジトリ名
        let url = "https://api.github.com/repos/rust-lang/rust_analyzer";
        let result = parse_repo_owner(url);

        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust_analyzer");
    }

    #[test]
    fn test_parse_repo_owner_with_numbers() {
        // 正常系: 数字を含むリポジトリ名
        let url = "https://api.github.com/repos/user123/project456";
        let result = parse_repo_owner(url);

        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "user123");
        assert_eq!(repo, "project456");
    }

    #[test]
    fn test_parse_repo_owner_long_path() {
        // 正常系: 長いパスでも最後の2つを抽出
        let url = "https://api.github.com/extra/path/repos/testowner/testrepo";
        let result = parse_repo_owner(url);

        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "testowner");
        assert_eq!(repo, "testrepo");
    }
}
