pub trait Future {
    // 1. 这是一个关联类型，代表异步任务最终完成时吐出的数据类型
    type Output;
    // 2. 这是核心方法：轮询/推进任务
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
/////////操作系统