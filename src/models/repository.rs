/// GitHub リポジトリを表現するデータモデル
#[derive(Debug, Clone)]
pub struct Repository {
    /// GitHub の内部 ID
    pub id: u64,
    /// "owner/repo" 形式のフルネーム
    pub name: String,
    /// オーナー名
    pub owner: String,
    /// リポジトリ名
    pub repo: String,
    /// リポジトリの説明
    pub description: Option<String>,
    /// スター数
    pub stars: u32,
    /// プライベートリポジトリかどうか
    pub private: bool,
}

/// Octocrab の Repository 型から内部 Repository 型への変換
impl From<octocrab::models::Repository> for Repository {
    fn from(repo: octocrab::models::Repository) -> Self {
        let full_name = repo.full_name.clone().unwrap_or_default();
        let parts: Vec<&str> = full_name.split('/').collect();
        
        Self {
            id: repo.id.0,
            name: full_name.clone(),
            owner: parts.get(0).unwrap_or(&"").to_string(),
            repo: parts.get(1).unwrap_or(&"").to_string(),
            description: repo.description.clone(),
            stars: repo.stargazers_count.unwrap_or(0),
            private: repo.private.unwrap_or(false),
        }
    }
}
