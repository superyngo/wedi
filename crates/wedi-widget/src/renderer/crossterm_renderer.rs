use crate::state::EditorState;
use anyhow::Result;
use std::io::Write;

/// Crossterm 渲染器
pub struct CrosstermRenderer;

impl CrosstermRenderer {
    /// 渲染編輯器到指定的輸出
    pub fn render<W: Write>(
        state: &EditorState,
        stdout: &mut W,
    ) -> Result<()> {
        // 使用 wedi-core 的 View::render 方法
        state.view.render(
            &state.buffer,
            &state.cursor,
            &state.selection,
            #[cfg(feature = "syntax-highlighting")]
            state.highlight_engine.as_ref(),
            #[cfg(feature = "syntax-highlighting")]
            &state.highlight_cache,
        )?;

        Ok(())
    }
}
