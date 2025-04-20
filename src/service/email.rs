use crate::config::Config;
use anyhow::{Result, anyhow};
use rand::{Rng, distributions::Alphanumeric, distributions::Distribution};
use std::iter;

pub fn generate_alias(config: &Config, custom_prefix: Option<String>) -> Result<String> {
    let prefix = match custom_prefix {
        Some(prefix) => prefix,
        None => match config.alias.prefix_mode.as_str() {
            "random" => generate_random_prefix(config),
            "custom" => {
                if config.alias.custom_prefixes.is_empty() {
                    return Err(anyhow!("配置错误: 选择了自定义前缀模式，但未提供任何前缀"));
                }
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..config.alias.custom_prefixes.len());
                config.alias.custom_prefixes[idx].clone()
            }
            _ => {
                return Err(anyhow!(
                    "配置错误: 不支持的前缀模式: {}",
                    config.alias.prefix_mode
                ));
            }
        },
    };

    Ok(format!("{}@{}", prefix, config.email.domain))
}

fn generate_random_prefix(config: &Config) -> String {
    let mut rng = rand::thread_rng();

    match config.alias.random_charset.as_str() {
        "alphanumeric" => iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(config.alias.random_length)
            .collect(),
        "alphabetic" => {
            let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
            let dist = rand::distributions::Uniform::from(0..chars.len());

            (0..config.alias.random_length)
                .map(|_| chars[dist.sample(&mut rng)])
                .collect()
        }
        "numeric" => {
            let chars: Vec<char> = "0123456789".chars().collect();
            let dist = rand::distributions::Uniform::from(0..chars.len());

            (0..config.alias.random_length)
                .map(|_| chars[dist.sample(&mut rng)])
                .collect()
        }
        _ => {
            // 默认使用字母数字组合
            iter::repeat(())
                .map(|()| rng.sample(Alphanumeric))
                .map(char::from)
                .take(config.alias.random_length)
                .collect()
        }
    }
}
