use hyper_util::rt::TokioIo;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use std::net::SocketAddr;
use tokio::net::TcpListener;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 像 Axum 一样丝滑地注册路由！
    let app_router = MyRouter::new()
        .route("GET", "/", index_handler)
        .route("GET", "/weather", weather_handler);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    println!("🚀 自定义框架已托管博客项目，正在监听: http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let router_clone = app_router.clone(); // 复制路由树引用给多线程使用

        // 2. 开启多线程异步并发，每一个进来访问的读者都由一个独立线程进行路由分发
        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let router = router_clone.clone();
                async move { Ok::<_, std::convert::Infallible>(router.dispatch(req).await) }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("❌ 框架网络层异常: {:?}", err);
            }
        });
    }
}








    pub async fn node_match(
        &self,
        segments: &[&str],
        captured_params: HashMap<String, String>,
    ) -> Option<(&HandlerFn, PathParams)> {
        //-> Option<(&HandlerFn, PathParams)>
        //来到了用户输入的终点层
        if segments.is_empty() && self.param_name != None {
            if let (Some(func), Some(path_rule)) = (&self.function, &self.param_name) {
                let mut final_params = HashMap::new();
                let rule_segs: Vec<&str> = path_rule.split('/').filter(|s| !s.is_empty()).collect();

                let mut param_counter = rule_segs.len();
                for r_seg in rule_segs {
                    if r_seg.starts_with(':') {
                        let param_name = r_seg[1..].to_string(); // 拿到纯名字 "id"
                                                                 // 找到当时对应层级塞进去的临时临时占位符
                        let temp_key = format!("param_{}", param_counter);
                        if let Some(val) = captured_params.get(&temp_key) {
                            final_params.insert(param_name, val.clone());
                        }
                    }
                    param_counter -= 1;
                }
                return Some((func, final_params));
            }
            return None;
        }
        let current_seg = segments[0];
        if let Some(next_node) = self.childern.get(current_seg) {
            if let Some(res) = next_node
                .node_match(&segments[1..], captured_params.clone())
                .await
            {
                return Some(res);
            }
        }
        return None;
    }