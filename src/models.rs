//! データモデルを定義するモジュール

use serde::{Deserialize, Serialize};

/// リポジトリ情報を保持する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// リポジトリ名 (e.g., "glm")
    pub name: String,
    /// 所有者のログイン名
    pub owner: String,
    /// リポジトリの説明
    pub description: Option<String>,
    /// スターの数
    pub stars: i64,
    /// プライベートリポジトリかどうか
    pub private: bool,
}

impl From<octocrab::models::Repository> for Repository {
    fn from(repo: octocrab::models::Repository) -> Self {
        Self {
            name: repo.name,
            owner: repo
                .owner
                .map_or_else(|| "N/A".to_string(), |owner| owner.login),
            description: repo.description,
            stars: repo.stargazers_count.unwrap_or(0) as i64,
            private: repo.private.unwrap_or(false),
        }
    }
}
