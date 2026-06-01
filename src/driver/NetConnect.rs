use std::future::Future;
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, TcpStream}; // 明确使用标准库的原生流
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::unix::AsyncFd; // Tokio 的底层描述符控制器（负责底层 Waker 登记）

pub struct AsyncConnectFuture {
    // 保持你的设计：用 Option 存放正在后台握手的底层 Socket
    // 想要驱动原生流，这是最底层的安全容器
    async_fd: Option<AsyncFd<TcpStream>>, 
    addr: SocketAddr,
    connect_initiated: bool, 
}

impl AsyncConnectFuture {
    pub fn new(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self {
            async_fd: None,
            addr,
            connect_initiated: false,
        })
    }
}

impl Future for AsyncConnectFuture {
    type Output = io::Result<TcpStream>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        loop {
            // 分支 1：保持你的设计，第一次进来，发起 TCP 三次握手
            if !this.connect_initiated {
                let domain = if this.addr.is_ipv4() {
                    socket2::Domain::IPV4
                } else {
                    socket2::Domain::IPV6
                };
                let socket = match socket2::Socket::new(
                    domain,
                    socket2::Type::STREAM,
                    Some(socket2::Protocol::TCP),
                ) {
                    Ok(s) => s,
                    Err(e) => return Poll::Ready(Err(e)),
                };

                if let Err(e) = socket.set_nonblocking(true) {
                    return Poll::Ready(Err(e));
                }

                match socket.connect(&this.addr.into()) {
                    Ok(_) => println!("连接瞬间打通。"),
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        println!("网络正在后台握手 (EINPROGRESS)。");
                    }
                    Err(e) => return Poll::Ready(Err(e)),
                }

                let std_stream: TcpStream = socket.into();
                
                // 将原生流封装进底层驱动，此时它会被注册到系统的 epoll/kqueue 中
                let async_fd = match AsyncFd::new(std_stream) {
                    Ok(fd) => fd,
                    Err(e) => return Poll::Ready(Err(e)),
                };

                // 【底层修复】利用你的核心思想：在此处进行 Waker 的底层登记
                // 我们必须检查它的返回值，不能用 let _ = 盲目丢弃
                match async_fd.poll_write_ready(cx) {
                    Poll::Ready(_) => {
                        // 如果运气极好，在这里就已经可写了，绝不能 Pending！
                        // 我们需要立刻进入下一轮 loop 去做 peer_addr 检查
                        this.async_fd = Some(async_fd);
                        this.connect_initiated = true;
                        continue; 
                    }
                    Poll::Pending => {
                        // 完美符合你的设计：此时 Waker 已被底层成功登记到 epoll 树上
                        // 我们可以安全、显式地返回 Pending，安心退出！
                        this.async_fd = Some(async_fd);
                        this.connect_initiated = true;
                        return Poll::Pending; 
                    }
                }
            }

            // 分支 2：保持你的设计，第二次被唤醒时进来，检查后台的三次握手到底成功了没有
            println!("检查后台的三次握手到底成功了没有...");

            // 纯粹的只读检查（&），通过 poll_write_ready 探测
            let poll_res = match &this.async_fd {
                Some(fd) => fd.poll_write_ready(cx),
                None => Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "连接已被消费",
                ))),
            };

            match poll_res {
                Poll::Pending => {
                    // 还没握手完，继续等待下一次操作系统唤醒
                    return Poll::Pending;
                }
                Poll::Ready(Ok(mut guard)) => {
                    // 可写事件就绪，代表三次握手结束。接下来进行你的安全检查
                    match &this.async_fd {
                        Some(fd) => {
                            // 通过 get_ref() 获取内部纯原生 stream 引用
                            if let Err(err) = fd.get_ref().peer_addr() {
                                println!(" 三次握手失败了: {:?}", err);
                                return Poll::Ready(Err(err));
                            }
                        }
                        None => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::NotConnected,
                                "致命逻辑漏洞：安检时连接已提前空壳化",
                            )));
                        }
                    }

                    // 检查通过！清除就绪状态，防止残余事件污染
                    guard.clear_ready();

                    // 彻底提取所有权
                    let async_fd = match this.async_fd.take() {
                        Some(fd) => fd,
                        None => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::NotConnected,
                                "致命逻辑漏洞：连接所有权已被提前抽空",
                            )));
                        }
                    };
                    
                    // 剥离掉底层包装，将纯正的原生 std::net::TcpStream 还给上层
                    let connected_stream = async_fd.into_inner();
                    println!("三次握手成功，成功提取原生流！");
                    return Poll::Ready(Ok(connected_stream));
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            }
        }
    }
}