# API Integration Contract: GitHub API

**Date**: 2025-01-22  
**Feature**: 001-repo-scoped-issues  
**Contract Type**: External API Integration (Octocrab)

## Overview

本機能は GitHub REST API を Octocrab クライアント (v0.38) 経由で利用します。このドキュメントは使用する API エンドポイント、リクエスト/レスポンス形式、エラーハンドリングを定義します。

---

## Authentication

**Method**: Personal Access Token (PAT) via `gh auth token`

**Implementation**:
```rust
let token = gh::get_github_token()?; // 既存関数を再利用
let octocrab = Octocrab::builder()
    .personal_token(token)
    .build()?;
```

**Required Scopes**:
- `repo` (プライベートリポジトリへのフルアクセス)
- `read:user` (ユーザー情報の読み取り)

**Error Handling**:
- Token が取得できない場合: "Run `gh auth login` to authenticate"
- Token が無効な場合: "Authentication failed. Please re-run `gh auth login`"

---

## API Endpoints

### 1. List Repositories for Authenticated User

**Endpoint**: `GET /user/repos`

**Octocrab Method**:
```rust
octocrab
    .current()
    .list_repos_for_authenticated_user()
    .per_page(100)
    .send()
    .await?
```

**Request Parameters**:
| Parameter | Value | Purpose |
|-----------|-------|---------|
| `per_page` | 100 | ページあたりの取得件数 (最大) |
| `type` | `all` (デフォルト) | すべてのリポジトリ (後でフィルタ) |

**Response Type**: `octocrab::Page<octocrab::models::Repository>`

**Response Fields** (使用するもの):
```rust
pub struct Repository {
    pub id: RepositoryId,           // リポジトリ ID
    pub name: String,               // リポジトリ名 (例: "glm")
    pub full_name: Option<String>,  // フルネーム (例: "owner/glm")
    pub owner: Option<User>,        // オーナー情報
    pub description: Option<String>,// 説明
    pub private: Option<bool>,      // プライベートか
    pub stargazers_count: Option<u32>, // スター数
    // ... その他のフィールドは無視
}
```

**Client-Side Filtering**:
```rust
repos.into_iter()
    .filter(|r| r.private == Some(true))
    .collect()
```

**Pagination**:
```rust
let mut all_repos = Vec::new();
let mut page = octocrab.current()
    .list_repos_for_authenticated_user()
    .per_page(100)
    .send()
    .await?;

loop {
    all_repos.extend(page.items);
    page = match octocrab.get_page(&page.next).await? {
        Some(next_page) => next_page,
        None => break,
    };
}
```

**Error Handling**:
- 401 Unauthorized: "Authentication failed"
- 403 Forbidden (Rate Limit): "API rate limit exceeded"
- Network Error: "Could not connect to GitHub"

---

### 2. List Issues for Repository

**Endpoint**: `GET /repos/{owner}/{repo}/issues`

**Octocrab Method**:
```rust
octocrab
    .issues(owner, repo)
    .list()
    .state(octocrab::params::State::Open)
    .per_page(100)
    .send()
    .await?
```

**Request Parameters**:
| Parameter | Value | Purpose |
|-----------|-------|---------|
| `state` | `open` | オープンな Issue のみ |
| `per_page` | 100 | ページあたりの取得件数 |

**Response Type**: `octocrab::Page<octocrab::models::issues::Issue>`

**Response Fields** (既存の Issue 構造体を使用):
```rust
pub struct Issue {
    pub id: IssueId,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub repository_url: String,
    pub user: User,
    // ... その他
}
```

**Error Handling**:
- 404 Not Found: "Repository not found or access denied"
- 401 Unauthorized: "Authentication failed"
- Network Error: "Could not connect to GitHub"

---

### 3. Create Issue

**Endpoint**: `POST /repos/{owner}/{repo}/issues`

**Octocrab Method**:
```rust
octocrab
    .issues(owner, repo)
    .create(title)
    .body(body)
    .send()
    .await?
```

**Request Body**:
```json
{
  "title": "Issue title",
  "body": "Issue description (optional)"
}
```

**Response Type**: `octocrab::models::issues::Issue`

**Response Example**:
```rust
Issue {
    id: 123456789,
    number: 42,
    title: "Issue title",
    body: Some("Issue description"),
    state: IssueState::Open,
    // ...
}
```

**Validation** (Client-Side):
- Title: 必須、空でない文字列
- Body: オプション、空でも可

**Error Handling**:
- 404 Not Found: "Repository not found"
- 422 Unprocessable Entity (Validation): "Invalid issue data: [details]"
- 401 Unauthorized: "Authentication failed"
- Network Error: "Could not connect to GitHub"

---

### 4. Update Issue (既存機能、変更なし)

**Endpoint**: `PATCH /repos/{owner}/{repo}/issues/{issue_number}`

**Octocrab Method**:
```rust
octocrab
    .issues(owner, repo)
    .update(issue_number)
    .body(new_body)
    .send()
    .await?
```

**Note**: 本機能では既存の Issue 編集機能 (`e` キー) を変更しません。

---

## Rate Limiting

**GitHub API Limits**:
- 認証済み: 5,000 requests/hour
- 未認証: 60 requests/hour (本アプリは常に認証済み)

**Rate Limit Headers** (Octocrab が自動的に処理):
- `X-RateLimit-Limit`: 上限
- `X-RateLimit-Remaining`: 残り回数
- `X-RateLimit-Reset`: リセット時刻 (Unix timestamp)

**Rate Limit Exceeded Handling**:
```rust
match octocrab.issues(owner, repo).list().send().await {
    Err(octocrab::Error::GitHub { .. }) if is_rate_limit_error => {
        app.set_error("GitHub API rate limit exceeded. Please try again later.");
    }
    // ...
}
```

**Client-Side Optimization**:
- リポジトリリストはアプリ起動時に 1 回取得しキャッシュ
- Issue リストはリポジトリ選択時またはリフレッシュ時のみ取得
- Issue 作成後は差分取得ではなく全体リフレッシュ (シンプル化)

---

## Error Response Contract

### Standard Error Structure

GitHub API は以下の形式でエラーを返します：

```json
{
  "message": "Bad credentials",
  "documentation_url": "https://docs.github.com/rest"
}
```

**Octocrab Error Type**:
```rust
pub enum octocrab::Error {
    GitHub { source: GitHubError, .. },
    Http { source: reqwest::Error, .. },
    // ...
}
```

**Error Mapping**:
| HTTP Status | Octocrab Error | User Message |
|-------------|----------------|--------------|
| 401 | `GitHub { status: 401, .. }` | "Authentication failed. Run `gh auth login`." |
| 403 | `GitHub { status: 403, .. }` | "API rate limit exceeded. Try again later." |
| 404 | `GitHub { status: 404, .. }` | "Repository or resource not found." |
| 422 | `GitHub { status: 422, .. }` | "Invalid input: [details]" |
| 500, 502, 503 | `GitHub { status: 5xx, .. }` | "GitHub service error. Try again later." |
| Network Error | `Http { .. }` | "Network error. Check your connection." |

---

## Data Conversion Contract

### Octocrab Repository → Internal Repository

```rust
impl From<octocrab::models::Repository> for crate::models::Repository {
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
```

### Octocrab Issue → Internal Usage

```rust
// Octocrab の Issue をそのまま使用 (変換不要)
// app.issues: Vec<octocrab::models::issues::Issue>
```

---

## Testing Contract

### Mock Strategy

**Unit Tests**: Octocrab クライアントをモック化

```rust
#[cfg(test)]
mod tests {
    use mockall::mock;
    
    // Octocrab のトレイトをモック
    mock! {
        OctocrabClient {
            async fn list_repos(&self) -> Result<Vec<Repository>>;
            async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>>;
            async fn create_issue(&self, owner: &str, repo: &str, title: &str, body: &str) -> Result<Issue>;
        }
    }
}
```

**Integration Tests**: GitHub API テストアカウントまたはローカル GitHub Enterprise

---

## Deprecation & Versioning

**API Version**: GitHub REST API v3 (Octocrab 0.38 がサポート)

**Future Considerations**:
- GraphQL API: v2 で検討（複数リソースを 1 回のリクエストで取得）
- Webhooks: リアルタイム Issue 更新の通知

**Backward Compatibility**:
- Octocrab 0.38 → 0.39 マイグレーション時は Breaking Changes に注意
- GitHub API の Deprecation 警告はレスポンスヘッダー `Sunset` でチェック

---

## Security Contract

**Token Storage**:
- Token は `gh auth token` コマンド経由で取得、メモリ上のみ保持
- ファイルやログへの Token 出力を禁止

**Error Messages**:
- Token や認証情報をエラーメッセージに含めない

**TLS**:
- すべての API 通信は HTTPS (Octocrab がデフォルトで強制)

---

## Summary

| API Operation | Endpoint | Method | Authentication | Pagination |
|---------------|----------|--------|----------------|------------|
| List Repos | `/user/repos` | `current().list_repos_for_authenticated_user()` | PAT | Yes |
| List Issues | `/repos/{owner}/{repo}/issues` | `issues(owner, repo).list()` | PAT | Yes |
| Create Issue | `/repos/{owner}/{repo}/issues` | `issues(owner, repo).create()` | PAT | No |
| Update Issue | `/repos/{owner}/{repo}/issues/{number}` | `issues(owner, repo).update()` | PAT | No |

**Total API Calls** (典型的なセッション):
- アプリ起動: 1 (repos)
- リポジトリ選択: 1 (issues)
- Issue 作成: 1 (create) + 1 (refresh issues)
- **合計: 4 requests** (レート制限内で十分余裕)
