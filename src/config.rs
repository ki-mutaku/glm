use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::Repository;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    pub last_repository: Option<Repository>,
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "glm", "glm").map(|dirs| dirs.config_dir().join("config.json"))
}

/// 指定されたパスから設定ファイルを読み込む（テスト可能な内部実装）
fn load_config_from_path(path: &Path) -> Result<AppConfig> {
    // ファイルが存在しない場合はデフォルトを返す（初回起動時など）
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(path)
        .context("設定ファイルの読み込みに失敗しました")?;
    
    let config = serde_json::from_str(&content)
        .context("設定ファイルのパースに失敗しました")?;

    Ok(config)
}

/// 設定ファイルを読み込む
/// 
/// ファイルが存在しない場合はデフォルト設定を返す
pub fn load_config() -> Result<AppConfig> {
    let path = match get_config_path() {
        Some(p) => p,
        None => {
            // 設定ディレクトリが取得できない場合はデフォルトを返す（エラーではない）
            return Ok(AppConfig::default());
        }
    };

    load_config_from_path(&path)
}

/// 指定されたパスに設定ファイルを保存する（テスト可能な内部実装）
fn save_config_to_path(config: &AppConfig, path: &Path) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .context("設定ディレクトリの作成に失敗しました")?;
    }

    let content = serde_json::to_string_pretty(config)
        .context("設定のシリアライズに失敗しました")?;
    
    fs::write(path, content)
        .context("設定ファイルの書き込みに失敗しました")?;

    Ok(())
}

/// 設定ファイルを保存する
pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = get_config_path()
        .context("設定ディレクトリのパスを取得できませんでした")?;

    save_config_to_path(config, &path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

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
        let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
        let config_path = temp_dir.path().join("nonexistent.json");

        // ファイルが存在しないことを確認
        assert!(!config_path.exists());

        // 存在しないファイルを読み込むとデフォルト設定が返される
        let config = load_config_from_path(&config_path).expect("設定の読み込みに失敗");
        assert_eq!(config, AppConfig::default());
        assert!(config.last_repository.is_none());
    }

    #[test]
    fn test_load_config_with_invalid_json() {
        // エッジケース: 壊れたJSONファイルからの読み込み
        let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
        let config_path = temp_dir.path().join("invalid.json");

        // 壊れたJSONを書き込む
        let mut file = fs::File::create(&config_path).expect("ファイルの作成に失敗");
        file.write_all(b"{ invalid json }").expect("書き込みに失敗");

        // 壊れたJSONを読み込むとエラーが返される
        let result = load_config_from_path(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("パース"));
    }

    #[test]
    fn test_save_and_load_config_roundtrip() {
        // 正常系: 保存と読み込みのラウンドトリップテスト
        let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
        let config_path = temp_dir.path().join("config.json");

        let repo = Repository {
            name: "roundtrip-test".to_string(),
            owner: "test-owner".to_string(),
            description: Some("ラウンドトリップテスト".to_string()),
            stars: 123,
            private: false,
        };

        let original_config = AppConfig {
            last_repository: Some(repo),
        };

        // 保存
        save_config_to_path(&original_config, &config_path).expect("設定の保存に失敗");

        // ファイルが作成されたことを確認
        assert!(config_path.exists());

        // 読み込み
        let loaded_config = load_config_from_path(&config_path).expect("設定の読み込みに失敗");

        // 元の設定と一致することを確認
        assert_eq!(loaded_config, original_config);
        assert!(loaded_config.last_repository.is_some());
        let loaded_repo = loaded_config.last_repository.unwrap();
        assert_eq!(loaded_repo.name, "roundtrip-test");
        assert_eq!(loaded_repo.owner, "test-owner");
        assert_eq!(loaded_repo.stars, 123);
    }
}
