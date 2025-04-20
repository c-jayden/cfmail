use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::RwLock;

/// 支持的语言列表
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLocale {
    EnUS,
    ZhCN,
}

impl SupportedLocale {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedLocale::EnUS => "en-US",
            SupportedLocale::ZhCN => "zh-CN",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "en-us" | "en_us" | "english" => Some(SupportedLocale::EnUS),
            "zh" | "zh-cn" | "zh_cn" | "chinese" => Some(SupportedLocale::ZhCN),
            _ => None,
        }
    }

    pub fn from_system() -> Self {
        // 尝试从系统获取语言设置
        if let Ok(lang) = std::env::var("LANG") {
            let lang = lang.split('.').next().unwrap_or("");
            if let Some(locale) = Self::from_str(lang) {
                return locale;
            }
        }

        // 尝试从LC_ALL获取语言设置
        if let Ok(lang) = std::env::var("LC_ALL") {
            let lang = lang.split('.').next().unwrap_or("");
            if let Some(locale) = Self::from_str(lang) {
                return locale;
            }
        }

        // 默认英语
        SupportedLocale::EnUS
    }
}

/// 当前选择的语言
static CURRENT_LOCALE: Lazy<RwLock<SupportedLocale>> =
    Lazy::new(|| RwLock::new(SupportedLocale::EnUS));

/// 存储所有语言的翻译
static TRANSLATIONS: Lazy<RwLock<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 加载指定语言的翻译文件
fn load_translations(locale: SupportedLocale) -> Result<(), String> {
    let locale_str = locale.as_str();
    let file_path = format!("locales/{}.json", locale_str);

    // 检查是否已加载
    {
        let translations = TRANSLATIONS.read().unwrap();
        if translations.contains_key(locale_str) {
            return Ok(());
        }
    }

    // 尝试加载文件
    let path = Path::new(&file_path);
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open file {}: {}", file_path, e)),
    };

    // 读取文件内容
    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content) {
        return Err(format!("Failed to read file {}: {}", file_path, e));
    }

    // 解析JSON
    let json: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to parse JSON from {}: {}", file_path, e)),
    };

    // 转换为扁平结构
    let mut flat_map = HashMap::new();
    flatten_json(&json, &mut flat_map, "");

    // 存储翻译
    let mut translations = TRANSLATIONS.write().unwrap();
    translations.insert(locale_str.to_string(), flat_map);

    Ok(())
}

/// 将嵌套的JSON转换为扁平结构
fn flatten_json(json: &Value, result: &mut HashMap<String, String>, prefix: &str) {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(value, result, &new_prefix);
            }
        }
        Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        _ => {
            // 其他类型不处理
        }
    }
}

/// 设置当前语言
pub fn set_locale(locale: SupportedLocale) {
    // 加载语言文件
    if let Err(e) = load_translations(locale) {
        eprintln!(
            "Warning: Failed to load translations for {}: {}",
            locale.as_str(),
            e
        );
    }

    let mut current = CURRENT_LOCALE.write().unwrap();
    *current = locale;
}

/// 根据系统设置选择语言
pub fn init_locale() {
    let locale = SupportedLocale::from_system();
    set_locale(locale);

    // 同时加载英文作为后备
    if locale != SupportedLocale::EnUS {
        if let Err(e) = load_translations(SupportedLocale::EnUS) {
            eprintln!("Warning: Failed to load English translations: {}", e);
        }
    }
}

/// 获取当前语言的翻译表
fn get_translation(key: &str) -> Option<String> {
    let translations = TRANSLATIONS.read().unwrap();
    let current = *CURRENT_LOCALE.read().unwrap();

    // 尝试从当前语言获取
    if let Some(map) = translations.get(current.as_str()) {
        if let Some(value) = map.get(key) {
            return Some(value.clone());
        }
    }

    // 如果当前语言没有这个键，尝试从英文获取
    if current != SupportedLocale::EnUS {
        if let Some(map) = translations.get(SupportedLocale::EnUS.as_str()) {
            if let Some(value) = map.get(key) {
                return Some(value.clone());
            }
        }
    }

    None
}

/// 获取翻译文本
pub fn translate(key: &str) -> String {
    get_translation(key).unwrap_or_else(|| key.to_string())
}

/// 带参数的翻译
pub fn translate_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut msg = translate(key);
    for (name, value) in args {
        let placeholder = format!("%{{{}}}", name);
        msg = msg.replace(&placeholder, value);
    }
    msg
}

/// 获取当前语言
pub fn get_current_locale() -> SupportedLocale {
    *CURRENT_LOCALE.read().unwrap()
}

/// 获取当前语言的显示名称
pub fn get_current_locale_name() -> &'static str {
    match get_current_locale() {
        SupportedLocale::EnUS => "English",
        SupportedLocale::ZhCN => "简体中文",
    }
}

/// 列出所有支持的语言
pub fn list_supported_locales() -> Vec<(&'static str, &'static str)> {
    vec![("en", "English"), ("zh", "简体中文")]
}
