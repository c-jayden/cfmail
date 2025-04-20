# 发布指南

本文档描述了如何使用GitHub Actions在GitHub上发布CFMAIL的新版本。

## 发布流程

CFMAIL使用GitHub Actions自动构建和发布多平台的二进制文件。发布流程设计为简洁直观：

1. 当您推送新的标签（以`v`开头，如`v0.1.0`）时，GitHub Actions会自动触发构建流程
2. GitHub Actions会为Windows、macOS和Linux平台编译二进制文件
3. 编译好的二进制文件会自动上传到相应的GitHub Release页面

## 如何发布新版本

### 准备工作

1. 确保所有更改已经合并到主分支
2. 更新`Cargo.toml`中的版本号
3. 更新CHANGELOG.md（如果有）
4. 确保所有测试通过

### 发布步骤

1. 创建一个新的Git标签，使用语义化版本号：

```bash
git tag -a v0.1.0 -m "版本0.1.0"
```

2. 推送标签到GitHub：

```bash
git push origin v0.1.0
```

3. GitHub Actions将自动触发并开始构建流程

4. 构建完成后，您可以在GitHub的Releases页面看到新版本，包含以下文件：
   - cfmail-windows-amd64.exe (Windows可执行文件)
   - cfmail-linux-amd64 (Linux可执行文件)
   - cfmail-macos-amd64 (macOS可执行文件)
   - cfmail-release.zip (包含所有平台二进制文件的压缩包)

5. 编辑自动创建的Release，添加版本说明和更新日志

## 手动构建

如果您需要在本地构建多平台二进制文件，可以使用项目根目录下的`cross-build.sh`脚本：

```bash
./cross-build.sh
```

这个脚本会为所有支持的平台构建二进制文件，并将结果放在`release`目录中。

## 注意事项

- 确保标签以`v`开头，否则不会触发GitHub Actions构建
- 发布版本应遵循[语义化版本](https://semver.org/)规范
- 发布前确保在本地测试，以免发布有问题的版本
- 如果GitHub Actions构建失败，检查Actions日志以了解详细错误信息 