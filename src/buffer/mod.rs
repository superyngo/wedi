mod history;
mod rope_buffer;

pub use rope_buffer::RopeBuffer;

#[derive(Debug, Clone)]
pub struct EncodingConfig {
    pub read_encoding: Option<&'static encoding_rs::Encoding>,
    pub save_encoding: Option<&'static encoding_rs::Encoding>,
}

// #[derive(Debug, Clone)]
// pub struct EncodingSpec {
//     pub encoding: Option<&'static encoding_rs::Encoding>,
//     pub is_user_specified: bool,
// }
