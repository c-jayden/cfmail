# 贡献指南

感谢您考虑为CFMAIL项目做出贡献！本文档提供了参与项目开发的指南。

## 提交问题

如果您发现bug或有功能请求，请先查看[现有issues](https://github.com/yourusername/cfmail/issues)，确保不存在重复问题。如果没有相关问题，请创建一个新的issue，并提供以下信息：

### Bug报告
- 清晰的bug描述
- 复现步骤
- 期望行为与实际行为
- 系统环境（操作系统、Rust版本等）
- 如可能，附上代码片段或错误日志

### 功能请求
- 清晰的功能描述
- 功能的使用场景
- 如可能，提供功能的实现思路

## 代码贡献

我们欢迎通过Pull Request(PR)提交代码贡献。请遵循以下步骤：

1. Fork本仓库
2. 创建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启一个Pull Request

## 代码规范

- 遵循Rust标准代码风格，使用`rustfmt`格式化代码
- 使用有意义的变量名和函数名
- 为公共API编写文档注释
- 编写单元测试和集成测试
- 确保您的代码通过`cargo clippy`检查

## 提交信息规范

- 使用现在时态 ("Add feature" 而非 "Added feature")
- 第一行应简明扼要，不超过72个字符
- 可选地在提交信息中引用相关的issue编号

例如：
```
Add alias expiration feature (#42)

- Add expiration date to alias model
- Implement automatic cleanup for expired aliases
- Update API to support expiration parameter
```

## 开发环境设置

确保您已安装以下工具：
- [Rust](https://www.rust-lang.org/tools/install) (推荐使用rustup)
- [Clippy](https://github.com/rust-lang/rust-clippy) - Rust代码质量工具
- [Rustfmt](https://github.com/rust-lang/rustfmt) - Rust代码格式化工具

## 许可证

通过贡献代码，您同意您的贡献将按照项目的[MIT许可证](LICENSE)进行许可。 