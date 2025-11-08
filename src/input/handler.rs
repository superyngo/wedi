#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // 字符輸入
    Insert(char),
    
    // 刪除操作
    Delete,
    Backspace,
    DeleteLine,
    
    // 光標移動
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveHome,
    MoveEnd,
    PageUp,
    PageDown,
    MoveToFileStart,  // Ctrl+Up: 跳到第一行
    MoveToFileEnd,    // Ctrl+Down: 跳到最後一行
    MoveToLineStart,  // Ctrl+Left: 跳到行首
    MoveToLineEnd,    // Ctrl+Right: 跳到行尾
    
    // 剪貼板操作
    Copy,
    Cut,
    Paste,
    
    // 文件操作
    Save,
    Quit,
    
    // 撤銷/重做
    Undo,
    Redo,
    
    // 搜索
    Find,
    FindNext,
    FindPrev,
    
    // 視圖控制
    ToggleLineNumbers,
    
    // 註解切換
    ToggleComment,
    
    // 縮排操作
    Indent,
    Unindent,
    
    // 選擇操作
    SelectAll,
    ExtendSelection(Direction),
    ClearSelection,
    
    // 跳轉
    GoToLine,
    
    // 清除訊息
    ClearMessage,
}
