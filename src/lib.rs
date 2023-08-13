#![feature(impl_trait_in_assoc_type)]

pub struct S;

#[volo::async_trait]
impl volo_gen::rust_demo_2::ItemBackendService for S {
	async fn get_item_backend(&self, _req: volo_gen::rust_demo_2::GetItemBackendRequest) -> ::core::result::Result<volo_gen::rust_demo_2::GetItemBackendResponse, ::volo_thrift::AnyhowError>{
		// 2.1
		// Ok(volo_gen::rust_demo_2::GetItemBackendResponse{
		// 	data: "hello volo".into(),
		// 	base_resp: volo_gen::rust_demo_2::BaseResp{
		// 		status_message: Some("success".into()),
		// 		status_code: Some(0),
		// 		extra: None,
		// 	},
		// })

		// 3.1
		// let res = func0();
		// // 分辨业务函数返回的结果
		// return match res {
		// 	// 将业务错误转成BaseResp进行传递
		// 	Err(e) => {
		// 		match e {
		// 			ServiceError::BizError(biz_error) => {
		// 				Ok(volo_gen::rust_demo_2::GetItemBackendResponse {
		// 					base_resp:volo_gen::rust_demo_2::BaseResp {
		// 						status_code: Some(biz_error.code),
		// 						status_message: Some(biz_error.message.into()),
		// 						extra: None,
		// 					},
		// 					..Default::default()
		// 				})
		// 			}
		// 			_ => {
		// 				Ok(volo_gen::rust_demo_2::GetItemBackendResponse {
		// 					base_resp: volo_gen::rust_demo_2::BaseResp {
		// 						status_code: Some(-1),
		// 						status_message: Some("system error".into()),
		// 						extra: None,
		// 					},
		// 					..Default::default()
		// 				})
		// 			}
		// 		}
		// 	}
		// 	Ok(biz) => {
		// 		Ok(Default::default())
		// 	}
		// }

		// 3.2
		// let res = func0();
		// // 分辨业务函数返回的结果
		// return match res {
		// 	// 将业务错误转成BaseResp进行传递.
		// 	Err(e) => {
		// 		Ok(volo_gen::rust_demo_2::GetItemBackendResponse {
		// 			base_resp: e.into(),
		// 			..Default::default()
		// 		})
		// 	}
		// 	Ok(biz) => {
		// 		Ok(Default::default())
		// 	}
		// }

		// 3.3
		let res = func0()?;
		Ok(Default::default())
	}
}

fn func0() -> Result<(), ServiceError> {
	Err(TEST_ERROR_CODE.into())
}

impl From<ServiceError> for volo_gen::rust_demo_2::BaseResp {
	fn from(e: ServiceError) -> Self {
		match e {
			ServiceError::BizError(e) => {
				Self {
					status_code: Some(e.code),
					status_message: Some(e.message.into()),
					extra: None,
				}
			},
			_ => {
				Self {
					status_code: Some(INTERNAL_ERROR_CODE.code),
					status_message: Some(INTERNAL_ERROR_CODE.message.into()),
					extra: None,
				}
			},
		}
	}
}

pub static TEST_ERROR_CODE: &ErrCode = &ErrCode {
	code: 12345678,
	message: "我是谁",
};
pub static INTERNAL_ERROR_CODE: &ErrCode = &ErrCode {
	code: -1,
	message: "系统错误",
};

#[derive(Clone, Debug, Default)]
pub struct ErrCode {
	pub code: i32,
	pub message: &'static str,
}
impl std::fmt::Display for &ErrCode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "biz error, code: {}, msg: {}", self.code, self.message)
	}
}
impl From<&'static ErrCode> for ServiceError {
	fn from(e: &'static ErrCode) -> Self {
		Self::BizError(&e)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
	#[error("biz error: {0}")]
	BizError(&'static ErrCode),
	#[error(transparent)]
	Other(#[from] anyhow::Error),
}

pub trait SetBaseResp {
	fn set_base_resp(&mut self, code: i32, msg: &'static str) {}
}