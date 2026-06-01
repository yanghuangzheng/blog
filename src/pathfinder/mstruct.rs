use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

type ApiUser = Pin<Box<dyn Future<Output = Response<Full<Bytes>>> + Send>>;
// 修改点：我们的业务处理函数现在不仅接收请求，还能接收解析好的路径参数 (PathParams)
type HandlerFn = Box<dyn Fn(Request<hyper::body::Incoming>, PathParams) -> ApiUser + Send + Sync>;
// 用于存放从 URL 中提取出来的动态参数，例如 {"id": "42"}
pub type PathParams = HashMap<String, String>;
// 每一个注册路由的条目
struct RouteEntry {
    method: String,             //主方法
    path_segments: Vec<String>, // 将 "/posts/:id" 拆成 ["posts", ":id"] 主要方法里面的次要路径方法 冒号后面就是值
    handler: HandlerFn,
}

pub struct MyRouter {
    routes: Vec<RouteEntry>, // 使用数组代替 HashMap 以便进行模糊匹配
}

impl MyRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    //路由注册
    pub fn route<F, Fut>(mut self, method: &str, path: &str, handler: F) -> Self
    where
        //因为刚开始传进来的函数肯定是没有Box包裹的 而且刚开始的输出也是没有box包裹的 f 和 fut 起到的是转换
        F: Fn(Request<hyper::body::Incoming>, PathParams) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<Full<Bytes>>> + Send + 'static,
    {
        //let key = format!("{} {}", method, path);
        // 将路径按 '/' 拆分，过滤掉空字符串。例如 "/posts/:id" -> ["posts", ":id"]
        let path_segments: Vec<String> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        // 将用户的普通 async fn 包装成框架内部统一的 HandlerFn 指针
        //let wrapped_handler = Box::new(move |req, params| Box::pin(handler(req, params) ))as ApiUser;
        let wrapped_handler: HandlerFn =
            Box::new(move |req, params| Box::pin(handler(req, params)) as ApiUser); //这里的as是返回变成ApiUser的意思
        self.routes.push(RouteEntry {
            method: method.to_string(),
            path_segments,
            handler: wrapped_handler,
        });
        self
    }
    //路由匹配
    pub async fn dispatch(&self, req: Request<hyper::body::Incoming>) -> Response<Full<Bytes>> {
        let req_method = req.method().as_str();
        let req_path = req.uri().path();

        // 同样拆分请求的网址，例如 "/posts/42" -> ["posts", "42"]
        let req_segments: Vec<&str> = req_path.split('/').filter(|s| !s.is_empty()).collect();

        // 遍历所有注册过的路由进行匹配
        for route in &self.routes {
            if route.method != req_method || route.path_segments.len() != req_segments.len() {
                continue; // 方法不同或段数不同，直接跳过
            }

            let mut params = HashMap::new();
            let mut is_match = true;

            // 逐段比对
            for (i, registered_seg) in route.path_segments.iter().enumerate() {
                //enumerate() 会同时给出当前片段的索引下标 i 和 注册的模板内容 registered_seg 本质就是一个普通的字符串切片 &str
                let actual_seg = req_segments[i]; //对比
                                                  //检查注册的模板片段是否以冒号 : 开头。在 Web 框架中，冒号通常代表这块是一个动态参数占位符。
                if registered_seg.starts_with(':') {
                    // 发现动态变量！把冒号后面的名字作为 Key，实际的值作为 Value 存入哈希表
                    let param_name = registered_seg[1..].to_string(); //去掉冒号
                    params.insert(param_name, actual_seg.to_string());
                } else if registered_seg != actual_seg {
                    // 普通字符串匹配失败
                    is_match = false;
                    break;
                }
            }

            if is_match {
                // 命中路由就地执行处理器，并将捕获到的路径参数传过去
                return (route.handler)(req, params).await;
            }
        }

        // 所有路由都匹配不上，返回 404
        let mut res = Response::new(Full::new(Bytes::from("404 页面未找到 (MyAxum)")));
        *res.status_mut() = StatusCode::NOT_FOUND;
        res
    }
}

//还需要  统一的响应转化  请求体智能提取  全局状态共享  基数树
/*

*/
