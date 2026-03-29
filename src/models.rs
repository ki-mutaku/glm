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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        // 正常系: Repositoryの基本的な作成をテスト
        let repo = Repository {
            name: "test-repo".to_string(),
            owner: "test-user".to_string(),
            description: Some("テストリポジトリ".to_string()),
            stars: 42,
            private: false,
        };

        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.owner, "test-user");
        assert_eq!(repo.description, Some("テストリポジトリ".to_string()));
        assert_eq!(repo.stars, 42);
        assert!(!repo.private);
    }

    #[test]
    fn test_repository_with_none_description() {
        // エッジケース: 説明なしのリポジトリをテスト
        let repo = Repository {
            name: "minimal-repo".to_string(),
            owner: "minimal-user".to_string(),
            description: None,
            stars: 0,
            private: true,
        };

        assert!(repo.description.is_none());
        assert_eq!(repo.stars, 0);
        assert!(repo.private);
    }

    #[test]
    fn test_repository_serialization() {
        // 正常系: JSONシリアライズ/デシリアライズをテスト
        let repo = Repository {
            name: "serialize-test".to_string(),
            owner: "test-owner".to_string(),
            description: Some("説明".to_string()),
            stars: 100,
            private: false,
        };

        // シリアライズ
        let json = serde_json::to_string(&repo).expect("シリアライズに失敗");
        
        // デシリアライズ
        let deserialized: Repository = serde_json::from_str(&json)
            .expect("デシリアライズに失敗");

        assert_eq!(deserialized.name, repo.name);
        assert_eq!(deserialized.owner, repo.owner);
        assert_eq!(deserialized.description, repo.description);
        assert_eq!(deserialized.stars, repo.stars);
        assert_eq!(deserialized.private, repo.private);
    }

    #[test]
    fn test_repository_with_large_stars() {
        // エッジケース: 大量のスター数を持つリポジトリをテスト
        let repo = Repository {
            name: "popular-repo".to_string(),
            owner: "popular-user".to_string(),
            description: Some("人気のリポジトリ".to_string()),
            stars: 999999,
            private: false,
        };

        assert_eq!(repo.stars, 999999);
    }
}
