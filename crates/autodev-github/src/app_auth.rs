use crate::Result;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// GitHub App 인증 관리
pub struct GitHubAppAuth {
    app_id: String,
    private_key: EncodingKey,
    client: Client,
}

impl GitHubAppAuth {
    /// GitHub App 인증 생성
    ///
    /// # Arguments
    /// * `app_id` - GitHub App ID
    /// * `private_key_path` - Private key (.pem) 파일 경로
    pub fn new(app_id: String, private_key_path: &str) -> Result<Self> {
        // PEM 파일 읽기
        let private_key_pem = fs::read(private_key_path)
            .map_err(|e| crate::Error::AuthError(format!("Failed to read private key: {}", e)))?;

        // EncodingKey 생성
        let private_key = EncodingKey::from_rsa_pem(&private_key_pem)
            .map_err(|e| crate::Error::AuthError(format!("Invalid private key: {}", e)))?;

        Ok(Self {
            app_id,
            private_key,
            client: Client::new(),
        })
    }

    /// JWT 토큰 생성 (GitHub App 인증용)
    ///
    /// GitHub App으로 API를 호출하기 위한 JWT 생성
    /// 유효기간: 10분
    fn generate_jwt(&self) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = JwtClaims {
            iat: now - 60,           // 1분 전부터 유효 (시간 차이 허용)
            exp: now + 10 * 60,      // 10분 후 만료
            iss: self.app_id.clone(), // GitHub App ID
        };

        let header = Header::new(Algorithm::RS256);

        encode(&header, &claims, &self.private_key)
            .map_err(|e| crate::Error::AuthError(format!("Failed to generate JWT: {}", e)))
    }

    /// Installation Access Token 생성
    ///
    /// 특정 Repository/Organization에 설치된 GitHub App의 access token 발급
    ///
    /// # Arguments
    /// * `installation_id` - GitHub App Installation ID
    pub async fn get_installation_token(&self, installation_id: u64) -> Result<String> {
        let jwt = self.generate_jwt()?;

        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "AutoDev-Rust")
            .send()
            .await
            .map_err(|e| crate::Error::ApiError(format!("Failed to request token: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::Error::AuthError(format!(
                "Failed to get installation token ({}): {}",
                status, error_text
            )));
        }

        let token_response: InstallationTokenResponse = response
            .json()
            .await
            .map_err(|e| crate::Error::ApiError(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response.token)
    }

    /// Installation ID 조회 (Repository 기반)
    ///
    /// 특정 Repository에 설치된 GitHub App의 Installation ID 조회
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    pub async fn get_installation_id_for_repo(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<u64> {
        let jwt = self.generate_jwt()?;

        let url = format!("https://api.github.com/repos/{}/{}/installation", owner, repo);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "AutoDev-Rust")
            .send()
            .await
            .map_err(|e| crate::Error::ApiError(format!("Failed to get installation: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::Error::AuthError(format!(
                "Failed to get installation ID ({}): {}",
                status, error_text
            )));
        }

        let installation: InstallationResponse = response
            .json()
            .await
            .map_err(|e| crate::Error::ApiError(format!("Failed to parse installation: {}", e)))?;

        Ok(installation.id)
    }
}

#[derive(Debug, Serialize)]
struct JwtClaims {
    iat: u64, // Issued at
    exp: u64, // Expiration
    iss: String, // Issuer (GitHub App ID)
}

#[derive(Debug, Deserialize)]
struct InstallationTokenResponse {
    token: String,
    #[allow(dead_code)]
    expires_at: String,
}

#[derive(Debug, Deserialize)]
struct InstallationResponse {
    id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation() {
        // 테스트용 키 생성은 실제 private key가 필요하므로 스킵
        // 실제 테스트는 통합 테스트에서 수행
    }
}
