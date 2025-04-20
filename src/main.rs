mod api;
mod config;
mod service;
mod ui;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use prettytable;
use service::mail_monitor::{CodeType, MailMonitor, MonitorOptions};
use std::io::{self, BufRead, Write};

#[derive(Clone, ValueEnum, Debug)]
enum CodeTypeArg {
    /// 数字验证码
    Numeric,
    /// 字母数字混合验证码
    Alphanumeric,
    /// 任意字符验证码
    Any,
}

impl From<CodeTypeArg> for CodeType {
    fn from(arg: CodeTypeArg) -> Self {
        match arg {
            CodeTypeArg::Numeric => CodeType::Numeric,
            CodeTypeArg::Alphanumeric => CodeType::Alphanumeric,
            CodeTypeArg::Any => CodeType::Any,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about = "Cloudflare邮箱别名生成和管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 生成新的邮箱别名
    Generate {
        /// 自定义前缀（可选）
        #[arg(short, long)]
        prefix: Option<String>,
    },
    /// 列出所有配置的邮箱别名
    List,
    /// 删除邮箱别名
    Delete,
    /// 监听验证码邮件
    WatchCode {
        /// 验证码长度
        #[arg(short, long)]
        length: Option<usize>,

        /// 验证码类型
        #[arg(short, long, value_enum, default_value_t = CodeTypeArg::Numeric)]
        code_type: CodeTypeArg,

        /// 发件人过滤
        #[arg(short, long)]
        from: Option<String>,

        /// 超时时间（秒）
        #[arg(short, long, default_value_t = 300)]
        timeout: u64,

        /// 轮询间隔（秒）
        #[arg(long, default_value_t = 3)]
        poll_interval: u64,
    },
    /// 初始化配置文件
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 显示应用程序标题
    ui::print_app_header();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { prefix } => {
            ui::print_module_header("生成邮箱别名");

            // 加载配置
            let spinner = ui::create_spinner("正在加载配置...");
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, "配置加载成功");
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("配置加载失败: {}", e));
                    ui::print_info("请确保在 ~/.config/cfmail/config.toml 创建了配置文件");
                    return Err(e);
                }
            };

            // 生成邮箱别名
            let spinner = ui::create_spinner("正在生成邮箱别名...");
            let email_alias = match service::email::generate_alias(&cfg, prefix.clone()) {
                Ok(alias) => {
                    ui::spinner_success(&spinner, &format!("邮箱别名生成成功: {}", alias));
                    alias
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("邮箱别名生成失败: {}", e));
                    return Err(e);
                }
            };

            // 创建Cloudflare邮件路由
            let spinner = ui::create_spinner("正在配置Cloudflare邮件路由...");
            let cf_client = api::cloudflare::CloudflareClient::new(&cfg);
            match cf_client.create_email_route(&email_alias).await {
                Ok(_) => {
                    ui::spinner_success(&spinner, "邮件路由配置成功");
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("邮件路由配置失败: {}", e));
                    return Err(e);
                }
            }

            // 复制到剪贴板
            if let Err(e) = util::clipboard::copy_to_clipboard(&email_alias) {
                ui::print_error(&format!("无法复制到剪贴板: {}", e));
            } else {
                ui::print_success("邮箱别名已复制到剪贴板");
            }

            // 显示最终结果
            ui::print_result_box("生成结果", &email_alias);
            ui::print_info("您可以立即使用此邮箱地址接收邮件");
        }
        Commands::List => {
            ui::print_module_header("邮箱别名列表");

            // 加载配置
            let spinner = ui::create_spinner("正在加载配置...");
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, "配置加载成功");
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("配置加载失败: {}", e));
                    return Err(e);
                }
            };

            // 获取邮箱别名列表
            let spinner = ui::create_spinner("正在获取邮箱别名列表...");
            let cf_client = api::cloudflare::CloudflareClient::new(&cfg);
            match cf_client.list_email_routes().await {
                Ok(aliases) => {
                    ui::spinner_success(
                        &spinner,
                        &format!("成功获取到 {} 个邮箱别名", aliases.len()),
                    );

                    // 添加换行，确保表格框不会和上面的消息在同一行
                    println!();

                    if aliases.is_empty() {
                        ui::print_warning("未找到邮箱别名");
                    } else {
                        ui::print_aliases_table(&aliases);
                    }
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("获取邮箱别名列表失败: {}", e));
                    return Err(e);
                }
            }
        }
        Commands::Delete => {
            ui::print_module_header("删除邮箱别名");

            // 加载配置
            let spinner = ui::create_spinner("正在加载配置...");
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, "配置加载成功");
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("配置加载失败: {}", e));
                    return Err(e);
                }
            };

            // 获取邮箱别名列表
            let spinner = ui::create_spinner("正在获取邮箱别名列表...");
            let cf_client = api::cloudflare::CloudflareClient::new(&cfg);
            let aliases = match cf_client.list_email_routes().await {
                Ok(aliases) => {
                    ui::spinner_success(
                        &spinner,
                        &format!("成功获取到 {} 个邮箱别名", aliases.len()),
                    );
                    aliases
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("获取邮箱别名列表失败: {}", e));
                    return Err(e);
                }
            };

            if aliases.is_empty() {
                ui::print_warning("未找到任何邮箱别名，无需删除");
                return Ok(());
            }

            // 显示所有别名列表
            ui::print_aliases_table(&aliases);

            println!();
            ui::print_info("请输入要删除的邮箱别名编号（多个编号用空格分隔，输入 'all' 全选）:");

            // 读取用户输入
            print!("请输入 > ");
            io::stdout().flush()?;

            let stdin = io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;

            let input = input.trim();

            // 解析用户输入，获取选中的索引
            let mut selected_indices = Vec::new();

            if input.to_lowercase() == "all" {
                // 全选
                selected_indices.extend(0..aliases.len());
                ui::print_warning(&format!("您选择了删除所有 {} 个邮箱别名", aliases.len()));
            } else {
                // 解析用户输入的编号
                for num_str in input.split_whitespace() {
                    if let Ok(mut num) = num_str.parse::<usize>() {
                        // 用户输入从1开始，但索引从0开始
                        num = num.saturating_sub(1);
                        if num < aliases.len() {
                            selected_indices.push(num);
                        } else {
                            ui::print_warning(&format!("忽略无效编号: {}", num_str));
                        }
                    } else {
                        ui::print_warning(&format!("忽略无效输入: {}", num_str));
                    }
                }

                // 去重
                selected_indices.sort();
                selected_indices.dedup();

                if selected_indices.is_empty() {
                    ui::print_info("未选择任何邮箱别名，操作已取消");
                    return Ok(());
                }

                ui::print_warning(&format!(
                    "您选择了删除 {} 个邮箱别名",
                    selected_indices.len()
                ));
            }

            // 显示选中的别名
            println!();
            ui::print_info("以下邮箱别名将被删除:");
            for &idx in &selected_indices {
                println!("  {} {}", "•".yellow(), aliases[idx].cyan());
            }

            // 确认删除
            println!();
            ui::print_warning("此操作不可撤销，请确认是否继续? (y/N)");
            print!("请输入 > ");
            io::stdout().flush()?;

            let mut confirm = String::new();
            stdin.lock().read_line(&mut confirm)?;

            if !confirm.trim().eq_ignore_ascii_case("y") {
                ui::print_info("操作已取消");
                return Ok(());
            }

            // 开始删除操作
            let spinner = ui::create_spinner("正在删除邮箱别名...");
            let mut success_count = 0;
            let mut failed_aliases = Vec::new();

            for &index in &selected_indices {
                let alias = &aliases[index];
                match cf_client.delete_email_route(alias).await {
                    Ok(_) => {
                        success_count += 1;
                    }
                    Err(e) => {
                        failed_aliases.push((alias.clone(), e.to_string()));
                    }
                }
            }

            if failed_aliases.is_empty() {
                ui::spinner_success(
                    &spinner,
                    &format!(
                        "成功删除 {}/{} 个邮箱别名",
                        success_count,
                        selected_indices.len()
                    ),
                );

                // 添加删除总结
                if success_count > 0 {
                    println!();
                    ui::print_card(
                        "删除操作总结",
                        &format!("共删除 {} 个邮箱别名", success_count),
                    );
                    ui::print_success("所有选中的邮箱别名已成功删除");
                }
            } else {
                ui::spinner_error(
                    &spinner,
                    &format!(
                        "删除部分失败: {}/{} 个邮箱别名",
                        success_count,
                        selected_indices.len()
                    ),
                );

                // 显示失败的别名
                println!();
                ui::print_card(
                    "删除失败的邮箱别名",
                    &format!("共 {} 个邮箱别名删除失败:", failed_aliases.len()),
                );

                // 创建表格
                let mut table = prettytable::Table::new();
                table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);

                // 添加标题行
                table.set_titles(prettytable::Row::new(vec![
                    prettytable::Cell::new("序号").style_spec("bFc"),
                    prettytable::Cell::new("邮箱别名").style_spec("bFc"),
                    prettytable::Cell::new("错误原因").style_spec("bFc"),
                ]));

                // 添加数据行
                for (i, (alias, error)) in failed_aliases.iter().enumerate() {
                    table.add_row(prettytable::Row::new(vec![
                        prettytable::Cell::new(&format!("{}.", i + 1)).style_spec("Fc"),
                        prettytable::Cell::new(alias).style_spec("Fy"),
                        prettytable::Cell::new(error).style_spec("Fr"),
                    ]));
                }

                // 打印表格
                table.printstd();
                println!();

                // 如果有成功删除的，显示成功删除的数量
                if success_count > 0 {
                    println!();
                    ui::print_success(&format!("成功删除 {} 个邮箱别名", success_count));
                }
            }
        }
        Commands::WatchCode {
            length,
            code_type,
            from,
            timeout,
            poll_interval,
        } => {
            ui::print_module_header("邮箱验证码监听");

            // 加载配置
            let spinner = ui::create_spinner("正在加载配置...");
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, "配置加载成功");
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("配置加载失败: {}", e));
                    ui::print_info("请确保在 ~/.config/cfmail/config.toml 创建了配置文件");
                    return Err(e);
                }
            };

            // 创建监听选项
            let options = MonitorOptions {
                code_length: *length,
                code_type: (*code_type).clone().into(),
                from_filter: from.clone(),
                timeout: *timeout,
                poll_interval: *poll_interval,
            };

            let monitor = MailMonitor::new(&cfg, options);

            // 信息提示
            let mut monitor_info = String::new();
            monitor_info.push_str(&format!("监听账户: {}\n", cfg.smtp.username));

            if let Some(len) = length {
                monitor_info.push_str(&format!("验证码长度: {}\n", len));
            } else {
                monitor_info.push_str("验证码长度: 自动检测 (4-8位)\n");
            }

            monitor_info.push_str(&format!("验证码类型: {:?}\n", code_type));

            if let Some(f) = from {
                monitor_info.push_str(&format!("发件人过滤: {}\n", f));
            }

            monitor_info.push_str(&format!("超时时间: {}秒", timeout));

            // 修改监听配置显示
            ui::print_card("监听配置", &monitor_info);

            // 创建加载动画
            let spinner = ui::create_spinner("等待验证码邮件...");

            // 开始监听
            match monitor.wait_for_code() {
                Ok(result) => {
                    ui::spinner_success(&spinner, "成功获取验证码");

                    // 复制到剪贴板
                    if let Err(e) = util::clipboard::copy_to_clipboard(&result.code) {
                        ui::print_error(&format!("无法复制到剪贴板: {}", e));
                    } else {
                        ui::print_success("验证码已复制到剪贴板");
                    }

                    // 修改结果显示
                    let result_info = format!(
                        "邮件标题: {}\n发件人: {}\n验证码: {}",
                        result.subject, result.from, result.code
                    );
                    ui::print_result_box("验证码结果", &result.code);
                    ui::print_card("邮件信息", &result_info);
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("获取验证码失败: {}", e));
                    return Err(e);
                }
            }
        }
        Commands::Init => {
            ui::print_module_header("初始化配置文件");

            // 初始化配置文件
            let spinner = ui::create_spinner("正在初始化配置文件...");
            match config::Config::init() {
                Ok(path) => {
                    ui::spinner_success(&spinner, "配置文件初始化成功");
                    // 显示结果
                    ui::print_result_box("配置文件位置", &path.to_string_lossy());
                    ui::print_info("请编辑此配置文件，填入您的Cloudflare和邮箱信息");
                }
                Err(e) => {
                    ui::spinner_error(&spinner, &format!("配置文件初始化失败: {}", e));
                    return Err(e);
                }
            };
        }
    }

    Ok(())
}
