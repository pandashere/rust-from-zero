//! 第三课主练习脚手架：拥有型笔记 `Note`。
//!
//! 每个 `todo!()` 都是学习者负责实现的部分。实现前行为测试会失败，
//! 这是学习流程的一部分；规格见 README「主练习」一节，卡住时按级查看 HINTS.md。

pub mod text;

/// 一条拥有标题与正文的笔记。标题和正文都归笔记所有。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Note {
    title: String,
    body: String,
}

impl Note {
    /// 创建一条笔记：取得 `title` 与 `body` 两个字符串的所有权，
    /// 把 `title` 的内容存为笔记标题，把 `body` 的内容存为笔记正文。
    /// 调用后传入的两个字符串由原绑定转归笔记所有。
    pub fn new(title: String, body: String) -> Self {
        let _ = (&title, &body);
        todo!()
    }

    /// 返回笔记标题的借用视图（`&str`）：标题内容原样返回，不复制、不修改。
    /// 只是借用，所以调用后笔记本身必须仍然可用。
    pub fn title(&self) -> &str {
        todo!()
    }

    /// 返回笔记正文的借用视图（`&str`）：正文内容原样返回，不复制、不修改。
    /// 只是借用，所以调用后笔记本身必须仍然可用。
    pub fn body(&self) -> &str {
        todo!()
    }

    /// 把 `line` 追加为正文的新一行：
    /// - 正文当前为空字符串时，正文直接变成 `line` 的内容，前面不加任何字符；
    /// - 正文非空时，先在正文末尾追加一个换行符 `'\n'`，再追加 `line` 的内容。
    ///
    /// 只修改正文，标题保持不变；调用结束后笔记仍归调用者所有。
    pub fn append_line(&mut self, line: &str) {
        let _ = &line;
        todo!()
    }

    /// 消费整条笔记，把标题与正文以拥有型字符串取出：
    /// 返回二元组 `(标题, 正文)`，两个元素的内容分别与取出前的标题、正文完全一致。
    /// 调用后原笔记不再可用，返回的两个字符串互不影响，可各自独立修改。
    pub fn into_parts(self) -> (String, String) {
        todo!()
    }
}
