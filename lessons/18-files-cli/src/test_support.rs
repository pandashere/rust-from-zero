//! 测试共用的小工具：每个测试创建独立的临时目录，结束时自动删除。
//!
//! 只在 `cargo test` 编译测试时参与构建，不会进入正式程序。

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

/// 一个独立的临时目录。离开作用域时自动删除，避免测试之间互相干扰。
pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// 创建目录。名字同时包含进程号、时间戳和递增序号，并行测试不会重名。
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before 1970")
            .as_nanos();
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "rust-from-zero-l18-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("failed to create a temporary directory");
        Self { path }
    }

    /// 在本目录下拼出一个子路径。不会创建文件。
    pub fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // 清理失败不应让测试结果变成“失败”，忽略即可。
        let _ = fs::remove_dir_all(&self.path);
    }
}
