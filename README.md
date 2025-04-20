# CFMAIL - Cloudflare 邮箱别名管理工具

<p align="center">
  <img src="docs/assets/cfmail-logo.png" alt="CFMAIL Logo" width="400"/>
</p>

<p align="center">
  <strong>一个强大的邮箱别名生成和管理工具，基于Cloudflare Email Routing服务</strong>
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#安装">安装</a> •
  <a href="#使用方法">使用方法</a> •
  <a href="#安全建议">安全建议</a> •
  <a href="#性能和安全优化">性能和安全优化</a> •
  <a href="#贡献">贡献</a> •
  <a href="#许可证">许可证</a>
</p>

## 功能特性

- **邮箱别名生成** - 快速创建自定义或随机邮箱别名
- **别名管理** - 列出和管理所有已创建的邮箱别名
- **验证码监听** - 自动监听邮箱中的验证码并提取
- **现代CLI界面** - 美观的命令行界面，提供良好的用户体验
- **配置灵活** - 支持多种验证码类型和自定义过滤条件
- **多语言支持** - 支持中文和英文界面，自动检测系统语言

## 安装

### 预编译二进制文件

从[发布页面](https://github.com/yourusername/cfmail/releases)下载适用于您操作系统的最新版本。

### 跨平台支持

CFMAIL支持以下平台：
- Windows (x86_64)
- macOS (x86_64)
- Linux (x86_64)

### 从源码构建

确保您已安装 [Rust](https://www.rust-lang.org/tools/install) 工具链。

```bash
git clone https://github.com/yourusername/cfmail.git
cd cfmail
cargo build --release
```

编译后的二进制文件将位于 `target/release` 目录中。

### 交叉编译

项目包含了用于交叉编译的脚本，可以在一个平台上编译出多个平台的二进制文件：

```bash
# 确保脚本有执行权限
chmod +x cross-build.sh
# 运行交叉编译脚本
./cross-build.sh
```

对于Windows交叉编译，您可能需要安装额外的工具：

```bash
# 在macOS上
brew install mingw-w64

# 在Linux上
sudo apt install mingw-w64
```

## 使用方法

### 配置

首次使用前，需要创建配置文件。请在以下位置创建 `config.toml` 文件：

- 用户目录下的 `~/.config/cfmail/config.toml`

您可以使用内置的初始化命令创建配置文件模板：

```bash
cfmail init
```

然后编辑生成的配置文件，填入您的Cloudflare和邮箱信息。

配置文件示例：

```toml
# Cloudflare API配置
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
```

### 基本命令

#### 生成新的邮箱别名

```bash
cfmail generate
```

使用自定义前缀：

```bash
cfmail generate --prefix newsletter
```

#### 列出已有的邮箱别名

```bash
cfmail list
```

#### 监听验证码

```bash
cfmail watch-code
```

指定验证码长度和类型：

```bash
cfmail watch-code --length 6 --code-type numeric
```

带有过滤条件：

```bash
cfmail watch-code --from example.com
```

#### 切换语言

程序支持英文和中文界面，可以通过以下方式切换：

```bash
# 使用英文
cfmail --locale en generate

# 使用中文
cfmail --locale zh list

# 语言参数可以组合任何命令
cfmail --locale zh watch-code --from example.com
```

程序会自动检测系统语言设置，如果系统设置为中文环境（LANG=zh_CN.*），则自动使用中文界面；否则默认使用英文界面。

您也可以通过设置环境变量来切换语言，无需在每个命令中使用`--locale`参数：

```bash
# Linux/macOS
export LANG=zh_CN.UTF-8
cfmail generate

# Windows PowerShell
$env:LANG = "zh_CN.UTF-8"
cfmail generate

# Windows CMD
set LANG=zh_CN.UTF-8
cfmail generate
```

#### 自定义语言

本程序的语言文件存储在`locales`目录下，采用JSON格式。如果您想添加新的语言，可以：

1. 复制`locales/en-US.json`文件，重命名为您想添加的语言代码（如`ja-JP.json`）
2. 翻译文件中的所有文本
3. 修改`src/util/i18n.rs`文件，添加对应的语言支持

```rust
// 在SupportedLocale枚举中添加
pub enum SupportedLocale {
    EnUS,
    ZhCN,
    JaJP,  // 新增日语
}

// 添加对应的str转换
pub fn as_str(&self) -> &'static str {
    match self {
        SupportedLocale::EnUS => "en-US",
        SupportedLocale::ZhCN => "zh-CN",
        SupportedLocale::JaJP => "ja-JP", // 新增
    }
}

// 添加字符串识别
pub fn from_str(s: &str) -> Option<Self> {
    match s.to_lowercase().as_str() {
        "en" | "en-us" | "en_us" | "english" => Some(SupportedLocale::EnUS),
        "zh" | "zh-cn" | "zh_cn" | "chinese" => Some(SupportedLocale::ZhCN),
        "ja" | "ja-jp" | "ja_jp" | "japanese" => Some(SupportedLocale::JaJP), // 新增
        _ => None,
    }
}

// 添加语言显示名称
pub fn get_current_locale_name() -> &'static str {
    match get_current_locale() {
        SupportedLocale::EnUS => "English",
        SupportedLocale::ZhCN => "简体中文",
        SupportedLocale::JaJP => "日本語", // 新增
    }
}

// 更新列表
pub fn list_supported_locales() -> Vec<(&'static str, &'static str)> {
    vec![
        ("en", "English"),
        ("zh", "简体中文"),
        ("ja", "日本語"), // 新增
    ]
}
```

## 安全建议

为确保您的账户安全，请注意以下几点：

1. **配置文件安全**：
   - 配置文件存储在您的用户目录下，确保设置合适的文件权限（建议权限为0600）
   - 永远不要将配置文件提交到公开的代码仓库
   - 对于Gmail等邮箱服务，建议使用应用专用密码而非主密码

2. **API密钥管理**：
   - 为Cloudflare账户创建专用的API令牌，并限制其权限
   - 定期轮换API令牌
   - 如果不再使用该工具，及时撤销相关API令牌

3. **别名使用建议**：
   - 针对不同服务使用不同的邮箱别名，以便追踪数据泄露来源
   - 不再使用的别名及时删除

## 性能和安全优化

本项目在设计上已实施以下安全和性能优化：

1. **配置安全**：
   - 配置文件仅存储在用户主目录下，不在代码仓库中
   - 在Unix系统上创建配置文件时使用权限0600（仅所有者可读写）
   - 配置文件中的敏感信息（API密钥、密码等）不会在日志中输出

2. **网络安全**：
   - IMAP连接使用TLS 1.2或更高版本加密
   - 在连接结束后安全地登出IMAP会话
   - API请求验证输入数据，防止发送空凭据

3. **性能优化**：
   - 验证码监听使用轮询间隔，避免过于频繁的请求
   - 尝试从纯文本内容中提取验证码，仅在必要时处理HTML内容
   - 优化的正则表达式匹配，减少资源消耗

## 贡献

欢迎提交问题和拉取请求！详情请参阅[贡献指南](CONTRIBUTING.md)。

## 许可证

[MIT](LICENSE) 