//! `endpoint` 领域模块：本模块自身私有，对外只通过 re-export 暴露三个
//! 公共 item。三个子模块同样私有，内部文件如何拆分是库的实现细节。

mod error;
mod model;
mod parser;

pub use error::ParseEndpointError;
pub use model::Endpoint;
pub use parser::parse_endpoint;
