use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode}; // 补上了 StatusCode 导入
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

// 修正你之前 insert_path 里的闭包转换细节，对齐 ApiUser 类型
pub type ApiUser = Pin<Box<dyn Future<Output = Response<Full<Bytes>>> + Send>>;
pub type HandlerFn =
    Box<dyn Fn(Request<hyper::body::Incoming>, PathParams) -> ApiUser + Send + Sync>;
pub type PathParams = HashMap<String, String>;

pub struct HashRadixTree {
    exactlychildern: HashMap<String, HashRadixNode>,
    backwardchildern: HashMap<String, HashRadixNode>,
}

pub struct HashRadixNode {
    childern: HashMap<String, HashRadixNode>,
    param_name: Option<String>,
    function: Option<HandlerFn>,
}
impl HashRadixNode {
    pub fn new() -> Self {
        Self {
            childern: HashMap::new(),
            param_name: None,
            function: None,
        }
    }
    pub fn node_match<'a>(
        &'a self,
        segments: &'a [&str],
        //captured_params: HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Option<(&'a HandlerFn, PathParams)>> + Send + 'a>> {
        // 2. 将整个逻辑用 Box::pin(async move { ... }) 包裹起来
        Box::pin(async move {
            // 来到了用户输入的终点层
            if segments.is_empty() {
                if let (Some(func), Some(path_rule)) = (&self.function, &self.param_name) {
                    let mut final_params = HashMap::new();
                    let rule_segs: Vec<&str> =
                        path_rule.split('/').filter(|s| !s.is_empty()).collect();

                    let mut param_counter = rule_segs.len();
                    for r_seg in rule_segs {
                        if r_seg.starts_with(':') {
                            let param_name = r_seg[1..].to_string();
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

            let current_seg = segments[0]; // 拿当前这一层的路径片段
            let next_node = self
                .childern
                .get(current_seg)
                .or_else(|| self.childern.get(":"));
            {
                if let Some(res) = next_node?
                    .node_match(&segments[1..], captured_params.clone())
                    .await
                {
                    return Some(res);
                }
            }
            None
        })
    }
    // 路由插入
    pub async fn insert_path<F, Fut>(&mut self, segments: &[&str], path: &str, handler: F)
    where
        F: Fn(Request<hyper::body::Incoming>, PathParams) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Response<Full<Bytes>>> + Send + 'static,
    {
        if segments.is_empty() {
            self.param_name = Some(path.to_string());
            let wrapped_handler: Option<HandlerFn> = Some(Box::new(move |req, params| {
                Box::pin(handler(req, params)) // 删除了无法直接强转的 as ApiUser，依靠隐式隐式特征对齐
            }));
            self.function = wrapped_handler;
            return;
        }
        let current_seg = segments[0];
        let key = if current_seg.starts_with(':') {
            ":"
        } else {
            current_seg
        };

        let next_node = self
            .childern
            .entry(key.to_string())
            .or_insert_with(HashRadixNode::new);

        next_node
            .insert_path(&segments[1..], path, handler.clone())
            .await;
    }
}

impl HashRadixTree {
    pub fn new() -> Self {
        Self {
            exactlychildern: HashMap::new(),
            backwardchildern: HashMap::new(),
        }
    }

    pub async fn route<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<hyper::body::Incoming>, PathParams) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Response<Full<Bytes>>> + Send + 'static,
    {
        let path_str: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect(); // 极其高效：这里只收集了指针地址和长度，没有发生任何字符串深拷贝
                                                                                       /*if path_str.len()==0{  //错误机制
                                                                                           return
                                                                                       }*/
        let mut backward_segs = path_str.clone();
        backward_segs.reverse();
        ////////////////////////////////////////////////////////////正向树
        let current_seg = path_str[0];
        let key = if current_seg.starts_with(':') {
            ":"
        } else {
            current_seg
        };

        let enext_node = self
            .exactlychildern
            .entry(key.to_string())
            .or_insert_with(HashRadixNode::new);
        enext_node
            .insert_path(&path_str[1..], path, handler.clone())
            .await;
        let current_seg = backward_segs[0];
        ////////////////////////////////////////////////////////////反向树
        let key = if current_seg.starts_with(':') {
            ":"
        } else {
            current_seg
        };

        let bnext_node = self
            .backwardchildern
            .entry(key.to_string())
            .or_insert_with(HashRadixNode::new);

        bnext_node
            .insert_path(&backward_segs[1..], path, handler.clone())
            .await;
    }
    ////路由匹配
    pub async fn match_path(
        &self,
        segments: &[&str],
        //captured_params: HashMap<String, String>,
        req: Request<hyper::body::Incoming>,
    ) -> Response<Full<Bytes>> {
        let current_seg = segments[0];
        //先走正向树
        if let Some(next_node) = self.exactlychildern.get(current_seg) {
            if let Some((func, path_rule)) = next_node
                .node_match(&segments[1..]) //, captured_params.clone()
                .await
            {
                return (func)(req, path_rule).await;
            }
        }
        //将路径反转再走反向树
        let mut backward_segments: Vec<&str> = segments.to_vec();
        backward_segments.reverse();
        let current_seg = backward_segments[0];
        if let Some(next_node) = self.backwardchildern.get(current_seg) {
            if let Some((func, path_rule)) = next_node
                .node_match(&backward_segments[1..]) //, captured_params.clone()
                .await
            {
                return (func)(req, path_rule).await;
            }
        }
        let mut aaa = Response::new(Full::new(Bytes::from("404 页面未找到 (MyAxum)")));
        *aaa.status_mut() = StatusCode::NOT_FOUND;
        return aaa;
    }
}
