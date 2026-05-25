use serde::Deserialize;

//1.天气最外层根结构
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] // 告诉 Rust 编译器允许字段使用驼峰命名法（如 feelsLike），不要报警告
pub struct WeatherRoot {
    pub code: String,       // 状态码：成功时为 "200"
    pub updateTime: String, // API 数据更新时间
    pub fxLink: String,     // 响应的天气预报网页链接
    pub now: WeatherNow,    // 核心：当前实况天气数据
    pub refer: ReferInfo,   // 数据来源引用信息
}
// 2. 对应 "now" 键值对中的实况天气详情
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)] // 告诉 Rust 编译器允许字段使用驼峰命名法（如 feelsLike），不要报警告
pub struct WeatherNow {
    pub obsTime: String,   // 天气观测时间
    pub temp: String,      // 实时温度（注意：和风天气返回的全是 String 字符串格式）
    pub feelsLike: String, // 体感温度
    pub icon: String,      // 天气图标代码
    pub text: String,      // 天气状况描述（例如："多云"）
    pub wind360: String,   // 风向角度
    pub windDir: String,   // 风向（例如："东南风"）
    pub windScale: String, // 风力等级
    pub windSpeed: String, // 风速，公里/小时
    pub humidity: String,  // 相对湿度百分比
    pub precip: String,    // 过去1小时降水量
    pub pressure: String,  // 大气压强
    pub vis: String,       // 能见度，公里
}
// 3. 对应 "refer" 键值对中的来源数据
#[derive(Deserialize, Debug)]
pub struct ReferInfo {
    pub sources: Vec<String>, // 数据源列表（如 ["QWeather", "NMC"]）
    pub license: Vec<String>, // 许可协议
}
//天气api初始化数据
#[derive(Deserialize, Debug)]
pub struct WeatherConfig {
    pub host_api: String,
    pub api_key: String,
    pub api_id: String,
}
//all结构体
//天气api初始化数据
#[derive(Deserialize, Debug)]
pub struct AllConfig {
    pub weather: WeatherConfig,
}
