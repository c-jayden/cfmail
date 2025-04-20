use crate::config::Config;
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use imap::Session;
use mail_parser::Message;
use regex::Regex;

/// 验证码类型
#[derive(Debug, Clone, Copy)]
pub enum CodeType {
    /// 纯数字
    Numeric,
    /// 字母数字混合
    Alphanumeric,
    /// 任意字符
    Any,
}

impl CodeType {
    /// 获取对应的正则表达式模式
    fn pattern(&self, length: Option<usize>) -> String {
        match self {
            Self::Numeric => {
                if let Some(len) = length {
                    format!(r"\b\d{{{}}}\b", len)
                } else {
                    r"\b\d{4,8}\b".to_string()
                }
            }
            Self::Alphanumeric => {
                if let Some(len) = length {
                    format!(r"\b[A-Za-z0-9]{{{}}}\b", len)
                } else {
                    r"\b[A-Za-z0-9]{4,8}\b".to_string()
                }
            }
            Self::Any => {
                if let Some(len) = length {
                    format!(r"\b\S{{{}}}\b", len)
                } else {
                    r"\b\S{4,8}\b".to_string()
                }
            }
        }
    }
}

/// 邮箱监听配置选项
pub struct MonitorOptions {
    /// 验证码长度
    pub code_length: Option<usize>,
    /// 验证码类型
    pub code_type: CodeType,
    /// 发件人过滤
    pub from_filter: Option<String>,
    /// 超时时间（秒）
    pub timeout: u64,
    /// 轮询间隔（秒）
    pub poll_interval: u64,
}

impl Default for MonitorOptions {
    fn default() -> Self {
        Self {
            code_length: None,
            code_type: CodeType::Numeric,
            from_filter: None,
            timeout: 300,      // 5分钟超时
            poll_interval: 10, // 10秒轮询一次
        }
    }
}

/// 邮件验证码结果
pub struct CodeResult {
    /// 提取到的验证码
    pub code: String,
    /// 来源邮件标题
    pub subject: String,
    /// 发件人
    pub from: String,
}

/// 邮件监听器
pub struct MailMonitor<'a> {
    config: &'a Config,
    options: MonitorOptions,
}

impl<'a> MailMonitor<'a> {
    /// 创建新的邮件监听器
    pub fn new(config: &'a Config, options: MonitorOptions) -> Self {
        Self { config, options }
    }

    /// 连接到IMAP服务器
    fn connect_imap(&self) -> Result<Session<native_tls::TlsStream<std::net::TcpStream>>> {
        let tls = native_tls::TlsConnector::builder()
            .min_protocol_version(Some(native_tls::Protocol::Tlsv12)) // 强制使用TLS 1.2或更高版本
            .build()
            .context("无法创建TLS连接器")?;

        // 连接到服务器
        let client = imap::connect(
            (
                self.config.smtp.imap_server.as_str(),
                self.config.smtp.imap_port,
            ),
            self.config.smtp.imap_server.as_str(),
            &tls,
        )
        .context("无法连接到IMAP服务器")?;

        // 登录
        let mut imap_session = client
            .login(&self.config.smtp.username, &self.config.smtp.password)
            .map_err(|e| anyhow!("IMAP登录失败: {:?}", e.0))?;

        // 选择收件箱
        imap_session.select("INBOX").context("无法选择收件箱")?;

        Ok(imap_session)
    }

    /// 创建验证码提取正则表达式
    fn create_regex(&self) -> Result<Regex> {
        let pattern = self.options.code_type.pattern(self.options.code_length);
        Regex::new(&pattern).context("无法创建正则表达式")
    }

    /// 获取消息的纯文本内容
    fn get_message_text(&self, message: &Message) -> Option<String> {
        message.text_bodies().next().and_then(|part| {
            std::str::from_utf8(part.contents())
                .ok()
                .map(|s| s.to_string())
        })
    }

    /// 获取消息的HTML内容
    fn get_message_html(&self, message: &Message) -> Option<String> {
        message.html_bodies().next().and_then(|part| {
            std::str::from_utf8(part.contents())
                .ok()
                .map(|s| s.to_string())
        })
    }

    /// 从文本中提取验证码
    fn extract_code_from_text(&self, text: &str, regex: &Regex) -> Option<String> {
        // 针对"Enter the code XXX"或"code below: XXX"这类格式的特殊处理
        let special_patterns = [
            // "Enter the code 123456"格式
            r"(?i)Enter the code[^0-9]*?([0-9]{4,8})\b",
            // "The verification code is 123456"格式
            r"(?i)verification code[^0-9]*?([0-9]{4,8})\b",
            // "Your code is 123456"格式
            r"(?i)your code[^0-9]*?([0-9]{4,8})\b",
            // "Code: 123456"格式
            r"(?i)code[^0-9]*?([0-9]{4,8})\b",
            // 纯数字块 - 通常邮件中单独一行的数字很可能是验证码
            r"(?m)^[ \t]*([0-9]{4,8})[ \t]*$",
        ];

        // 先尝试用特殊模式匹配
        for pattern in special_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        return Some(m.as_str().to_string());
                    }
                }
            }
        }

        // 改进匹配逻辑，避免匹配到年份或其他非验证码数字
        // 1. 优先查找验证码标记后的数字
        let code_markers = [
            "验证码",
            "code",
            "密码",
            "校验码",
            "动态码",
            "口令",
            "码",
            "验证代码",
            "动态密码",
            "验证",
            "verification",
        ];

        // 先尝试查找带有常见提示词的验证码
        for marker in code_markers {
            let marker_regex = format!(r"{}[\s:：]*([0-9A-Za-z]{{4,8}})", regex::escape(marker));
            if let Ok(r) = Regex::new(&marker_regex) {
                if let Some(caps) = r.captures(text) {
                    if let Some(m) = caps.get(1) {
                        return Some(m.as_str().to_string());
                    }
                }
            }
        }

        // 如果没找到带标记的验证码，再使用通用模式查找
        regex
            .captures(text)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    /// 从邮件中提取验证码
    fn extract_code_from_message(&self, message: &Message, regex: &Regex) -> Option<String> {
        // 首先尝试从纯文本中提取
        if let Some(text) = self.get_message_text(message) {
            if let Some(code) = self.extract_code_from_text(&text, regex) {
                return Some(code);
            }
        }

        // 如果纯文本中没有找到，尝试从HTML中提取
        if let Some(html) = self.get_message_html(message) {
            let text = html2text::from_read(html.as_bytes(), 80);
            if let Some(code) = self.extract_code_from_text(&text, regex) {
                return Some(code);
            }
        }

        None
    }

    /// 检查邮件是否符合过滤条件
    #[allow(dead_code)]
    fn is_matching_filter(&self, message: &Message) -> bool {
        // 检查发件人过滤
        if let Some(filter) = &self.options.from_filter {
            // 获取发件人和检查匹配
            let from_matches = message
                .headers()
                .iter()
                .filter(|header| header.name().eq_ignore_ascii_case("from"))
                .any(|header| {
                    // value()直接返回HeaderValue而不是Option<T>
                    let value = header.value();
                    let value_str = value.as_text_ref().unwrap_or("");
                    value_str.to_lowercase().contains(&filter.to_lowercase())
                });

            if !from_matches {
                return false;
            }
        }

        true
    }

    /// 获取消息接收时间
    #[allow(dead_code)]
    fn get_message_date(&self, message: &Message) -> Option<DateTime<Utc>> {
        // 尝试获取日期头信息
        for header in message.headers() {
            if header.name().eq_ignore_ascii_case("date") {
                // value()直接返回HeaderValue而不是Option<T>
                let value = header.value();
                if let Some(date_str) = value.as_text_ref() {
                    // 尝试从字符串解析日期
                    if let Ok(date) = chrono::DateTime::parse_from_rfc2822(date_str) {
                        return Some(date.with_timezone(&Utc));
                    }
                }
            }
        }

        // 如果没有有效日期，使用当前时间
        Some(Utc::now())
    }

    /// 获取邮件标题
    fn get_message_subject(&self, message: &Message) -> String {
        for header in message.headers() {
            if header.name().eq_ignore_ascii_case("subject") {
                let value = header.value();
                if let Some(subject) = value.as_text_ref() {
                    return subject.to_string();
                }
            }
        }

        "[无标题]".to_string()
    }

    /// 获取发件人
    fn get_message_sender(&self, message: &Message) -> String {
        // 尝试使用 mail_parser API 获取发件人
        // 1. 尝试通过 from 方法获取
        let from_value = message.from();
        if let Some(from_text) = from_value.as_text_ref() {
            println!("[调试] from()方法成功: {}", from_text);
            // 尝试解析复杂的发件人格式
            if let Some(email_start) = from_text.find('<') {
                if let Some(email_end) = from_text.find('>') {
                    if email_start < email_end {
                        let name = from_text[0..email_start].trim();
                        let email = &from_text[email_start + 1..email_end];
                        if !name.is_empty() && !email.is_empty() {
                            return format!("{} <{}>", name, email);
                        } else if !email.is_empty() {
                            return format!("<{}>", email);
                        } else if !name.is_empty() {
                            return name.to_string();
                        }
                    }
                }
            }
            return from_text.to_string();
        } else {
            println!("[调试] from()方法未返回文本内容");
        }

        // 2. 直接尝试查找From头部
        println!("[调试] 开始查找From头部");
        for header in message.headers() {
            println!("[调试] 检查头部: {}", header.name());
            if header.name().eq_ignore_ascii_case("from") {
                println!("[调试] 找到From头部");
                let value = header.value();
                println!("[调试] From头部值类型: {:?}", value);
                if let Some(from) = value.as_text_ref() {
                    println!("[调试] From文本内容: {}", from);
                    // 尝试解析复杂的发件人格式
                    if let Some(email_start) = from.find('<') {
                        if let Some(email_end) = from.find('>') {
                            if email_start < email_end {
                                let name = from[0..email_start].trim();
                                let email = &from[email_start + 1..email_end];
                                if !name.is_empty() && !email.is_empty() {
                                    return format!("{} <{}>", name, email);
                                } else if !email.is_empty() {
                                    return format!("<{}>", email);
                                } else if !name.is_empty() {
                                    return name.to_string();
                                }
                            }
                        }
                    }
                    return from.to_string();
                } else {
                    println!("[调试] From头部无法提取文本");
                }
            }
        }

        // 3. 打印所有可用的头部，以便调试
        println!("[调试] 没有找到有效的From头部。所有头部列表:");
        for header in message.headers() {
            println!("[调试] 头部名称: {}", header.name());
            if let Some(text) = header.value().as_text_ref() {
                println!("[调试]   值: {}", text);
            } else {
                println!("[调试]   值: <无法提取文本>");
            }
        }

        "[未知发件人]".to_string()
    }

    /// 等待验证码邮件
    pub fn wait_for_code(&self) -> Result<CodeResult> {
        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(self.options.timeout);
        let poll_interval = std::time::Duration::from_secs(self.options.poll_interval);

        // 创建正则表达式
        let regex = self.create_regex()?;

        while start_time.elapsed() < timeout_duration {
            // 连接IMAP服务器
            let mut imap_session = match self.connect_imap() {
                Ok(session) => session,
                Err(e) => {
                    eprintln!("连接IMAP服务器失败: {}", e);
                    std::thread::sleep(poll_interval);
                    continue;
                }
            };

            // 搜索未读邮件
            let search_criteria = if let Some(ref sender) = self.options.from_filter {
                format!("UNSEEN FROM \"{}\"", sender)
            } else {
                "UNSEEN".to_string()
            };

            let seq_nums = match imap_session.search(&search_criteria) {
                Ok(nums) => nums,
                Err(e) => {
                    eprintln!("搜索邮件失败: {:?}", e);
                    let _ = imap_session.logout();
                    std::thread::sleep(poll_interval);
                    continue;
                }
            };

            if !seq_nums.is_empty() {
                // 从新到旧排序
                let mut seq_nums_vec: Vec<_> = seq_nums.iter().collect();
                seq_nums_vec.sort_by(|a, b| b.cmp(a));

                // 处理邮件
                for seq_num in seq_nums_vec {
                    let fetch_result = match imap_session.fetch(seq_num.to_string(), "RFC822") {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("获取邮件内容失败: {:?}", e);
                            continue;
                        }
                    };

                    for message in fetch_result.iter() {
                        if let Some(body) = message.body() {
                            if let Some(mail) = mail_parser::Message::parse(body) {
                                // 尝试从邮件中提取验证码
                                if let Some(code) = self.extract_code_from_message(&mail, &regex) {
                                    // 从邮件头获取发件人信息
                                    let from = self.get_message_sender(&mail);
                                    // 从邮件头获取主题
                                    let subject = self.get_message_subject(&mail);

                                    // 安全地退出IMAP会话
                                    let _ = imap_session.logout();

                                    return Ok(CodeResult {
                                        code,
                                        from,
                                        subject,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // 安全地退出IMAP会话
            let _ = imap_session.logout();

            // 等待下一次轮询
            std::thread::sleep(poll_interval);
        }

        Err(anyhow!("超时: 未找到包含验证码的邮件"))
    }
}
