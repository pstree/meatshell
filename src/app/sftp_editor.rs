//! 内置 SFTP 文件查看 / 编辑器 —— Rust 侧辅助函数
//! 编辑器修改：由 app.rs 移入 sftp 模块（与 ui/sftp_editor.slint 对应）。
//! 包含：行号槽文本的生成，以及语法着色层（注释行/普通行）的刷新。

use crate::ui::*;

/// Build the editor's line-number gutter text: "1\n2\n…\nN", one number per line
/// of `content`, matching its (newline-separated) line count (#81).
pub(crate) fn line_numbers_for(content: &str) -> String {
    use std::fmt::Write;
    let lines = content.split('\n').count().max(1);
    let mut s = String::with_capacity(lines * 4);
    for i in 1..=lines {
        if i > 1 {
            s.push('\n');
        }
        let _ = write!(s, "{i}");
    }
    s
}

/// 编辑器修改：更新内置文本编辑器（SFTP 查看/编辑）的着色层。
/// 首个非空白字符为 `#` 的行归入绿色 `editor-comment-text` 层，其余行归入
/// `editor-normal-text` 层；两层都绘制在半透明的编辑器 TextInput 之下，
/// 从而透出颜色实现语法高亮（qian 分支特性）。
pub(crate) fn update_editor_text_layers(win: &AppWindow, content: &str) {
    let mut comment_lines = Vec::new();
    let mut normal_lines = Vec::new();
    for line in content.split('\n') {
        if line.trim_start().starts_with('#') {
            comment_lines.push(line);
            normal_lines.push("");
        } else {
            comment_lines.push("");
            normal_lines.push(line);
        }
    }
    win.set_editor_comment_text(comment_lines.join("\n").into());
    win.set_editor_normal_text(normal_lines.join("\n").into());
}