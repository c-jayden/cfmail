use serde::{Deserialize, Serialize};

// 邮件路由创建请求
#[derive(Debug, Serialize)]
pub struct EmailRouteCreate {
    #[serde(rename = "matchers")]
    pub matchers: Vec<EmailRouteMatcher>,
    #[serde(rename = "actions")]
    pub actions: Vec<EmailRouteAction>,
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// 匹配器定义
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailRouteMatcher {
    #[serde(rename = "type")]
    pub matcher_type: String,
    #[serde(rename = "field", default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(rename = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

// 动作定义
#[derive(Debug, Serialize)]
pub struct EmailRouteAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(rename = "value")]
    pub value: Vec<String>,
}

// Cloudflare API通用响应格式
#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    #[serde(rename = "success")]
    pub success: bool,
    #[serde(rename = "errors")]
    pub errors: Vec<CloudflareError>,
    #[serde(rename = "result")]
    pub result: Option<T>,
}

// Cloudflare错误定义
#[derive(Debug, Deserialize)]
pub struct CloudflareError {
    #[serde(rename = "code")]
    pub code: i32,
    #[serde(rename = "message")]
    pub message: String,
}

// 邮件路由规则
#[derive(Debug, Deserialize)]
pub struct EmailRoute {
    #[serde(rename = "id")]
    #[allow(dead_code)]
    pub id: String,
    #[serde(rename = "matchers")]
    pub matchers: Vec<EmailRouteMatcher>,
    #[serde(rename = "name", default)]
    #[allow(dead_code)]
    pub name: String,
}
