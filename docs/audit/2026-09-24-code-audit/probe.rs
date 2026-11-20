// Library probes P1–P11 from docs/audit/2026-09-24-code-audit.md (copied from gitignored scratch
// 2026-11-20, unchanged below this header). Historical evidence: probes the pre-fix wedi-core API of
// 2026-09-24 and may not compile against the current tree.
// Run: put this file at src/main.rs of a throwaway crate with
//   [dependencies] wedi-core = { path = "<repo>/crates/wedi-core", features = ["syntax-highlighting"] }
//   encoding_rs = "0.8"; tempfile = "3.8"
// and `cargo run --release`.
//! Audit probes: each prints the observed behavior of one suspected defect.
use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Instant;

use wedi_core::buffer::{EncodingConfig, RopeBuffer};
use wedi_core::comment::CommentHandler;
use wedi_core::highlight::HighlightEngine;
use wedi_core::{Cursor, Search, View};

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as isize, Ordering::Relaxed);
        System.dealloc(p, l)
    }
}
#[global_allocator]
static A: Counting = Counting;

fn none() -> EncodingConfig {
    EncodingConfig { read_encoding: None, save_encoding: None }
}

fn main() {
    let dir = tempfile::TempDir::new().unwrap();

    // P1 UTF-16LE round trip
    let p = dir.path().join("u16.txt");
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend("Hello".encode_utf16().flat_map(|c| c.to_le_bytes()));
    fs::write(&p, &bytes).unwrap();
    let mut b = RopeBuffer::from_file_with_encoding(&p, &none()).unwrap();
    b.insert(0, "X");
    b.save().unwrap();
    let out = fs::read(&p).unwrap();
    println!("P1 utf16le: save_encoding={} in={:02X?} out={:02X?} out_is_utf8={:?}",
        b.save_encoding().name(), &bytes[..6], &out[..out.len().min(8)], std::str::from_utf8(&out).ok());

    // P2 UTF-8 BOM round trip
    let p = dir.path().join("bom.txt");
    fs::write(&p, [0xEF, 0xBB, 0xBF, b'h', b'i']).unwrap();
    let mut b = RopeBuffer::from_file_with_encoding(&p, &none()).unwrap();
    b.save().unwrap();
    println!("P2 utf8 bom: out={:02X?}", fs::read(&p).unwrap());

    // P3 unmappable char under GBK
    let p = dir.path().join("gbk.txt");
    let mut b = RopeBuffer::new();
    b.set_save_encoding(encoding_rs::GBK);
    b.insert(0, "a\u{1F600}b");
    let r = b.save_to(&p);
    println!("P3 gbk emoji: save_result_ok={} file={:?}", r.is_ok(), String::from_utf8_lossy(&fs::read(&p).unwrap()));

    // P4 search column unit
    let mut b = RopeBuffer::new();
    b.insert(0, "中文abc\nxyz\n");
    let mut s = Search::new();
    s.set_query("abc".into());
    s.find_matches(&b);
    println!("P4 search: matches={:?} line0_char_len={}", s.get_matches(), "中文abc".chars().count());

    // P5 view cache lookup for row above offset_row
    let mut b = RopeBuffer::new();
    b.insert(0, &format!("{}\nshort\nshort\nshort\n", "x".repeat(50)));
    let mut v = View::new_simple(5, 24);
    v.offset_row = 1;
    let mut c = Cursor::new();
    c.row = 1;
    v.scroll_if_needed(&c, &b, false);
    let above = v.calculate_visual_lines_for_row(&b, 0);
    let mut fresh = View::new_simple(5, 24);
    fresh.offset_row = 0;
    let truth = fresh.calculate_visual_lines_for_row(&b, 0);
    println!("P5 view row0 while offset=1: got {} visual lines {:?}; true {} lines", above.len(), above, truth.len());
    c.move_up(&b, &v);
    println!("P5 move_up from row1: row={} visual_line_index={} (expected last={})", c.row, c.visual_line_index, truth.len() - 1);

    // P6 per-frame highlighter leak
    let mut eng = HighlightEngine::new(None, true).unwrap();
    eng.set_file(Some(Path::new("x.rs")));
    drop(eng.create_highlighter());
    let before = LIVE.load(Ordering::Relaxed);
    for _ in 0..100 {
        drop(eng.create_highlighter());
    }
    let after = LIVE.load(Ordering::Relaxed);
    println!("P6 create_highlighter x100 leaked {} bytes ({} B/frame)", after - before, (after - before) / 100);

    // P7 undo granularity
    let mut b = RopeBuffer::new();
    for (i, ch) in "hello".chars().enumerate() {
        b.insert_char(i, ch);
    }
    let mut n = 0;
    while b.undo().is_some() {
        n += 1;
    }
    println!("P7 typing 'hello' needs {} undo steps", n);

    // P8 CRLF backspace at line start (replicates Editor::handle_command Backspace, row>0 col==0)
    let mut b = RopeBuffer::new();
    b.insert(0, "abc\r\ndef");
    let prev_len = b.get_line_content(0).trim_end_matches(['\n', '\r']).chars().count();
    let pos = b.line_to_char(0) + prev_len;
    b.delete_char(pos);
    println!("P8 crlf backspace: lines={} text={:?}", b.line_count(), b.get_line_content(0) + &b.get_line_content(1));

    // P9/P10 comment toggling
    let mut ch = CommentHandler::new();
    ch.detect_from_path(Path::new("Makefile"));
    println!("P9 Makefile tab line: {:?} -> {:?}", "\tcc -o x", ch.toggle_line_comment("\tcc -o x"));
    ch.detect_from_path(Path::new("data.json"));
    println!("P10 json: {:?}; html: {:?}", ch.toggle_line_comment("{"), {
        let mut h = CommentHandler::new();
        h.detect_from_path(Path::new("i.html"));
        h.toggle_line_comment("<p>")
    });

    // P11 per-frame highlight cost for a 500-line Rust file
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../src/editor.rs")).unwrap();
    let lines: Vec<String> = src.split_inclusive('\n').take(500).map(String::from).collect();
    let t = Instant::now();
    let frames = 20;
    for _ in 0..frames {
        let mut h = eng.create_highlighter().unwrap();
        for l in &lines {
            let _ = h.highlight_line(l);
        }
    }
    println!("P11 highlight 500 lines: {:.2} ms/frame", t.elapsed().as_secs_f64() * 1000.0 / frames as f64);
}
