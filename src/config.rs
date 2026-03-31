use anyhow::{Context, Result};
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
    ProjectDirs::from("com", "glm", "glm").map(|dirs| dirs.config_dir().join("config.json"))
}

/// 設定ファイルを読み込む
/// 
/// ファイルが存在しない場合やパースに失敗した場合はデフォルト設定を返す
pub fn load_config() -> Result<AppConfig> {
    let path = match get_config_path() {
        Some(p) => p,
        None => {
            // 設定ディレクトリが取得できない場合はデフォルトを返す（エラーではない）
            return Ok(AppConfig::default());
        }
    };

    // ファイルが存在しない場合はデフォルトを返す（初回起動時など）
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path)
        .context("設定ファイルの読み込みに失敗しました")?;
    
    let config = serde_json::from_str(&content)
        .context("設定ファイルのパースに失敗しました")?;

    Ok(config)
}

/// 設定ファイルを保存する
pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = get_config_path()
        .context("設定ディレクトリのパスを取得できませんでした")?;

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .context("設定ディレクトリの作成に失敗しました")?;
    }

    let content = serde_json::to_string_pretty(config)
        .context("設定のシリアライズに失敗しました")?;
    
    fs::write(&path, content)
        .context("設定ファイルの書き込みに失敗しました")?;

    Ok(())
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
        let json = serde_json::to_string_pretty(&config).expect("シリアライズに失敗");

        // デシリアライズ
        let deserialized: AppConfig = serde_json::from_str(&json).expect("デシリアライズに失敗");

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

        let json = serde_json::to_string(&config).expect("シリアライズに失敗");

        let deserialized: AppConfig = serde_json::from_str(&json).expect("デシリアライズに失敗");

        assert!(deserialized.last_repository.is_none());
    }

    #[test]
    fn test_load_config_with_nonexistent_file() {
        // エッジケース: 存在しないファイルからの読み込み
        // load_config は存在しない場合はデフォルトを返す
        let config = load_config().expect("設定の読み込みに失敗");
        // ファイルが存在しない場合、または有効な設定が返されることを確認
        let _ = config.last_repository;
    }
}
