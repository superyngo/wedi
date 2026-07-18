mod history;
mod rope_buffer;

pub use rope_buffer::RopeBuffer;

#[derive(Debug, Clone)]
pub struct EncodingConfig {
    pub read_encoding: Option<&'static encoding_rs::Encoding>,
    pub save_encoding: Option<&'static encoding_rs::Encoding>,
}

/// 解析編碼標籤字串為編碼（命令列參數與 Ctrl+E 對話框共用的單一來源）
pub fn parse_encoding_label(label: &str) -> Option<&'static encoding_rs::Encoding> {
    match label.to_lowercase().as_str() {
        "utf-8" | "utf8" => Some(encoding_rs::UTF_8),
        "utf-16le" | "utf16le" => Some(encoding_rs::UTF_16LE),
        "utf-16be" | "utf16be" => Some(encoding_rs::UTF_16BE),
        "gbk" | "cp936" => Some(encoding_rs::GBK),
        "shift-jis" | "shift_jis" | "sjis" => Some(encoding_rs::SHIFT_JIS),
        "big5" | "cp950" => encoding_rs::Encoding::for_label(b"big5"),
        "cp1252" | "windows-1252" => Some(encoding_rs::WINDOWS_1252),
        _ => encoding_rs::Encoding::for_label(label.as_bytes()),
    }
}

// #[derive(Debug, Clone)]
// pub struct EncodingSpec {
//     pub encoding: Option<&'static encoding_rs::Encoding>,
//     pub is_user_specified: bool,
// }
