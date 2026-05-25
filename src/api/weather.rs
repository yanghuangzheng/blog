use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;

use crate::global::global::ALL_CONFIG;
use crate::other::structs::WeatherRoot;

//请求头容器
pub async fn get_city_weather() -> Result<WeatherRoot, Box<dyn Error>> {
    let config = ALL_CONFIG
        .get()
        .expect("❌ 全局配置尚未初始化，请先调用 load_config()");
    let api_key = &config.weather.api_key;
    let host_api = &config.weather.host_api;

    let mut headers = HeaderMap::new();
    headers.insert("X-Qw-Api-Key", HeaderValue::from_str(api_key)?);
    // 2. 将请求头塞入客户端，发送请求
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .gzip(true)
        .build()?;
    let url = format!("https://{}/v7/weather/now?location=101010100", host_api);
    //let url = "https://://qweather.com/v7/weather/now?location=101010100";
    let response = client.get(url).send().await?;
    let weather_root: WeatherRoot = response.json::<WeatherRoot>().await?;
    println!("============ 完整的 API 数据结构 ============");
    println!("{:#?}", weather_root);
    println!("=============================================");
    Ok(weather_root)
}
