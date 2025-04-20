use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Cell, Row, Table, format};
use std::time::Duration;

/// 主题颜色设置
pub struct Theme {
    pub primary: colored::Color,
    pub secondary: colored::Color,
    pub success: colored::Color,
    pub error: colored::Color,
    pub warning: colored::Color,
    pub info: colored::Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: colored::Color::Magenta, // 主色调
            secondary: colored::Color::Cyan,  // 次要色调
            success: colored::Color::Green,   // 成功
            error: colored::Color::Red,       // 错误
            warning: colored::Color::Yellow,  // 警告
            info: colored::Color::Blue,       // 信息
        }
    }
}

// 全局主题
static THEME: Theme = Theme {
    primary: colored::Color::Magenta,
    secondary: colored::Color::Cyan,
    success: colored::Color::Green,
    error: colored::Color::Red,
    warning: colored::Color::Yellow,
    info: colored::Color::Blue,
};

/// 打印带有彩色的消息
pub fn print_message(prefix: &str, message: &str, color: colored::Color) {
    println!("{} {}", prefix.color(color).bold(), message);
}

/// 打印成功消息
pub fn print_success(message: &str) {
    print_message("✓", message, THEME.success);
}

/// 打印错误消息
pub fn print_error(message: &str) {
    print_message("✗", message, THEME.error);
}

/// 打印信息
pub fn print_info(message: &str) {
    print_message("ℹ", message, THEME.info);
}

/// 打印警告
pub fn print_warning(message: &str) {
    print_message("⚠", message, THEME.warning);
}

/// 打印高亮标题
#[allow(dead_code)]
pub fn print_title(title: &str) {
    let divider = "─".repeat(title.len() + 4);
    println!("\n{}", divider.color(THEME.primary));
    println!("  {}", title.color(THEME.primary).bold());
    println!("{}\n", divider.color(THEME.primary));
}

/// 打印带有标题的信息块（简化版）
pub fn print_card(title: &str, content: &str) {
    // 使用标题
    print_section_title(title);

    // 打印内容（每行缩进两个空格）
    for line in content.lines() {
        println!("  {}", line);
    }

    // 添加一行空行作为分隔
    println!();
}

/// 打印命令帮助
#[allow(dead_code)]
pub fn print_command_help(command: &str, description: &str) {
    println!("  {} {}", command.color(THEME.primary).bold(), description);
}

/// 创建加载中动画
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.magenta} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// 展示完成消息并停止加载动画
pub fn spinner_success(spinner: &ProgressBar, message: &str) {
    spinner.finish_and_clear();
    println!("{} {}", "✓".green().bold(), message);
}

/// 展示错误消息并停止加载动画
pub fn spinner_error(spinner: &ProgressBar, message: &str) {
    spinner.finish_and_clear();
    println!("{} {}", "✗".red().bold(), message);
}

/// 打印应用程序标题
pub fn print_app_header() {
    // 使用 ASCII 艺术字体显示 CFMAIL
    let logo = r#"
   ______  _________  __  ___   _____  ______
  / ____/ / ____/   |/  |/  /  /  _/ |/ / __ \
 / /     / /_  / /| / /|_/ /   / / |   / / / /
/ /___  / __/ / ___ / /  / /  _/ / /   / /_/ /
\____/ /_/   /_/  |_/_/  /_/  /___//_/|_\____/
    "#;

    println!("{}", logo.bright_magenta().bold());
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{}",
        format!("Cloudflare 邮箱别名生成器 v{}", version).bright_cyan()
    );
    println!("{}", "一个强大、简洁的邮箱别名管理工具".cyan());
    println!("{}\n", "=".repeat(60).bright_magenta());
}

/// 打印功能模块标题
pub fn print_module_header(title: &str) {
    let title_display = format!("【 {} 】", title);
    let padding = (60 - title_display.len()) / 2;
    println!(
        "\n{}{}{}",
        "=".repeat(padding).bright_cyan(),
        title_display.bright_white().bold().on_bright_magenta(),
        "=".repeat(padding).bright_cyan()
    );
}

/// 打印带有进度和状态的列表项
#[allow(dead_code)]
pub fn print_list_item(index: usize, text: &str, status: Option<&str>) {
    let index_str = format!("{}.", index + 1);
    print!(
        "{} {}",
        index_str.bright_cyan().bold(),
        text.bright_yellow()
    );

    if let Some(status_text) = status {
        if status_text.contains("成功") || status_text.contains("活跃") {
            println!(" [{}]", status_text.green());
        } else if status_text.contains("失败") || status_text.contains("错误") {
            println!(" [{}]", status_text.red());
        } else {
            println!(" [{}]", status_text.yellow());
        }
    } else {
        println!();
    }
}

/// 打印章节标题（简化版）
fn print_section_title(title: &str) {
    println!("\n{}", title.color(THEME.primary).bold());
    println!("{}", "─".repeat(title.len()).color(THEME.secondary));
}

/// 打印结果框（简化版）
pub fn print_result_box(title: &str, value: &str) {
    print_section_title(title);
    println!("  {}", value.bright_yellow().bold());
    println!();
}

/// 打印表格
#[allow(dead_code)]
pub fn print_table<T: ToString>(headers: &[&str], rows: &[Vec<T>]) {
    // 创建表格
    let mut table = Table::new();

    // 设置表格格式 - 使用带圆角的边框样式
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    // 添加标题行
    let header_row = Row::new(
        headers
            .iter()
            .map(|h| Cell::new(h).style_spec("bFc"))
            .collect(),
    );
    table.add_row(header_row);

    // 添加数据行
    for row in rows {
        let cells: Vec<Cell> = row
            .iter()
            .map(|cell| Cell::new(&cell.to_string()))
            .collect();
        table.add_row(Row::new(cells));
    }

    // 打印表格
    table.printstd();
    println!();
}

/// 打印别名表格
pub fn print_aliases_table(aliases: &[String]) {
    if aliases.is_empty() {
        print_warning("未找到邮箱别名");
        return;
    }

    // 创建表格标题
    print_section_title(&format!("活跃的邮箱别名 (共{}个)", aliases.len()));

    // 创建表格
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    // 添加标题行
    table.set_titles(Row::new(vec![
        Cell::new("序号").style_spec("bFc"),
        Cell::new("邮箱别名").style_spec("bFc"),
        Cell::new("状态").style_spec("bFc"),
    ]));

    // 添加数据行
    for (i, alias) in aliases.iter().enumerate() {
        table.add_row(Row::new(vec![
            Cell::new(&format!("{}.", i + 1)).style_spec("Fc"),
            Cell::new(alias).style_spec("Fy"),
            Cell::new("活跃").style_spec("Fg"),
        ]));
    }

    // 打印表格
    table.printstd();
    println!();
}
