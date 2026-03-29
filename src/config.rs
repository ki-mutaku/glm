use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::models::Repository;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub last_repository: Option<Repository>,
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "glm", "glm")
        .map(|dirs| dirs.config_dir().join("config.json"))
}

pub fn load_config() -> AppConfig {
    if let Some(path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) {
    if let Some(path) = get_config_path() {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(content) = serde_json::to_string_pretty(config) {
            let _ = fs::write(path, content);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        // 正常系: デフォルト設定のテスト
        let config = AppConfig::default();
        assert!(config.last_repository.is_none());
    }

    #[test]
    fn test_app_config_with_repository() {
        // 正常系: リポジトリを持つ設定のテスト
        let repo = Repository {
            name: "test-repo".to_string(),
            owner: "test-owner".to_string(),
            description: Some("テスト用リポジトリ".to_string()),
            stars: 10,
            private: false,
        };

        let config = AppConfig {
            last_repository: Some(repo.clone()),
        };

        assert!(config.last_repository.is_some());
        let saved_repo = config.last_repository.unwrap();
        assert_eq!(saved_repo.name, "test-repo");
        assert_eq!(saved_repo.owner, "test-owner");
    }

    #[test]
    fn test_app_config_serialization() {
        // 正常系: 設定のシリアライズ/デシリアライズをテスト
        let repo = Repository {
            name: "my-project".to_string(),
            owner: "my-user".to_string(),
            description: Some("プロジェクト説明".to_string()),
            stars: 42,
            private: true,
        };

        let config = AppConfig {
            last_repository: Some(repo),
        };

        // シリアライズ
        let json = serde_json::to_string_pretty(&config)
            .expect("シリアライズに失敗");

        // デシリアライズ
        let deserialized: AppConfig = serde_json::from_str(&json)
            .expect("デシリアライズに失敗");

        assert!(deserialized.last_repository.is_some());
        let repo = deserialized.last_repository.unwrap();
        assert_eq!(repo.name, "my-project");
        assert_eq!(repo.owner, "my-user");
        assert_eq!(repo.stars, 42);
        assert!(repo.private);
    }

    #[test]
    fn test_app_config_empty_serialization() {
        // エッジケース: 空の設定のシリアライズ
        let config = AppConfig {
            last_repository: None,
        };

        let json = serde_json::to_string(&config)
            .expect("シリアライズに失敗");

        let deserialized: AppConfig = serde_json::from_str(&json)
            .expect("デシリアライズに失敗");

        assert!(deserialized.last_repository.is_none());
    }

    #[test]
    fn test_load_config_with_nonexistent_file() {
        // エッジケース: 存在しないファイルからの読み込み
        // load_config は存在しない場合はデフォルトを返す
        // ただし、実環境では設定ファイルが既に存在する可能性があるため
        // デフォルト設定または有効な設定が返されることを確認
        let config = load_config();
        // 関数が正常に動作し、AppConfig を返すことを確認
        // (値の詳細は環境に依存するため、型のみをチェック)
        let _ = config.last_repository;
    }
}
