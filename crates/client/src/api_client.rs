use rorumall_shared::ApiError;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client_keys::{sign_request, KeyPair};

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    keys: Option<KeyPair>,
    handle: Option<String>,
    domain: Option<String>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: String::new(),
            keys: None,
            handle: None,
            domain: None,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_signing(
        mut self,
        keys: Option<KeyPair>,
        handle: Option<String>,
        domain: Option<String>,
    ) -> Self {
        self.keys = keys;
        self.handle = handle;
        self.domain = domain;
        self
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        if self.base_url.is_empty() {
            if path.starts_with('/') {
                path.to_string()
            } else {
                format!("/{path}")
            }
        } else {
            let base = self.base_url.trim_end_matches('/');
            let path = path.trim_start_matches('/');
            format!("{base}/{path}")
        }
    }

    fn apply_signing(
        &self,
        mut rb: reqwest::RequestBuilder,
        method: &str,
        url: &str,
        path: &str,
        body: &[u8],
    ) -> reqwest::RequestBuilder {
        if let (Some(keys), Some(handle), Some(domain)) = (&self.keys, &self.handle, &self.domain) {
            let path_only = if let Ok(u) = reqwest::Url::parse(url) {
                u.path().to_string()
            } else {
                path.split('?').next().unwrap_or(path).to_string()
            };

            if let Some(headers) = sign_request(method, &path_only, body, keys, handle, domain) {
                rb = rb.header("X-OFSCP-Actor", headers.actor);
                rb = rb.header("X-OFSCP-Timestamp", headers.timestamp);
                rb = rb.header(
                    "X-OFSCP-Signature",
                    format!("keyId=\"{}\", signature=\"{}\"", headers.key_id, headers.signature),
                );
            }
        }
        rb
    }

    pub async fn get_json<TRes: DeserializeOwned>(&self, path: &str) -> Result<TRes, ApiError> {
        let url = self.url(path);
        let rb = self.client.get(&url);
        let rb = self.apply_signing(rb, "GET", &url, path, &[]);

        let resp = rb.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(format!("failed to read body: {e}")))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
    }

    pub async fn post_json<TReq: Serialize, TRes: DeserializeOwned>(
        &self,
        path: &str,
        body: &TReq,
    ) -> Result<TRes, ApiError> {
        let url = self.url(path);
        let body_bytes = serde_json::to_vec(body).map_err(|e| ApiError::Deserialize(e.to_string()))?;
        let rb = self.client.post(&url);
        let rb = self.apply_signing(rb, "POST", &url, path, &body_bytes);

        let resp = rb
            .body(body_bytes)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        if text.is_empty() {
            serde_json::from_str("null").map_err(|e| ApiError::Deserialize(e.to_string()))
        } else {
            serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
        }
    }

    pub async fn put_json<TReq: Serialize, TRes: DeserializeOwned>(
        &self,
        path: &str,
        body: &TReq,
    ) -> Result<TRes, ApiError> {
        let url = self.url(path);
        let body_bytes = serde_json::to_vec(body).map_err(|e| ApiError::Deserialize(e.to_string()))?;
        let rb = self.client.put(&url);
        let rb = self.apply_signing(rb, "PUT", &url, path, &body_bytes);

        let resp = rb
            .body(body_bytes)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        if text.is_empty() {
            serde_json::from_str("null").map_err(|e| ApiError::Deserialize(e.to_string()))
        } else {
            serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
        }
    }

    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let url = self.url(path);
        let rb = self.client.delete(&url);
        let rb = self.apply_signing(rb, "DELETE", &url, path, &[]);

        let resp = rb.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(format!("failed to read body: {e}")))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        Ok(())
    }

    pub async fn patch_json<TReq: Serialize, TRes: DeserializeOwned>(
        &self,
        path: &str,
        body: &TReq,
    ) -> Result<TRes, ApiError> {
        let url = self.url(path);
        let body_bytes = serde_json::to_vec(body).map_err(|e| ApiError::Deserialize(e.to_string()))?;
        let rb = self.client.patch(&url);
        let rb = self.apply_signing(rb, "PATCH", &url, path, &body_bytes);

        let resp = rb
            .body(body_bytes)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        if text.is_empty() {
            serde_json::from_str("null").map_err(|e| ApiError::Deserialize(e.to_string()))
        } else {
            serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
        }
    }

    // --- Profile/Presence/Privacy API methods ---

    pub async fn update_profile(
        &self,
        update: &rorumall_shared::UpdateProfileRequest,
    ) -> Result<rorumall_shared::UserProfile, ApiError> {
        self.patch_json("/api/me/profile", update).await
    }

    pub async fn get_own_presence(&self) -> Result<rorumall_shared::Presence, ApiError> {
        self.get_json("/api/me/presence").await
    }

    pub async fn update_presence(
        &self,
        update: &rorumall_shared::UpdatePresenceRequest,
    ) -> Result<rorumall_shared::Presence, ApiError> {
        self.put_json("/api/me/presence", update).await
    }

    pub async fn get_user_presence(&self, handle: &str) -> Result<rorumall_shared::Presence, ApiError> {
        self.get_json(&format!("/api/users/{}/presence", handle)).await
    }

    pub async fn get_privacy_settings(&self) -> Result<rorumall_shared::PrivacySettings, ApiError> {
        self.get_json("/api/me/privacy").await
    }

    pub async fn update_privacy_settings(
        &self,
        settings: &rorumall_shared::PrivacySettings,
    ) -> Result<rorumall_shared::PrivacySettings, ApiError> {
        self.put_json("/api/me/privacy", settings).await
    }

    pub async fn get_user_profile(&self, handle: &str) -> Result<rorumall_shared::UserProfile, ApiError> {
        self.get_json(&format!("/api/users/{}/profile", handle)).await
    }

    pub async fn get_channels(
        &self,
        group_id: &str,
    ) -> Result<Vec<rorumall_shared::Channel>, ApiError> {
        self.get_json(&format!("/api/groups/{}/channels", group_id))
            .await
    }

    pub async fn create_channel(
        &self,
        group_id: &str,
        name: &str,
        channel_type: &str,
    ) -> Result<rorumall_shared::Channel, ApiError> {
        self.post_json(
            &format!("/api/groups/{}/channels", group_id),
            &serde_json::json!({ "name": name, "type": channel_type }),
        )
        .await
    }

    pub async fn create_group(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<rorumall_shared::Group, ApiError> {
        let mut body = serde_json::json!({ "name": name });
        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }
        self.post_json("/api/groups", &body).await
    }

    pub async fn list_group_members(
        &self,
        group_id: &str,
    ) -> Result<rorumall_shared::ListMembersResponse, ApiError> {
        self.get_json(&format!("/api/groups/{}/members", group_id)).await
    }

    pub async fn remove_group_member(&self, group_id: &str, user_id: &str) -> Result<(), ApiError> {
        self.delete(&format!(
            "/api/groups/{}/members/{}",
            group_id,
            urlencoding::encode(user_id)
        ))
        .await
    }

    pub async fn update_member_roles(
        &self,
        group_id: &str,
        user_id: &str,
        operation: &str,
        role: &str,
    ) -> Result<(), ApiError> {
        self.patch_json::<_, ()>(
            &format!(
                "/api/groups/{}/members/{}",
                group_id,
                urlencoding::encode(user_id)
            ),
            &rorumall_shared::UpdateMemberRolesRequest {
                operation: operation.to_string(),
                role: role.to_string(),
            },
        )
        .await?;
        Ok(())
    }

    pub async fn set_member_base_role(&self, group_id: &str, user_id: &str, role: &str) -> Result<(), ApiError> {
        self.update_member_roles(group_id, user_id, "set_base", role).await
    }

    pub async fn add_member_role(&self, group_id: &str, user_id: &str, role: &str) -> Result<(), ApiError> {
        self.update_member_roles(group_id, user_id, "add", role).await
    }

    pub async fn remove_member_role(&self, group_id: &str, user_id: &str, role: &str) -> Result<(), ApiError> {
        self.update_member_roles(group_id, user_id, "remove", role).await
    }

    pub async fn update_group_privacy(
        &self,
        group_id: &str,
        settings: &rorumall_shared::UpdateGroupPrivacyRequest,
    ) -> Result<(), ApiError> {
        let _: () = self
            .patch_json(&format!("/api/groups/{}/privacy", group_id), settings)
            .await?;
        Ok(())
    }

    pub async fn list_roles(&self, group_id: &str) -> Result<rorumall_shared::ListRolesResponse, ApiError> {
        self.get_json(&format!("/api/groups/{}/roles", group_id)).await
    }

    pub async fn create_role(
        &self,
        group_id: &str,
        request: &rorumall_shared::CreateRoleRequest,
    ) -> Result<rorumall_shared::GroupRole, ApiError> {
        self.post_json(&format!("/api/groups/{}/roles", group_id), request).await
    }

    pub async fn update_role(
        &self,
        group_id: &str,
        role_id: &str,
        request: &rorumall_shared::UpdateRoleRequest,
    ) -> Result<rorumall_shared::GroupRole, ApiError> {
        self.put_json(&format!("/api/groups/{}/roles/{}", group_id, role_id), request).await
    }

    pub async fn delete_role(&self, group_id: &str, role_id: &str) -> Result<(), ApiError> {
        self.delete(&format!("/api/groups/{}/roles/{}", group_id, role_id)).await
    }

    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>, ApiError> {
        let url = self.url(path);
        let rb = self.client.get(&url);
        let rb = self.apply_signing(rb, "GET", &url, path, &[]);

        let resp = rb.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();

        if !is_success {
            let text = resp
                .text()
                .await
                .map_err(|e| ApiError::Network(format!("failed to read body: {e}")))?;
            return Err(ApiError::Http { status, body: text });
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| ApiError::Network(format!("failed to read bytes: {e}")))
    }

    pub async fn upload_file(
        &self,
        file_data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<rorumall_shared::Attachment, ApiError> {
        let url = self.url("/api/uploads");
        let part = reqwest::multipart::Part::bytes(file_data)
            .file_name(filename.to_string())
            .mime_str(content_type)
            .map_err(|e| ApiError::Network(format!("Invalid MIME type: {}", e)))?;
        let form = reqwest::multipart::Form::new().part("file", part);
        let rb = self.client.post(&url).multipart(form);
        let rb = self.apply_signing(rb, "POST", &url, "/api/uploads", &[]);

        let resp = rb.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
    }

    pub async fn upload_avatar(
        &self,
        file_data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<rorumall_shared::AvatarResponse, ApiError> {
        let url = self.url("/api/me/avatar");
        let part = reqwest::multipart::Part::bytes(file_data)
            .file_name(filename.to_string())
            .mime_str(content_type)
            .map_err(|e| ApiError::Network(format!("Invalid MIME type: {}", e)))?;
        let form = reqwest::multipart::Form::new().part("avatar", part);
        let rb = self.client.post(&url).multipart(form);
        let rb = self.apply_signing(rb, "POST", &url, "/api/me/avatar", &[]);

        let resp = rb.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
    }

    pub async fn upload_group_avatar(
        &self,
        group_id: &str,
        file_data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<rorumall_shared::AvatarResponse, ApiError> {
        let path = format!("/api/groups/{}/avatar", group_id);
        let url = self.url(&path);
        let part = reqwest::multipart::Part::bytes(file_data)
            .file_name(filename.to_string())
            .mime_str(content_type)
            .map_err(|e| ApiError::Network(format!("Invalid MIME type: {}", e)))?;
        let form = reqwest::multipart::Form::new().part("avatar", part);
        let rb = self.client.post(&url).multipart(form);
        let rb = self.apply_signing(rb, "POST", &url, &path, &[]);

        let resp = rb.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let text = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;

        if !is_success {
            return Err(ApiError::Http { status, body: text });
        }
        serde_json::from_str(&text).map_err(|e| ApiError::Deserialize(e.to_string()))
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
