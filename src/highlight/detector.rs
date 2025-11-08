// 文件類型檢測
// 這個模組將在後續階段實現

use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Go,
    Java,
    Html,
    Css,
    Markdown,
    Json,
    Yaml,
    Toml,
    Shell,
    Unknown,
}

impl FileType {
    #[allow(dead_code)]
    pub fn from_path(path: &Path) -> Self {
        let extension = path.extension().and_then(|s| s.to_str());

        match extension {
            Some("rs") => FileType::Rust,
            Some("py") => FileType::Python,
            Some("js") => FileType::JavaScript,
            Some("ts") => FileType::TypeScript,
            Some("c") | Some("h") => FileType::C,
            Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") => FileType::Cpp,
            Some("go") => FileType::Go,
            Some("java") => FileType::Java,
            Some("html") | Some("htm") => FileType::Html,
            Some("css") => FileType::Css,
            Some("md") | Some("markdown") => FileType::Markdown,
            Some("json") => FileType::Json,
            Some("yaml") | Some("yml") => FileType::Yaml,
            Some("toml") => FileType::Toml,
            Some("sh") | Some("bash") => FileType::Shell,
            _ => FileType::Unknown,
        }
    }
}
