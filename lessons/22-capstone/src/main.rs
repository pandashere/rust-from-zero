//! `datakit` 的命令行入口。
//!
//! 真正的逻辑都在库里；这里只把操作系统参数交给 [`l22_capstone::app`]，
//! 并用它返回的数字设置进程退出码。

fn main() {
    std::process::exit(l22_capstone::app::main_entry(std::env::args().skip(1)));
}
