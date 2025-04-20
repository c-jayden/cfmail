mod api;
mod config;
mod service;
mod ui;
mod util;

use crate::util::i18n;
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use prettytable;
use service::mail_monitor::{CodeType, MailMonitor, MonitorOptions};
use std::io::{self, BufRead, Write};

/// 获取根据当前语言设置的命令描述
fn t(key: &str) -> String {
    i18n::translate(key)
}

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
#[command(author, version, about = "Cloudflare Email Alias Generator", long_about = None)]
struct Cli {
    /// 设置语言 (en, zh)
    #[arg(long, global = true)]
    locale: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new email alias
    Generate {
        /// Custom prefix (optional)
        #[arg(short, long)]
        prefix: Option<String>,
    },
    /// List all configured email aliases
    List,
    /// Delete email aliases
    Delete,
    /// Monitor for verification code emails
    WatchCode {
        /// Verification code length
        #[arg(short, long)]
        length: Option<usize>,

        /// Verification code type
        #[arg(short, long, value_enum, default_value_t = CodeTypeArg::Numeric)]
        code_type: CodeTypeArg,

        /// Sender filter
        #[arg(short, long)]
        from: Option<String>,

        /// Timeout in seconds
        #[arg(short, long, default_value_t = 300)]
        timeout: u64,

        /// Polling interval in seconds
        #[arg(long, default_value_t = 3)]
        poll_interval: u64,
    },
    /// Initialize configuration file
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化语言
    i18n::init_locale();

    // 解析命令行参数
    let cli = Cli::parse();

    // 如果指定了语言，设置语言
    if let Some(locale_str) = &cli.locale {
        if let Some(locale) = i18n::SupportedLocale::from_str(locale_str) {
            i18n::set_locale(locale);
        } else {
            eprintln!("Unsupported locale: {}. Using default.", locale_str);
        }
    }

    // 显示应用程序标题
    ui::print_app_header();

    // 显示当前语言（仅在调试模式下）
    if std::env::var("CFMAIL_DEBUG").is_ok() {
        println!("Current language: {}\n", i18n::get_current_locale_name());
    }

    match &cli.command {
        Commands::Generate { prefix } => {
            ui::print_module_header(&i18n::translate("modules.generate"));

            // 加载配置
            let spinner = ui::create_spinner(&i18n::translate("ui.loading_config"));
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.config_loaded"));
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.config_failed", &[("error", &e.to_string())]),
                    );
                    ui::print_info(&i18n::translate("ui.config_hint"));
                    return Err(e);
                }
            };

            // 生成邮箱别名
            let spinner = ui::create_spinner(&i18n::translate("ui.generating_alias"));
            let email_alias = match service::email::generate_alias(&cfg, prefix.clone()) {
                Ok(alias) => {
                    ui::spinner_success(
                        &spinner,
                        &i18n::translate_args("ui.alias_generated", &[("alias", &alias)]),
                    );
                    alias
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.alias_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            };

            // 创建Cloudflare邮件路由
            let spinner = ui::create_spinner(&i18n::translate("ui.configuring_route"));
            let cf_client = api::cloudflare::CloudflareClient::new(&cfg);
            match cf_client.create_email_route(&email_alias).await {
                Ok(_) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.route_configured"));
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.route_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            }

            // 复制到剪贴板
            if let Err(e) = util::clipboard::copy_to_clipboard(&email_alias) {
                ui::print_error(&i18n::translate_args(
                    "ui.clipboard_failed",
                    &[("error", &e.to_string())],
                ));
            } else {
                ui::print_success(&i18n::translate("ui.clipboard_success"));
            }

            // 显示最终结果
            ui::print_result_box(&i18n::translate("ui.result_title"), &email_alias);
            ui::print_info(&i18n::translate("ui.alias_ready"));
        }
        Commands::List => {
            ui::print_module_header(&i18n::translate("modules.list"));

            // 加载配置
            let spinner = ui::create_spinner(&i18n::translate("ui.loading_config"));
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.config_loaded"));
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.config_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            };

            // 获取邮箱别名列表
            let spinner = ui::create_spinner(&i18n::translate("ui.fetching_aliases"));
            let cf_client = api::cloudflare::CloudflareClient::new(&cfg);
            match cf_client.list_email_routes().await {
                Ok(aliases) => {
                    ui::spinner_success(
                        &spinner,
                        &i18n::translate_args(
                            "ui.aliases_fetched",
                            &[("count", &aliases.len().to_string())],
                        ),
                    );

                    // 添加换行，确保表格框不会和上面的消息在同一行
                    println!();

                    if aliases.is_empty() {
                        ui::print_warning(&i18n::translate("ui.no_aliases"));
                    } else {
                        ui::print_aliases_table(&aliases);
                    }
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.aliases_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            }
        }
        Commands::Delete => {
            ui::print_module_header(&i18n::translate("modules.delete"));

            // 加载配置
            let spinner = ui::create_spinner(&i18n::translate("ui.loading_config"));
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.config_loaded"));
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.config_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            };

            // 获取邮箱别名列表
            let spinner = ui::create_spinner(&i18n::translate("ui.fetching_aliases"));
            let cf_client = api::cloudflare::CloudflareClient::new(&cfg);
            let aliases = match cf_client.list_email_routes().await {
                Ok(aliases) => {
                    ui::spinner_success(
                        &spinner,
                        &i18n::translate_args(
                            "ui.aliases_fetched",
                            &[("count", &aliases.len().to_string())],
                        ),
                    );
                    aliases
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.aliases_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            };

            if aliases.is_empty() {
                ui::print_warning(&i18n::translate("commands.delete.no_aliases"));
                return Ok(());
            }

            // 显示所有别名列表
            ui::print_aliases_table(&aliases);

            println!();
            ui::print_info(&i18n::translate("commands.delete.enter_numbers"));

            // 读取用户输入
            print!("{}", i18n::translate("commands.delete.prompt"));
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
                ui::print_warning(&i18n::translate_args(
                    "commands.delete.selected_all",
                    &[("count", &aliases.len().to_string())],
                ));
            } else {
                // 解析用户输入的编号
                for num_str in input.split_whitespace() {
                    if let Ok(mut num) = num_str.parse::<usize>() {
                        // 用户输入从1开始，但索引从0开始
                        num = num.saturating_sub(1);
                        if num < aliases.len() {
                            selected_indices.push(num);
                        } else {
                            ui::print_warning(&i18n::translate_args(
                                "commands.delete.ignore_invalid_number",
                                &[("number", num_str)],
                            ));
                        }
                    } else {
                        ui::print_warning(&i18n::translate_args(
                            "commands.delete.ignore_invalid_input",
                            &[("input", num_str)],
                        ));
                    }
                }

                // 去重
                selected_indices.sort();
                selected_indices.dedup();

                if selected_indices.is_empty() {
                    ui::print_info(&i18n::translate("commands.delete.no_selection"));
                    return Ok(());
                }

                ui::print_warning(&i18n::translate_args(
                    "commands.delete.selected_count",
                    &[("count", &selected_indices.len().to_string())],
                ));
            }

            // 显示选中的别名
            println!();
            ui::print_info(&i18n::translate("commands.delete.to_be_deleted"));
            for &idx in &selected_indices {
                println!("  {} {}", "•".yellow(), aliases[idx].cyan());
            }

            // 确认删除
            println!();
            ui::print_warning(&i18n::translate("commands.delete.confirm"));
            print!("{}", i18n::translate("commands.delete.prompt"));
            io::stdout().flush()?;

            let mut confirm = String::new();
            stdin.lock().read_line(&mut confirm)?;

            if !confirm.trim().eq_ignore_ascii_case("y") {
                ui::print_info(&i18n::translate("commands.delete.cancelled"));
                return Ok(());
            }

            // 开始删除操作
            let spinner = ui::create_spinner(&i18n::translate("commands.delete.deleting"));
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
                    &i18n::translate_args(
                        "commands.delete.delete_success",
                        &[
                            ("success", &success_count.to_string()),
                            ("total", &selected_indices.len().to_string()),
                        ],
                    ),
                );

                // 添加删除总结
                if success_count > 0 {
                    println!();
                    ui::print_card(
                        &i18n::translate("commands.delete.summary_title"),
                        &i18n::translate_args(
                            "commands.delete.summary_content",
                            &[("count", &success_count.to_string())],
                        ),
                    );
                    ui::print_success(&i18n::translate("commands.delete.all_deleted"));
                }
            } else {
                ui::spinner_error(
                    &spinner,
                    &i18n::translate_args(
                        "commands.delete.delete_partial",
                        &[
                            ("success", &success_count.to_string()),
                            ("total", &selected_indices.len().to_string()),
                        ],
                    ),
                );

                // 显示失败的别名
                println!();
                ui::print_card(
                    &i18n::translate("commands.delete.failure_title"),
                    &i18n::translate_args(
                        "commands.delete.failure_content",
                        &[("count", &failed_aliases.len().to_string())],
                    ),
                );

                // 创建表格
                let mut table = prettytable::Table::new();
                table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);

                // 添加标题行
                table.add_row(prettytable::row![
                    b->i18n::translate("commands.delete.failure_table_headers.email"),
                    b->i18n::translate("commands.delete.failure_table_headers.reason")
                ]);

                // 添加数据行
                for (alias, error) in failed_aliases {
                    table.add_row(prettytable::row![
                        FY->alias,
                        FR->error
                    ]);
                }

                // 显示表格
                table.printstd();
            }
        }
        Commands::WatchCode {
            length,
            code_type,
            from,
            timeout,
            poll_interval,
        } => {
            ui::print_module_header(&i18n::translate("modules.watch"));

            // 加载配置
            let spinner = ui::create_spinner(&i18n::translate("ui.loading_config"));
            let cfg = match config::Config::load() {
                Ok(cfg) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.config_loaded"));
                    cfg
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.config_failed", &[("error", &e.to_string())]),
                    );
                    ui::print_info(&i18n::translate("ui.config_hint"));
                    return Err(e);
                }
            };

            // 创建邮件监听器选项
            let options = MonitorOptions {
                code_length: *length,
                code_type: code_type.clone().into(),
                from_filter: from.clone(),
                timeout: *timeout,
                poll_interval: *poll_interval,
            };

            // 创建邮件监听器
            let monitor = MailMonitor::new(&cfg, options);

            // 等待验证码
            let spinner = ui::create_spinner(&i18n::translate("ui.waiting_code"));
            match monitor.wait_for_code() {
                Ok(result) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.code_found"));

                    // 复制到剪贴板
                    if let Err(e) = util::clipboard::copy_to_clipboard(&result.code) {
                        ui::print_error(&i18n::translate_args(
                            "ui.clipboard_failed",
                            &[("error", &e.to_string())],
                        ));
                    } else {
                        ui::print_success(&i18n::translate("ui.code_copied"));
                    }

                    // 显示最终结果
                    ui::print_result_box(&i18n::translate("ui.code_result"), &result.code);
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args("ui.code_failed", &[("error", &e.to_string())]),
                    );
                    return Err(e);
                }
            }
        }
        Commands::Init => {
            ui::print_module_header(&i18n::translate("modules.init"));

            // 初始化配置文件
            let spinner = ui::create_spinner(&i18n::translate("ui.initializing_config"));
            match config::Config::init() {
                Ok(path) => {
                    ui::spinner_success(&spinner, &i18n::translate("ui.config_initialized"));
                    // 显示结果
                    ui::print_result_box(
                        &i18n::translate("ui.config_location"),
                        &path.to_string_lossy(),
                    );
                    ui::print_info(&i18n::translate("ui.config_edit_hint"));
                }
                Err(e) => {
                    ui::spinner_error(
                        &spinner,
                        &i18n::translate_args(
                            "ui.config_init_failed",
                            &[("error", &e.to_string())],
                        ),
                    );
                    return Err(e);
                }
            };
        }
    }

    Ok(())
}
