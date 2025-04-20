use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use reqwest;

use super::models::{
    CloudflareResponse, EmailRoute, EmailRouteAction, EmailRouteCreate, EmailRouteMatcher,
};

/// Cloudflare API客户端，用于操作Email Routing服务
pub struct CloudflareClient {
    client: reqwest::Client,
    zone_id: String,
    target_email: String,
}

impl CloudflareClient {
    /// 创建新的Cloudflare客户端
    ///
    /// 根据配置选择合适的认证方式
    pub fn new(config: &Config) -> Self {
        // 根据配置的认证类型选择认证方式
        let client = match config.cloudflare.auth_type.as_str() {
            "api_token" => Self::create_token_client(&config).expect("无法创建API Token客户端"),
            _ => Self::create_key_client(&config).expect("无法创建API Key客户端"),
        };

        Self {
            client,
            zone_id: config.cloudflare.zone_id.clone(),
            target_email: config.email.target_email.clone(),
        }
    }

    /// 使用API Token创建客户端
    fn create_token_client(config: &Config) -> Result<reqwest::Client> {
        let mut headers = reqwest::header::HeaderMap::new();

        // 安全处理：不在日志或错误消息中包含实际令牌
        if config.cloudflare.api_token.is_empty() {
            return Err(anyhow!("API Token不能为空"));
        }

        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer {}",
                config.cloudflare.api_token
            ))
            .context("无效的API Token")?,
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        Ok(reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("无法创建HTTP客户端")?)
    }

    /// 使用API Key创建客户端
    fn create_key_client(config: &Config) -> Result<reqwest::Client> {
        let mut headers = reqwest::header::HeaderMap::new();

        // 安全处理：检查但不输出敏感信息
        if config.cloudflare.email.is_empty() {
            return Err(anyhow!("Cloudflare邮箱不能为空"));
        }
        if config.cloudflare.api_key.is_empty() {
            return Err(anyhow!("API Key不能为空"));
        }

        headers.insert(
            "X-Auth-Email",
            reqwest::header::HeaderValue::from_str(&config.cloudflare.email)
                .context("无效的邮箱地址")?,
        );
        headers.insert(
            "X-Auth-Key",
            reqwest::header::HeaderValue::from_str(&config.cloudflare.api_key)
                .context("无效的API Key")?,
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        Ok(reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("无法创建HTTP客户端")?)
    }

    /// 创建邮箱路由规则
    ///
    /// 将指定的邮箱别名转发到目标邮箱
    pub async fn create_email_route(&self, email_alias: &str) -> Result<()> {
        let email_route = EmailRouteCreate {
            matchers: vec![EmailRouteMatcher {
                matcher_type: "literal".to_string(),
                field: Some("to".to_string()),
                value: Some(email_alias.to_string()),
            }],
            actions: vec![EmailRouteAction {
                action_type: "forward".to_string(),
                value: vec![self.target_email.clone()],
            }],
            enabled: true,
            name: Some(format!("自动创建的转发规则: {}", email_alias)),
        };

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/email/routing/rules",
            self.zone_id
        );

        let response = self
            .client
            .post(&url)
            .json(&email_route)
            .send()
            .await
            .context("发送请求创建邮箱路由失败")?;

        // 获取响应状态和文本以便更好地诊断
        let status = response.status();
        let body = response.text().await.context("读取响应内容失败")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Cloudflare API错误: 状态码 {}，响应内容: {}",
                status,
                body
            ));
        }

        // 尝试解析JSON，使用EmailRoute结构作为响应类型
        match serde_json::from_str::<CloudflareResponse<EmailRoute>>(&body) {
            Ok(cf_response) => {
                if !cf_response.success {
                    let error_msg = cf_response
                        .errors
                        .into_iter()
                        .map(|e| format!("{}: {}", e.code, e.message))
                        .collect::<Vec<_>>()
                        .join(", ");

                    return Err(anyhow!("Cloudflare API错误: {}", error_msg));
                }

                // 创建成功
                Ok(())
            }
            Err(e) => Err(anyhow!("无法解析Cloudflare响应: {}，原始响应: {}", e, body)),
        }
    }

    /// 获取已配置的邮箱别名列表
    pub async fn list_email_routes(&self) -> Result<Vec<String>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/email/routing/rules",
            self.zone_id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("发送请求获取邮箱路由列表失败")?;

        // 获取响应状态和文本以便更好地诊断
        let status = response.status();
        let body = response.text().await.context("读取响应内容失败")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Cloudflare API错误: 状态码 {}，响应内容: {}",
                status,
                body
            ));
        }

        // 尝试解析JSON，如果失败则返回原始文本
        match serde_json::from_str::<CloudflareResponse<Vec<EmailRoute>>>(&body) {
            Ok(cf_response) => {
                if !cf_response.success {
                    let error_msg = cf_response
                        .errors
                        .into_iter()
                        .map(|e| format!("{}: {}", e.code, e.message))
                        .collect::<Vec<_>>()
                        .join(", ");

                    return Err(anyhow!("Cloudflare API错误: {}", error_msg));
                }

                let aliases = cf_response
                    .result
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|route| {
                        // 使用路由ID和名称(虽然在正常流程中不输出这些调试信息)
                        if cfg!(debug_assertions) {
                            println!("处理邮件路由: ID={}, 名称={}", route.id, route.name);
                        }

                        for matcher in route.matchers {
                            if matcher.matcher_type == "literal"
                                && matcher.field.as_deref() == Some("to")
                                && matcher.value.is_some()
                            {
                                return matcher.value;
                            }
                        }
                        None
                    })
                    .collect();

                Ok(aliases)
            }
            Err(e) => Err(anyhow!("无法解析Cloudflare响应: {}，原始响应: {}", e, body)),
        }
    }

    /// 删除指定的邮箱别名路由
    ///
    /// 根据邮箱别名找到对应的路由ID并删除
    pub async fn delete_email_route(&self, email_alias: &str) -> Result<()> {
        // 获取所有路由
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/email/routing/rules",
            self.zone_id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("发送请求获取邮箱路由列表失败")?;

        // 获取响应状态和文本以便更好地诊断
        let status = response.status();
        let body = response.text().await.context("读取响应内容失败")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Cloudflare API错误: 状态码 {}，响应内容: {}",
                status,
                body
            ));
        }

        // 解析响应以查找匹配的路由ID
        let cf_response = serde_json::from_str::<CloudflareResponse<Vec<EmailRoute>>>(&body)
            .context("无法解析Cloudflare响应")?;

        if !cf_response.success {
            let error_msg = cf_response
                .errors
                .into_iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");

            return Err(anyhow!("Cloudflare API错误: {}", error_msg));
        }

        // 查找匹配的路由ID
        let route_id = cf_response
            .result
            .unwrap_or_default()
            .into_iter()
            .find_map(|route| {
                for matcher in route.matchers {
                    if matcher.matcher_type == "literal"
                        && matcher.field.as_deref() == Some("to")
                        && matcher.value.as_deref() == Some(email_alias)
                    {
                        return Some(route.id);
                    }
                }
                None
            })
            .ok_or_else(|| anyhow!("未找到匹配的邮箱别名: {}", email_alias))?;

        // 删除找到的路由
        let delete_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/email/routing/rules/{}",
            self.zone_id, route_id
        );

        let delete_response = self
            .client
            .delete(&delete_url)
            .send()
            .await
            .context("发送请求删除邮箱路由失败")?;

        // 获取响应状态和文本以便更好地诊断
        let delete_status = delete_response.status();
        let delete_body = delete_response.text().await.context("读取响应内容失败")?;

        if !delete_status.is_success() {
            return Err(anyhow!(
                "Cloudflare API错误: 状态码 {}，响应内容: {}",
                delete_status,
                delete_body
            ));
        }

        // 解析删除响应
        let delete_cf_response =
            serde_json::from_str::<CloudflareResponse<serde_json::Value>>(&delete_body)
                .context("无法解析Cloudflare删除响应")?;

        if !delete_cf_response.success {
            let error_msg = delete_cf_response
                .errors
                .into_iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");

            return Err(anyhow!("Cloudflare API删除错误: {}", error_msg));
        }

        // 删除成功
        Ok(())
    }
}
