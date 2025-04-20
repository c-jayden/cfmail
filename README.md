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

## 安装

### 预编译二进制文件

从[发布页面](https://github.com/yourusername/cfmail/releases)下载适用于您操作系统的最新版本。

### 从源码构建

确保您已安装 [Rust](https://www.rust-lang.org/tools/install) 工具链。

```bash
git clone https://github.com/yourusername/cfmail.git
cd cfmail
cargo build --release
```

编译后的二进制文件将位于 `target/release` 目录中。

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