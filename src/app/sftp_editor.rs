//! 内置 SFTP 文件查看 / 编辑器 —— Rust 侧辅助函数
//! 编辑器修改：语法着色层（注释行 / 普通行）的刷新（qian 分支特性）。
//! 编辑器 UI 现在位于独立顶层窗口 ui/editor_window.slint（EditorWindow），
//! 行号槽由 Slint 侧按 editor-lines 逐行测量，不再需要 Rust 生成行号文本。

use crate::ui::*;

/// 编辑器修改：更新内置文本编辑器（SFTP 查看/编辑）的着色层。
/// 首个非空白字符为 `#` 的行归入绿色 `editor-comment-text` 层，其余行归入
/// `editor-normal-text` 层；两层都绘制在半透明的编辑器 TextInput 之下，
/// 从而透出颜色实现语法高亮（qian 分支特性）。
pub(crate) fn update_editor_text_layers(editor: &EditorWindow, content: &str) {
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
    editor.set_editor_comment_text(comment_lines.join("\n").into());
    editor.set_editor_normal_text(normal_lines.join("\n").into());
}
