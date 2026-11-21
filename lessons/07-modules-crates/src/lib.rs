//! 第七课《module、crate、package 与可见性》的练习脚手架。
//!
//! 本 crate 同时是一个 library 和一个 binary：
//!
//! - library 侧提供两个领域模块，分别对应主练习和课后巩固：`endpoint`
//!   解析 `host:port` 文本，`module_path` 校验模块路径文本；
//! - binary 侧（`src/main.rs`）是另一个 crate，只通过本文件暴露的公共 API
//!   使用这两个领域，示范“薄 main”。
//!
//! 两个领域模块都保持私有，crate root 只用 `pub use` 暴露稳定的名字。
//! 文件怎么摆、内部拆成几个模块，都是库可以随时调整的实现细节；调用者
//! 只依赖这里给出的公共路径。
//!
//! 完成实现前，调用公共函数会执行到 `todo!()` 并 panic。这是脚手架的预期
//! 状态，不是编译错误。
//!
//! # 两条边界，两个 compile_fail 例子
//!
//! 下面两段代码故意无法编译，`cargo test` 会把它们当作文档测试运行，并
//! 确认“确实编译失败”。
//!
//! 第一段说明：私有模块不能从 crate 外穿过，即使里面的类型是 `pub`。
//!
//! ```compile_fail
//! use l07_modules_crates::endpoint::Endpoint;
//! # fn main() {}
//! ```
//!
//! 第二段说明：字段私有后，外部不能用 struct 字面量绕过验证。
//!
//! ```compile_fail
//! use l07_modules_crates::Endpoint;
//!
//! fn main() {
//!     let _endpoint = Endpoint {
//!         host: String::from("example.com"),
//!         port: 443,
//!     };
//! }
//! ```

mod endpoint;
mod module_path;

pub use endpoint::{Endpoint, ParseEndpointError, parse_endpoint};
pub use module_path::{ModulePath, ParseModulePathError, parse_module_path};
