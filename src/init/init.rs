use std::error::Error;
use std::fs;

use crate::global::global::ALL_CONFIG;
use crate::other::structs;

pub async fn load_config() -> Result<(), Box<dyn Error>> {
    // 读取文件内容为字符串
    let config_str = fs::read_to_string("config.toml")?;
    // 将 TOML 字符串解析为 Rust 结构体
    let config: structs::AllConfig = toml::from_str(&config_str)?;
    // 3. 核心修改：使用 .set() 或 .get_or_init() 初始化全局静态变量
    // 如果之前已经成功初始化过，它会直接返回现有的，防止重复加载
    let _global_config = ALL_CONFIG.get_or_init(|| async { config }).await;

    Ok(())
}
