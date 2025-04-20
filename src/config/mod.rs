use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub cloudflare: CloudflareConfig,
    pub email: EmailConfig,
    pub alias: AliasConfig,
    pub smtp: SmtpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudflareConfig {
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    pub api_key: String,
    pub api_token: String,
    pub zone_id: String,
    pub email: String,
}

fn default_auth_type() -> String {
    "api_key".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub domain: String,
    pub target_email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AliasConfig {
    pub prefix_mode: String,
    pub custom_prefixes: Vec<String>,
    pub random_length: usize,
    pub random_charset: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    pub username: String,
    pub password: String,
    pub imap_server: String,
    pub imap_port: u16,
    #[allow(dead_code)]
    pub smtp_server: String,
    #[allow(dead_code)]
    pub smtp_port: u16,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_file()?;
        let mut file = File::open(&config_path)
            .with_context(|| format!("无法打开配置文件: {}", config_path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| "无法读取配置文件内容")?;

        toml::from_str(&contents).with_context(|| "解析配置文件失败")
    }

    pub fn init() -> Result<std::path::PathBuf> {
        let config_dir = dirs::home_dir()
            .context("无法获取用户主目录")?
            .join(".config/cfmail");

        let config_path = config_dir.join("config.toml");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("无法创建配置目录: {}", config_dir.display()))?;
        }

        if config_path.exists() {
            return Ok(config_path);
        }

        let mut file = {
            #[cfg(unix)]
            {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .mode(0o600)
                    .open(&config_path)
            }
            #[cfg(not(unix))]
            {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&config_path)
            }
        }
        .with_context(|| format!("无法创建配置文件: {}", config_path.display()))?;

        let template = r#"# Cloudflare API配置
[cloudflare]
auth_type = "api_key"
api_token = "您的API令牌"
api_key = "您的API密钥"
zone_id = "您的区域ID"
email = "您的Cloudflare账户邮箱"

# 邮箱配置
[email]
domain = "您的自定义域名"
target_email = "您的目标邮箱"

# 别名生成配置
[alias]
prefix_mode = "random"
custom_prefixes = ["support", "contact", "info"]
random_length = 8
random_charset = "alphabetic"

# SMTP和IMAP配置
[smtp]
username = "您的邮箱地址"
password = "您的邮箱密码"
imap_server = "imap.example.com"
imap_port = 993
smtp_server = "smtp.example.com"
smtp_port = 587
"#;

        file.write_all(template.as_bytes())
            .with_context(|| "无法写入配置模板")?;

        Ok(config_path)
    }

    fn find_config_file() -> Result<std::path::PathBuf> {
        let config_dir = dirs::home_dir()
            .context("无法获取用户主目录")?
            .join(".config/cfmail");

        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            return Ok(config_path);
        }

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .with_context(|| format!("无法创建配置目录: {}", config_dir.display()))?;
        }

        Err(anyhow::anyhow!(
            "找不到配置文件。请在 {} 目录下创建config.toml文件。",
            config_dir.display()
        ))
    }
}
