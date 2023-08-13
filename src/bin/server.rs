#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use rust_demo_2::{S};

#[volo::main]
async fn main() {
    // 只有注册 subscriber 后， 才能在控制台上看到日志输出
    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::rust_demo_2::ItemBackendServiceServer::new(S)
        .layer_front(ErrorLayer)
        .run(addr)
        .await
        .unwrap();
}

use futures::Future;
use rust_demo_2::{ServiceError, SetBaseResp};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    string::String,
};
use metainfo::Forward;
use volo_gen::rust_demo_2::{GetItemBackendResponse, ItemBackendServiceGetItemBackendResultSend, ItemBackendServiceResponseSend};

// 定义Response类型映射
lazy_static::lazy_static! {
    pub static ref BizResponseMap: Arc<RwLock<HashMap<String, ItemBackendServiceResponseSend>>> = make_biz_response();
}
fn make_biz_response() -> Arc<RwLock<HashMap<String, ItemBackendServiceResponseSend>>> {
    let mut res = HashMap::new();
    res.insert(String::from("GetItemBackend"), ItemBackendServiceResponseSend::GetItemBackend(ItemBackendServiceGetItemBackendResultSend::Ok(GetItemBackendResponse{..Default::default()})));

    return Arc::new( RwLock::new( res))
}

// 定义错误处理中间件
pub struct ErrorLayer;
impl<S> volo::Layer<S> for ErrorLayer {
    type Service = ErrorService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ErrorService(inner)
    }
}
#[derive(Clone)]
pub struct ErrorService<S>(S);
impl<Cx, Req, S> volo::Service<Cx, Req> for ErrorService<S>
    where
        Req: std::fmt::Debug + Send + 'static, // 特征约束
        S: Send + Sync + 'static + volo::Service<Cx, Req, Error = volo_thrift::AnyhowError, Response = volo_gen::rust_demo_2::ItemBackendServiceResponseSend>,
        S::Response: std::fmt::Debug,
        S::Error: std::fmt::Debug,
        Cx: Send + 'static + volo::context::Context,
{

    type Response = S::Response; // 关联类型
    type Error = S::Error;
    type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'cx where S: 'cx;

    #[inline]
    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Req) -> Self::Future<'cx>
        where
            's: 'cx,
    {
        async move {
            metainfo::METAINFO
                .with(|metainfo| {
                    let method = cx.rpc_info().method.clone().unwrap_or_default();
                    tracing::info!("Rust Demo method {:?}", method);
                    let mut info = metainfo.borrow_mut();
                    info.set_persistent("biz_response", method);
                });

            let now = std::time::Instant::now();
            // {:?}打印的日志不好用, 能不能打印Json?
            tracing::info!("Rust Demo {:?}", &req);
            let resp = self.0.call(cx, req).await;
            tracing::info!("Rust Demo response {:?}", &resp);
            tracing::info!("Rust Demo took {}ms", now.elapsed().as_millis());

            // 处理返回的结果
            return match resp {
                // 没错误不用管
                Ok(r) => {
                    Ok(r)
                }
                // 有错误, 处理错误
                Err(e) => {
                    tracing::info!("Rust Demo any error");
                    // 判断是不是ServiceError
                    if let Some(service_error) = e.downcast_ref::<ServiceError>() {
                        tracing::info!("Rust Demo service error");
                        match service_error {
                            // 业务错误, 可以转换为BaseResp
                            ServiceError::BizError(biz_error) => {
                                tracing::info!("Rust Demo biz_error {:?} {:?}", biz_error.code, biz_error.message);
                                // 查询业务Response类型
                                let biz_resp_key_option = metainfo::METAINFO
                                    .with(|metainfo| {
                                        let mut info = metainfo.borrow_mut();
                                        let biz_resp_key_option = info.get_persistent("biz_response");
                                        return biz_resp_key_option
                                    });

                                tracing::info!("Rust Demo resp type key {:?}", biz_resp_key_option);

                                match biz_resp_key_option {
                                    // 查找 Response Type Key
                                    Some(biz_resp_key) => {
                                        tracing::info!("Rust Demo find key");
                                        let biz_response_map_guard = BizResponseMap.read().unwrap();
                                        let biz_resp_option = biz_response_map_guard.get(biz_resp_key.as_str());
                                        // 查找 Response Type
                                        match biz_resp_option {
                                            Some(biz_resp) => {
                                                tracing::info!("Rust Demo begin transform!");
                                                let b = &mut biz_resp.clone();
                                                b.set_base_resp(biz_error.clone().code, biz_error.clone().message);
                                                return Ok(b.to_owned())
                                            }
                                            None => { tracing::info!("Rust Demo not found resp type"); }
                                        }
                                    }
                                    None => { tracing::info!("Rust Demo not found resp type key"); }
                                }
                            },
                            _ => tracing::info!("Rust Demo unknown error"),
                        }
                    }
                    // 无法处理的Error, 原样返回
                    return Err(e);
                }
            }
        }
    }
}
