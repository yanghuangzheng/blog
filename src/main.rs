use blog::api::weather;
use blog::init::init;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init::load_config().await?;
    // 2. 开启新线程并行查天气
    tokio::spawn(async {
        // 在内部用 match 接住错误，不要用问号 `?` 向上抛
        match weather::get_city_weather().await {
            Ok(weather_data) => {
                println!("☀️ 成功获取天气: {:?}", weather_data.now.text);
            }
            Err(e) => {
                eprintln!("❌ 异步线程获取天气失败: {}", e);
            }
        }
    });
    loop {}
}
