# 🤖 Copilot CLI Session

> **Session ID:** `6946668f-d612-485f-8dc9-cb85794624ea`  
> **Started:** 2025/12/26 下午3:36:40  
> **Duration:** 13m 25s  
> **Exported:** 2025/12/26 下午3:50:06

<sub>⏱️ 0s</sub>

### ℹ️ Info

Logged in with gh as user: superyngo

---

<sub>⏱️ 2s</sub>

### ℹ️ Info

Connected to GitHub MCP Server

---

<sub>⏱️ 39s</sub>

### ℹ️ Info

Total usage est:       0 Premium requests
Total duration (API):  0s
Total duration (wall): 39s
Total code changes:    0 lines added, 0 lines removed

---

<sub>⏱️ 5m 59s</sub>

### 👤 User

評估單行/多行顯示切換。
1.初步想法是整合到行號顯示，只要透過ctrl+l一起toggle。
	顯示行號=\>多行顯示
	關閉行號=\>單行顯示
2.單行顯示時超過螢幕寬度的部分如何移動游標過去顯示？
3.對sytex hightling的影響?
請先評估。

---

<sub>⏱️ 6m 7s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi.**

D:\Users\user\Documents\rust\wedi

<details>
<summary>15 lines</summary>

```
.claude
.git
.github
.gitignore
assets
Cargo.lock
Cargo.toml
CHANGELOG.md
dev
install.ps1
install.sh
LICENSE
README.md
src
target
```

</details>


---

<sub>⏱️ 6m 7s</sub>

### ✅ `grep`

"show_line_numbers|ctrl.*l|Ctrl.*L" (D:\Users\user\Documents\rust\wedi\src)

```
D:\Users\user\Documents\rust\wedi\src\help.rs
D:\Users\user\Documents\rust\wedi\src\input\handler.rs
D:\Users\user\Documents\rust\wedi\src\view.rs
```


---

<sub>⏱️ 6m 12s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi\src.**

D:\Users\user\Documents\rust\wedi\src

<details>
<summary>16 lines</summary>

```
buffer
clipboard.rs
comment.rs
config.rs
cursor.rs
dialog.rs
editor.rs
help.rs
highlight
input
lib.rs
main.rs
search.rs
terminal.rs
utils
view.rs
```

</details>


---

<sub>⏱️ 6m 12s</sub>

### ✅ `grep`

"wrap|Wrap|horizontal.*scroll|scroll.*horizontal" (D:\Users\user\Documents\rust\wedi\src)

<details>
<summary>9 matches</summary>

```
D:\Users\user\Documents\rust\wedi\src\comment.rs
D:\Users\user\Documents\rust\wedi\src\editor.rs
D:\Users\user\Documents\rust\wedi\src\buffer\rope_buffer.rs
D:\Users\user\Documents\rust\wedi\src\main.rs
D:\Users\user\Documents\rust\wedi\src\view.rs
D:\Users\user\Documents\rust\wedi\src\highlight\cache.rs
D:\Users\user\Documents\rust\wedi\src\highlight\engine.rs
D:\Users\user\Documents\rust\wedi\src\utils\line_wrapper.rs
D:\Users\user\Documents\rust\wedi\src\utils\mod.rs
```

</details>


---

<sub>⏱️ 6m 12s</sub>

### ℹ️ Info

Operation cancelled by user

---

<sub>⏱️ 6m 37s</sub>

### 👤 User

還有單行顯示時的selection行為也要評估

---

<sub>⏱️ 6m 44s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi\src\view.rs.**

D:\Users\user\Documents\rust\wedi\src\view.rs

<details>
<summary>805 lines</summary>

```
1. use crate::buffer::RopeBuffer;
2. use crate::cursor::Cursor;
3. use crate::terminal::Terminal;
4. use crate::utils::visual_width;
5. use anyhow::Result;
6. use crossterm::{
7.     cursor, execute, queue,
8.     style::{self, Attribute, Color},
9. };
10. use std::io::{self, Write};
11. use unicode_width::UnicodeWidthChar;
12. 
13. // 視圖配置常量
14. const TAB_WIDTH: usize = 4; // Tab 寬度（空格數）
15. const CACHE_MULTIPLIER: usize = 3; // 緩存大小倍數（螢幕行數 × 倍數）
16. 
17. #[derive(Clone, Debug)]
18. pub struct LineLayout {
19.     /// 視覺行（已處理 Tab 並依螢幕寬度換行）
20.     pub visual_lines: Vec<String>,
21.     /// 視覺行高度（visual_lines.len()）
22.     pub visual_height: usize,
23.     /// logical_col -> visual_col（整行累計視覺座標）
24.     pub logical_to_visual: Vec<usize>,
25. }
26. 
27. impl LineLayout {
28.     pub fn new(buffer: &RopeBuffer, row: usize, available_width: usize) -> Option<Self> {
29.         let line = buffer.line(row)?;
30.         let mut line_str = line.to_string();
31.         // 去掉結尾換行符
32.         while matches!(line_str.chars().last(), Some('\n' | '\r')) {
33.             line_str.pop();
34.         }
35. 
36.         let (displayed_line, logical_to_visual) = expand_tabs_and_build_map(&line_str);
37.         let visual_lines = wrap_line(&displayed_line, available_width);
38.         let visual_height = visual_lines.len();
39. 
40.         Some(LineLayout {
41.             visual_lines,
42.             visual_height,
43.             logical_to_visual,
44.         })
45.     }
46. }
47. 
48. fn expand_tabs_and_build_map(line: &str) -> (String, Vec<usize>) {
49.     let mut displayed = String::new();
50.     let mut logical_to_visual = Vec::new();
51.     let mut visual_col = 0;
52. 
53.     for ch in line.chars() {
54.         // 記錄「這個 logical_col 對應的視覺座標」
55.         logical_to_visual.push(visual_col);
56. 
57.         if ch == '\t' {
58.             for _ in 0..TAB_WIDTH {
59.                 displayed.push(' ');
60.             }
61.             visual_col += TAB_WIDTH;
62.         } else {
63.             let w = UnicodeWidthChar::width(ch).unwrap_or(1);
64.             displayed.push(ch);
65.             visual_col += w;
66.         }
67.     }
68. 
69.     // 尾端一個 mapping，讓「行尾」也有對應視覺座標
70.     logical_to_visual.push(visual_col);
71. 
72.     (displayed, logical_to_visual)
73. }
74. 
75. #[allow(dead_code)]
76. fn calculate_hash(line: &str) -> u64 {
77.     use std::collections::hash_map::DefaultHasher;
78.     use std::hash::{Hash, Hasher};
79. 
80.     let mut hasher = DefaultHasher::new();
81.     line.hash(&mut hasher);
82.     hasher.finish()
83. }
84. 
85. #[derive(Debug, Clone, Copy)]
86. pub struct Selection {
87.     pub start: (usize, usize), // (row, col)
88.     pub end: (usize, usize),   // (row, col)
89. }
90. 
91. pub struct View {
92.     pub offset_row: usize, // 視窗頂部顯示的行號（邏輯行）
93.     pub show_line_numbers: bool,
94.     pub screen_rows: usize,
95.     pub screen_cols: usize,
96.     // 行快取：從 offset_row 起往下的數行
97.     line_layout_cache: Vec<Option<LineLayout>>,
98. }
99. 
100. impl View {
101.     pub fn new(terminal: &Terminal) -> Self {
102.         let (cols, rows) = terminal.size();
103.         let screen_rows = rows.saturating_sub(1) as usize; // 減去狀態欄
104.         let cache_size = screen_rows.max(1) * CACHE_MULTIPLIER;
105. 
106.         Self {
107.             offset_row: 0,
108.             show_line_numbers: true,
109.             screen_rows,
110.             screen_cols: cols as usize,
111.             line_layout_cache: vec![None; cache_size],
112.         }
113.     }
114. 
115.     /// 完全清空緩存（用於大範圍變更或視窗調整）
116.     pub fn invalidate_cache(&mut self) {
117.         let cache_size = self.screen_rows.max(1) * CACHE_MULTIPLIER;
118.         self.line_layout_cache.clear();
119.         self.line_layout_cache.resize(cache_size, None);
120.     }
121. 
122.     /// 部分失效：僅清除指定邏輯行的緩存（用於單行編輯）
123.     pub fn invalidate_line(&mut self, logical_row: usize) {
124.         if logical_row < self.offset_row {
125.             return; // 不在可見範圍內，無需清除
126.         }
127. 
128.         let cache_index = logical_row.saturating_sub(self.offset_row);
129.         if cache_index < self.line_layout_cache.len() {
130.             self.line_layout_cache[cache_index] = None;
131.         }
132.     }
133. 
134.     /// 部分失效：清除指定範圍的緩存（用於多行編輯）
135.     #[allow(dead_code)]
136.     pub fn invalidate_lines(&mut self, start_row: usize, end_row: usize) {
137.         for row in start_row..=end_row {
138.             self.invalidate_line(row);
139.         }
140.     }
141. 
142.     #[allow(dead_code)]
143.     pub fn update_size(&mut self) {
144.         let size = crossterm::terminal::size().unwrap_or((80, 24));
145.         let new_screen_rows = size.1.saturating_sub(1) as usize;
146.         let new_screen_cols = size.0 as usize;
147. 
148.         if self.screen_rows != new_screen_rows || self.screen_cols != new_screen_cols {
149.             self.screen_rows = new_screen_rows;
150.             self.screen_cols = new_screen_cols;
151.             self.invalidate_cache(); // 寬度或高度改變時使快取失效
152.         }
153.     }
154. 
155.     pub fn render(
156.         &mut self,
157.         buffer: &RopeBuffer,
158.         cursor: &Cursor,
159.         selection: Option<&Selection>,
160.         message: Option<&str>,
161.         #[cfg(feature = "syntax-highlighting")] highlighted_lines: Option<
162.             &std::collections::HashMap<usize, String>,
163.         >,
164.     ) -> Result<()> {
165.         let has_debug_ruler = message.is_some_and(|m| m.starts_with("DEBUG"));
166. 
167.         self.scroll_if_needed(cursor, buffer, has_debug_ruler);
168. 
169.         let mut stdout = io::stdout();
170. 
171.         execute!(stdout, cursor::Hide)?;
172.         execute!(stdout, cursor::MoveTo(0, 0))?;
173. 
174.         let ruler_offset = if has_debug_ruler {
175.             self.render_column_ruler(&mut stdout, buffer)?;
176.             1
177.         } else {
178.             0
179.         };
180. 
181.         let line_num_width = self.calculate_line_number_width(buffer);
182.         let available_width = self.get_available_width(buffer);
183. 
184.         // 計算選擇範圍（轉換為視覺列）
185.         let sel_visual_range = selection.map(|sel| {
186.             let (start_row, start_col) = sel.start.min(sel.end);
187.             let (end_row, end_col) = sel.start.max(sel.end);
188. 
189.             // 將start_col轉換為視覺列
190.             let start_visual_col = if start_row < buffer.line_count() {
191.                 let line = buffer
192.                     .line(start_row)
193.                     .map(|s| s.to_string())
194.                     .unwrap_or_default();
195.                 let line = line.trim_end_matches(['\n', '\r']);
196.                 self.logical_col_to_visual_col(line, start_col)
197.             } else {
198.                 start_col
199.             };
200. 
201.             // 將end_col轉換為視覺列
202.             let end_visual_col = if end_row < buffer.line_count() {
203.                 let line = buffer
204.                     .line(end_row)
205.                     .map(|s| s.to_string())
206.                     .unwrap_or_default();
207.                 let line = line.trim_end_matches(['\n', '\r']);
208.                 self.logical_col_to_visual_col(line, end_col)
209.             } else {
210.                 end_col
211.             };
212. 
213.             ((start_row, start_visual_col), (end_row, end_visual_col))
214.         });
215. 
216.         let mut screen_row = ruler_offset;
217.         let mut file_row = self.offset_row;
218. 
219.         while screen_row < self.screen_rows && file_row < buffer.line_count() {
220.             queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;
221. 
222.             if self.show_line_numbers {
223.                 let line_num = format!("{:>width$} ", file_row + 1, width = line_num_width - 1);
224.                 queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
225.                 queue!(stdout, style::Print(&line_num))?;
226.                 queue!(stdout, style::ResetColor)?;
227.             }
228. 
229.             let cache_index = file_row.saturating_sub(self.offset_row);
230.             let layout_opt = self
231.                 .line_layout_cache
232.                 .get(cache_index)
233.                 .and_then(|l| l.as_ref())
234.                 .cloned();
235. 
236.             let layout = if let Some(layout) = layout_opt {
237.                 layout
238.             } else if let Some(new_layout) = LineLayout::new(buffer, file_row, available_width) {
239.                 if cache_index < self.line_layout_cache.len() {
240.                     self.line_layout_cache[cache_index] = Some(new_layout.clone());
241.                 }
242.                 new_layout
243.             } else {
244.                 // 空行或超出範圍
245.                 LineLayout {
246.                     visual_lines: vec![String::new()],
247.                     visual_height: 1,
248.                     logical_to_visual: vec![0],
249.                 }
250.             };
251. 
252.             for (visual_idx, visual_line) in layout.visual_lines.iter().enumerate() {
253.                 if screen_row >= self.screen_rows {
254.                     break;
255.                 }
256. 
257.                 if visual_idx > 0 {
258.                     screen_row += 1;
259.                     if screen_row >= self.screen_rows {
260.                         break;
261.                     }
262.                     queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;
263. 
264.                     if self.show_line_numbers {
265.                         for _ in 0..line_num_width {
266.                             queue!(stdout, style::Print(" "))?;
267.                         }
268.                     }
269.                 }
270. 
271.                 // 渲染視覺行，支持 selection 高亮和語法高亮
272. 
273.                 // 檢查是否有語法高亮（無選擇時）
274.                 #[cfg(feature = "syntax-highlighting")]
275.                 let use_syntax_highlight = selection.is_none()
276.                     && visual_idx == 0  // 只在第一個 visual line 使用（簡化處理）
277.                     && highlighted_lines.and_then(|h| h.get(&file_row)).is_some();
278. 
279.                 #[cfg(not(feature = "syntax-highlighting"))]
280.                 let use_syntax_highlight = false;
281. 
282.                 if let Some(((start_row, start_col), (end_row, end_col))) = sel_visual_range {
283.                     if file_row >= start_row && file_row <= end_row {
284.                         // 這一行有選擇，需要逐字符渲染
285.                         // 計算這個visual_line在整個邏輯行中的視覺起始位置
286.                         let visual_line_start: usize = layout
287.                             .visual_lines
288.                             .iter()
289.                             .take(visual_idx)
290.                             .map(|line| visual_width(line))
291.                             .sum();
292. 
293.                         let chars: Vec<char> = visual_line.chars().collect();
294.                         let mut current_visual_pos = visual_line_start;
295. 
296.                         for &ch in chars.iter() {
297.                             let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
298. 
299.                             // 判斷這個字符是否在選擇範圍內
300.                             let is_selected = if file_row == start_row && file_row == end_row {
301.                                 // 選擇在同一行
302.                                 current_visual_pos >= start_col && current_visual_pos < end_col
303.                             } else if file_row == start_row {
304.                                 // 選擇起始行
305.                                 current_visual_pos >= start_col
306.                             } else if file_row == end_row {
307.                                 // 選擇結束行
308.                                 current_visual_pos < end_col
309.                             } else {
310.                                 // 選擇中間的行，全選
311.                                 true
312.                             };
313. 
314.                             if is_selected {
315.                                 queue!(stdout, style::SetAttribute(Attribute::Reverse))?;
316.                             }
317.                             queue!(stdout, style::Print(ch))?;
318.                             if is_selected {
319.                                 queue!(stdout, style::SetAttribute(Attribute::NoReverse))?;
320.                             }
321. 
322.                             current_visual_pos += ch_width;
323.                         }
324.                     } else {
325.                         // 這一行沒有選擇，直接打印
326.                         queue!(stdout, style::Print(visual_line))?;
327.                     }
328.                 } else {
329.                     // 沒有選擇
330.                     if use_syntax_highlight {
331.                         // 使用語法高亮
332.                         #[cfg(feature = "syntax-highlighting")]
333.                         if let Some(highlighted) = highlighted_lines.and_then(|h| h.get(&file_row))
334.                         {
335.                             // 輸出高亮後的文字（包含 ANSI 色碼）
336.                             queue!(stdout, style::Print(highlighted))?;
337.                         } else {
338.                             // 降級為純文字
339.                             queue!(stdout, style::Print(visual_line))?;
340.                         }
341. 
342.                         #[cfg(not(feature = "syntax-highlighting"))]
343.                         queue!(stdout, style::Print(visual_line))?;
344.                     } else {
345.                         // 純文字渲染
346.                         queue!(stdout, style::Print(visual_line))?;
347.                     }
348.                 }
349. 
350.                 queue!(
351.                     stdout,
352.                     crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
353.                 )?;
354.             }
355. 
356.             screen_row += 1;
357.             file_row += 1;
358.         }
359. 
360.         // 畫底部的 ~ 行
361.         while screen_row < self.screen_rows {
362.             queue!(stdout, cursor::MoveTo(0, screen_row as u16))?;
363.             queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
364.             queue!(stdout, style::Print("~"))?;
365.             queue!(stdout, style::ResetColor)?;
366.             queue!(
367.                 stdout,
368.                 crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
369.             )?;
370.             screen_row += 1;
371.         }
372. 
373.         self.render_status_bar(buffer, selection.is_some(), message, cursor)?;
374. 
375.         // 移動終端光標到當前cursor位置
376.         let ruler_offset = if has_debug_ruler { 1 } else { 0 };
377.         let (cursor_x, cursor_y) = self.get_cursor_visual_position(cursor, buffer);
378.         let cursor_y = cursor_y + ruler_offset;
379.         execute!(stdout, cursor::MoveTo(cursor_x as u16, cursor_y as u16))?;
380. 
381.         execute!(stdout, cursor::Show)?;
382.         stdout.flush()?;
383.         Ok(())
384.     }
385. 
386.     pub fn scroll_if_needed(
387.         &mut self,
388.         cursor: &Cursor,
389.         buffer: &RopeBuffer,
390.         has_debug_ruler: bool,
391.     ) {
392.         // 向上滾動
393.         if cursor.row < self.offset_row {
394.             self.offset_row = cursor.row;
395.             self.invalidate_cache();
396.             return;
397.         }
398. 
399.         let effective_rows = self.get_effective_screen_rows(has_debug_ruler);
400. 
401.         // 大幅跳轉優化：如果跳轉距離超過 3 個螢幕高度，直接設置 offset_row
402.         // 這避免了計算中間所有行的視覺高度，大幅提升大文件跳轉性能
403.         let jump_threshold = effective_rows * 3;
404.         let distance = cursor.row.saturating_sub(self.offset_row);
405. 
406.         if distance > jump_threshold {
407.             // 將 offset_row 設置為讓光標位於螢幕中間偏上的位置
408.             // 這樣用戶可以看到光標上下文，體驗更好
409.             self.offset_row = cursor.row.saturating_sub(effective_rows / 3);
410.             self.invalidate_cache();
411.             return;
412.         }
413. 
414.         // 計算目前 offset_row ~ cursor.row 的視覺高度
415.         let mut visual_offset = 0;
416.         let available_width = self.get_available_width(buffer);
417. 
418.         for row in self.offset_row..=cursor.row {
419.             let cache_index = row.saturating_sub(self.offset_row);
420.             if let Some(Some(layout)) = self.line_layout_cache.get(cache_index) {
421.                 visual_offset += layout.visual_height;
422.             } else if let Some(layout) = LineLayout::new(buffer, row, available_width) {
423.                 visual_offset += layout.visual_height;
424.                 if cache_index < self.line_layout_cache.len() {
425.                     self.line_layout_cache[cache_index] = Some(layout);
426.                 }
427.             }
428.         }
429. 
430.         // 如果沒超出螢幕，就不用動
431.         if visual_offset < effective_rows {
432.             return;
433.         }
434. 
435.         // 向下推 offset_row，每次扣掉最上面那一行的視覺高度
436.         while self.offset_row < cursor.row && visual_offset >= effective_rows {
437.             let top_layout_opt = self
438.                 .line_layout_cache
439.                 .first()
440.                 .and_then(|l| l.as_ref())
441.                 .cloned();
442. 
443.             if let Some(layout) = top_layout_opt {
444.                 visual_offset = visual_offset.saturating_sub(layout.visual_height);
445.             } else if let Some(layout) = LineLayout::new(buffer, self.offset_row, available_width) {
446.                 visual_offset = visual_offset.saturating_sub(layout.visual_height);
447.                 if !self.line_layout_cache.is_empty() {
448.                     self.line_layout_cache[0] = Some(layout);
449.                 }
450.             }
451. 
452.             self.offset_row += 1;
453. 
454.             if !self.line_layout_cache.is_empty() {
455.                 self.line_layout_cache.remove(0);
456.                 self.line_layout_cache.push(None);
457.             }
458.         }
459.     }
460. 
461.     fn render_status_bar(
462.         &self,
463.         buffer: &RopeBuffer,
464.         selection_mode: bool,
465.         message: Option<&str>,
466.         cursor: &Cursor,
467.     ) -> Result<()> {
468.         let mut stdout = io::stdout();
469.         queue!(stdout, cursor::MoveTo(0, self.screen_rows as u16))?;
470. 
471.         queue!(stdout, style::SetBackgroundColor(Color::DarkGrey))?;
472.         queue!(stdout, style::SetForegroundColor(Color::White))?;
473. 
474.         let modified = if buffer.is_modified() {
475.             " [modified]"
476.         } else {
477.             ""
478.         };
479.         let filename = buffer.file_name();
480. 
481.         let mode_indicator = if selection_mode {
482.             " [Selection Mode]"
483.         } else {
484.             ""
485.         };
486. 
487.         let status = if let Some(msg) = message {
488.             format!(" {}{}{}  - {}", filename, modified, mode_indicator, msg)
489.         } else {
490.             format!(
491.                 " {}{}{}  Line {}/{}  Ctrl+W:Save Ctrl+Q:Quit",
492.                 filename,
493.                 modified,
494.                 mode_indicator,
495.                 cursor.row + 1,
496.                 buffer.line_count()
497.             )
498.         };
499. 
500.         // 確保狀態欄填滿整行（使用視覺寬度）
501.         let status = if visual_width(&status) < self.screen_cols {
502.             format!("{:width$}", status, width = self.screen_cols)
503.         } else {
504.             let mut result = String::new();
505.             let mut current_width = 0;
506.             for ch in status.chars() {
507.                 let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
508.                 if current_width + ch_width > self.screen_cols {
509.                     break;
510.                 }
511.                 result.push(ch);
512.                 current_width += ch_width;
513.             }
514.             result
515.         };
516. 
517.         queue!(stdout, style::Print(status))?;
518.         queue!(stdout, style::ResetColor)?;
519. 
520.         Ok(())
521.     }
522. 
523.     pub fn toggle_line_numbers(&mut self) {
524.         self.show_line_numbers = !self.show_line_numbers;
525.     }
526. 
527.     /// 計算行號寬度（包含右側空格）
528.     fn calculate_line_number_width(&self, buffer: &RopeBuffer) -> usize {
529.         if self.show_line_numbers {
530.             buffer.line_count().to_string().len() + 1
531.         } else {
532.             0
533.         }
534.     }
535. 
536.     /// 獲取可用於顯示內容的寬度（扣除行號寬度）
537.     pub fn get_available_width(&self, buffer: &RopeBuffer) -> usize {
538.         let line_num_width = self.calculate_line_number_width(buffer);
539.         self.screen_cols
540.             .saturating_sub(line_num_width)
541.             .saturating_sub(1)
542.     }
543. 
544.     /// 計算指定邏輯行的視覺行分割（給其他模組用，不依賴 cache 也可以）
545.     pub fn calculate_visual_lines_for_row(&self, buffer: &RopeBuffer, row: usize) -> Vec<String> {
546.         if row >= buffer.line_count() {
547.             return vec![String::new()];
548.         }
549. 
550.         // 如果 row 剛好在快取範圍內，優先使用快取
551.         let cache_index = row.saturating_sub(self.offset_row);
552.         if let Some(Some(layout)) = self.line_layout_cache.get(cache_index) {
553.             return layout.visual_lines.clone();
554.         }
555. 
556.         let available_width = self.get_available_width(buffer);
557.         let line = buffer.line(row).map(|s| s.to_string()).unwrap_or_default();
558.         let mut line = line;
559.         while matches!(line.chars().last(), Some('\n' | '\r')) {
560.             line.pop();
561.         }
562. 
563.         let (displayed_line, _) = expand_tabs_and_build_map(&line);
564.         wrap_line(&displayed_line, available_width)
565.     }
566. 
567.     /// 將邏輯列轉換為視覺列（考慮 Tab 展開和字符寬度）
568.     pub fn logical_col_to_visual_col(&self, line: &str, logical_col: usize) -> usize {
569.         // 這個函式目前只拿到一行字串，不知道 row，無法用 cache。
570.         // 保留原來的行為：直接掃一遍。
571.         let mut visual_col = 0;
572.         for (idx, ch) in line.chars().enumerate() {
573.             if idx >= logical_col {
574.                 break;
575.             }
576.             if ch == '\t' {
577.                 visual_col += TAB_WIDTH;
578.             } else {
579.                 visual_col += UnicodeWidthChar::width(ch).unwrap_or(1);
580.             }
581.         }
582.         visual_col
583.     }
584. 
585.     /// 從視覺行索引和視覺列轉換為邏輯列
586.     pub fn visual_to_logical_col(
587.         &self,
588.         buffer: &RopeBuffer,
589.         row: usize,
590.         visual_line_index: usize,
591.         visual_col: usize,
592.     ) -> usize {
593.         // 優先使用快取（如果該行目前在視窗 cache 內）
594.         let cache_index = row.saturating_sub(self.offset_row);
595.         if let Some(Some(layout)) = self.line_layout_cache.get(cache_index) {
596.             if visual_line_index >= layout.visual_lines.len() {
597.                 return 0;
598.             }
599. 
600.             // 計算前面視覺行的總視覺寬度
601.             let mut accumulated_width = 0;
602.             for line in layout.visual_lines.iter().take(visual_line_index) {
603.                 accumulated_width += visual_width(line);
604.             }
605. 
606.             // 加上當前視覺行內的列位置
607.             let col_in_visual =
608.                 visual_col.min(visual_width(&layout.visual_lines[visual_line_index]));
609.             let visual_col_total = accumulated_width + col_in_visual;
610. 
611.             // 在 logical_to_visual 中尋找「視覺座標 >= visual_col_total」的最小 logical_col
612.             let mut logical_col = 0;
613.             for (idx, &vcol) in layout.logical_to_visual.iter().enumerate() {
614.                 if vcol > visual_col_total {
615.                     break;
616.                 }
617.                 logical_col = idx;
618.             }
619.             return logical_col;
620.         }
621. 
622.         // 若不在 cache 範圍，退回原本的計算方式（慢但安全）
623.         let visual_lines = self.calculate_visual_lines_for_row(buffer, row);
624. 
625.         if visual_line_index >= visual_lines.len() {
626.             return 0;
627.         }
628. 
629.         // 計算前面視覺行的總視覺寬度
630.         let mut accumulated_width = 0;
631.         for line in visual_lines.iter().take(visual_line_index) {
632.             accumulated_width += visual_width(line);
633.         }
634. 
635.         let col_in_visual = visual_col.min(visual_width(&visual_lines[visual_line_index]));
636.         let visual_col_total = accumulated_width + col_in_visual;
637. 
638.         if let Some(line) = buffer.line(row) {
639.             let mut line_str = line.to_string();
640.             while matches!(line_str.chars().last(), Some('\n' | '\r')) {
641.                 line_str.pop();
642.             }
643. 
644.             let mut logical_col = 0;
645.             let mut current_visual = 0;
646. 
647.             for ch in line_str.chars() {
648.                 if current_visual >= visual_col_total {
649.                     break;
650.                 }
651. 
652.                 if ch == '\t' {
653.                     current_visual += TAB_WIDTH;
654.                 } else {
655.                     current_visual += UnicodeWidthChar::width(ch).unwrap_or(1);
656.                 }
657. 
658.                 logical_col += 1;
659.             }
660. 
661.             logical_col
662.         } else {
663.             0
664.         }
665.     }
666. 
667.     /// 實際可用於顯示文本的螢幕行數（扣除 debug 標尺）
668.     pub fn get_effective_screen_rows(&self, has_debug_ruler: bool) -> usize {
669.         if has_debug_ruler {
670.             self.screen_rows.saturating_sub(1)
671.         } else {
672.             self.screen_rows
673.         }
674.     }
675. 
676.     /// 獲取cursor的視覺位置（螢幕座標）
677.     pub fn get_cursor_visual_position(
678.         &self,
679.         cursor: &Cursor,
680.         buffer: &RopeBuffer,
681.     ) -> (usize, usize) {
682.         let line_num_width = self.calculate_line_number_width(buffer);
683. 
684.         // 計算cursor所在的螢幕行
685.         let mut screen_y = 0;
686.         let mut file_row = self.offset_row;
687. 
688.         while file_row < cursor.row && screen_y < self.screen_rows {
689.             let cache_index = file_row.saturating_sub(self.offset_row);
690.             let layout_opt = self
691.                 .line_layout_cache
692.                 .get(cache_index)
693.                 .and_then(|l| l.as_ref())
694.                 .cloned();
695. 
696.             let layout = if let Some(layout) = layout_opt {
697.                 layout
698.             } else {
699.                 LineLayout::new(buffer, file_row, self.get_available_width(buffer)).unwrap_or_else(
700.                     || LineLayout {
701.                         visual_lines: vec![String::new()],
702.                         visual_height: 1,
703.                         logical_to_visual: vec![0],
704.                     },
705.                 )
706.             };
707. 
708.             screen_y += layout.visual_height;
709.             file_row += 1;
710.         }
711. 
712.         // 添加cursor行內的視覺行偏移
713.         screen_y += cursor.visual_line_index;
714. 
715.         // 如果超出螢幕，返回最後一行
716.         let screen_y = screen_y.min(self.screen_rows.saturating_sub(1));
717. 
718.         // 計算cursor在視覺行內的x位置
719.         let visual_lines = self.calculate_visual_lines_for_row(buffer, cursor.row);
720.         let mut screen_x = line_num_width;
721. 
722.         if cursor.visual_line_index < visual_lines.len() {
723.             // 計算前面視覺行的累計寬度
724.             let mut accumulated_width = 0;
725.             for line in visual_lines.iter().take(cursor.visual_line_index) {
726.                 accumulated_width += visual_width(line);
727.             }
728. 
729.             // cursor在整個邏輯行中的視覺col
730.             let line_str = buffer
731.                 .line(cursor.row)
732.                 .map(|s| s.to_string())
733.                 .unwrap_or_default();
734.             let line_str = line_str.trim_end_matches(['\n', '\r']);
735.             let cursor_visual_col = self.logical_col_to_visual_col(line_str, cursor.col);
736. 
737.             // 在當前視覺行內的col
738.             let visual_col_in_line = cursor_visual_col.saturating_sub(accumulated_width);
739. 
740.             // 加上行號寬度
741.             screen_x += visual_col_in_line;
742.         }
743. 
744.         (screen_x, screen_y)
745.     }
746. 
747.     /// 渲染列標尺（顯示列位置個位數字）
748.     fn render_column_ruler(&self, stdout: &mut io::Stdout, buffer: &RopeBuffer) -> Result<()> {
749.         queue!(stdout, cursor::MoveTo(0, 0))?;
750.         queue!(stdout, style::SetForegroundColor(Color::DarkGrey))?;
751. 
752.         let line_num_width = self.calculate_line_number_width(buffer);
753. 
754.         for _ in 0..line_num_width {
755.             queue!(stdout, style::Print(" "))?;
756.         }
757. 
758.         let available_cols = self
759.             .screen_cols
760.             .saturating_sub(line_num_width)
761.             .saturating_sub(1);
762.         for col in 0..available_cols {
763.             let digit = col % 10;
764.             queue!(stdout, style::Print(digit))?;
765.         }
766. 
767.         queue!(stdout, style::ResetColor)?;
768.         Ok(())
769.     }
770. }
771. 
772. /// 將行按可用寬度切分成多個視覺行（共用）
773. fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
774.     if max_width == 0 {
775.         return vec![String::new()];
776.     }
777. 
778.     let mut result = Vec::new();
779.     let mut current_line = String::new();
780.     let mut current_width = 0;
781. 
782.     for ch in line.chars() {
783.         let char_width = UnicodeWidthChar::width(ch).unwrap_or(1);
784. 
785.         if current_width + char_width > max_width && !current_line.is_empty() {
786.             result.push(current_line);
787.             current_line = String::new();
788.             current_width = 0;
789.         }
790. 
791.         current_line.push(ch);
792.         current_width += char_width;
793.     }
794. 
795.     if !current_line.is_empty() {
796.         result.push(current_line);
797.     }
798. 
799.     if result.is_empty() {
800.         result.push(String::new());
801.     }
802. 
803.     result
804. }
805.
```

</details>


---

<sub>⏱️ 6m 44s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi\src\utils\line_wrapper.rs.**

D:\Users\user\Documents\rust\wedi\src\utils\line_wrapper.rs

<details>
<summary>50 lines</summary>

```
1. // 自動換行邏輯
2. // 這個模組將在後續階段實現
3. 
4. #[allow(dead_code)]
5. pub struct LineWrapper {
6.     max_width: usize,
7. }
8. 
9. impl LineWrapper {
10.     #[allow(dead_code)]
11.     pub fn new(max_width: usize) -> Self {
12.         Self { max_width }
13.     }
14. 
15.     #[allow(dead_code)]
16.     pub fn wrap_line(&self, line: &str) -> Vec<String> {
17.         if line.len() <= self.max_width {
18.             return vec![line.to_string()];
19.         }
20. 
21.         let mut wrapped = Vec::new();
22.         let mut current = String::new();
23.         let mut current_width = 0;
24. 
25.         for ch in line.chars() {
26.             let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
27. 
28.             if current_width + char_width > self.max_width {
29.                 wrapped.push(current.clone());
30.                 current.clear();
31.                 current_width = 0;
32.             }
33. 
34.             current.push(ch);
35.             current_width += char_width;
36.         }
37. 
38.         if !current.is_empty() {
39.             wrapped.push(current);
40.         }
41. 
42.         wrapped
43.     }
44. 
45.     #[allow(dead_code)]
46.     pub fn set_max_width(&mut self, width: usize) {
47.         self.max_width = width;
48.     }
49. }
50.
```

</details>


---

<sub>⏱️ 6m 44s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi\src\input\handler.rs.**

D:\Users\user\Documents\rust\wedi\src\input\handler.rs

<details>
<summary>106 lines</summary>

```
1. #[allow(dead_code)]
2. #[derive(Debug, Clone, Copy, PartialEq, Eq)]
3. pub enum Direction {
4.     Up,
5.     Down,
6.     Left,
7.     Right,
8.     Home,
9.     End,
10.     PageUp,
11.     PageDown,
12.     TenthUp,   // 跳躍 1/10 文件向上
13.     TenthDown, // 跳躍 1/10 文件向下
14.     FileStart,
15.     FileEnd,
16. }
17. 
18. #[allow(dead_code)]
19. #[derive(Debug, Clone, PartialEq, Eq)]
20. pub enum Command {
21.     // 字符輸入
22.     Insert(char),
23. 
24.     // 刪除操作
25.     Delete,
26.     Backspace,
27.     DeleteLine,
28. 
29.     // 光標移動
30.     MoveUp,
31.     MoveDown,
32.     MoveLeft,
33.     MoveRight,
34.     MoveHome, // Home： 跳到行首
35.     MoveEnd,  // End： 跳到行尾
36.     PageUp,
37.     PageDown,
38.     MoveToFileStart, // Ctrl+Up: 跳到第一行
39.     MoveToFileEnd,   // Ctrl+Down: 跳到最後一行
40.     // MoveToLineStart, // Ctrl+Left: 跳到行首
41.     // MoveToLineEnd,   // Ctrl+Right: 跳到行尾
42. 
43.     // 剪貼板操作
44.     Copy,
45.     Cut,
46.     Paste,
47.     CopyInternal,  // 使用內部剪貼簿複製
48.     CutInternal,   // 使用內部剪貼簿剪切
49.     PasteInternal, // 使用內部剪貼簿貼上
50. 
51.     // 視窗調整
52.     Resize,
53. 
54.     // 文件操作
55.     Save,
56.     Quit,
57. 
58.     // 撤銷/重做
59.     Undo,
60.     Redo,
61. 
62.     // 搜索
63.     Find,
64.     FindNext,
65.     FindPrev,
66. 
67.     // 視圖控制
68.     ToggleLineNumbers,
69. 
70.     // 註解切換
71.     ToggleComment,
72. 
73.     // 縮排操作
74.     Indent,
75.     Unindent,
76. 
77.     // 選擇操作
78.     SelectAll,
79.     ExtendSelection(Direction),
80.     #[allow(dead_code)]
81.     ClearSelection,
82. 
83.     // 跳轉
84.     GoToLine,
85. 
86.     // 清除訊息
87.     ClearMessage,
88. 
89.     // 選擇模式切換
90.     ToggleSelectionMode,
91. 
92.     // 編碼切換
93.     ChangeEncoding,
94. 
95.     // 文件 1/10 跳躍
96.     JumpTenthUp,
97.     JumpTenthDown,
98. 
99.     // 語法高亮模式切換
100.     #[cfg(feature = "syntax-highlighting")]
101.     ToggleSyntaxHighlight,
102. 
103.     // 顯示幫助
104.     ShowHelp,
105. }
106.
```

</details>


---

<sub>⏱️ 6m 44s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi\src\cursor.rs.**

D:\Users\user\Documents\rust\wedi\src\cursor.rs

<details>
<summary>245 lines</summary>

```
1. use crate::buffer::RopeBuffer;
2. use crate::utils::visual_width;
3. use crate::view::View;
4. 
5. #[derive(Debug, Clone, Copy)]
6. pub struct Cursor {
7.     pub row: usize,                // 邏輯行號 (0-based)
8.     pub col: usize,                // 邏輯列號 (0-based)
9.     pub visual_line_index: usize,  // 在當前邏輯行的第幾個視覺行 (0-based)
10.     pub desired_visual_col: usize, // 期望的視覺列位置（用於上下移動）
11. }
12. 
13. impl Cursor {
14.     pub fn new() -> Self {
15.         Self {
16.             row: 0,
17.             col: 0,
18.             visual_line_index: 0,
19.             desired_visual_col: 0,
20.         }
21.     }
22. 
23.     pub fn move_up(&mut self, buffer: &RopeBuffer, view: &View) {
24.         if self.visual_line_index > 0 {
25.             // 在同一邏輯行內向上移動到上一個視覺行
26.             self.visual_line_index -= 1;
27.             self.update_logical_col_from_visual(buffer, view);
28.         } else {
29.             // 移動到上一個邏輯行
30.             if self.row > 0 {
31.                 self.row -= 1;
32.                 // 移動到該邏輯行的最後一個視覺行
33.                 let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
34.                 self.visual_line_index = visual_lines.len().saturating_sub(1);
35.                 self.update_logical_col_from_visual(buffer, view);
36.             }
37.         }
38.     }
39. 
40.     pub fn move_down(&mut self, buffer: &RopeBuffer, view: &View) {
41.         let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
42. 
43.         if self.visual_line_index + 1 < visual_lines.len() {
44.             // 在同一邏輯行內向下移動到下一個視覺行
45.             self.visual_line_index += 1;
46.             self.update_logical_col_from_visual(buffer, view);
47.         } else {
48.             // 移動到下一個邏輯行
49.             if self.row + 1 < buffer.line_count() {
50.                 self.row += 1;
51.                 self.visual_line_index = 0;
52.                 self.update_logical_col_from_visual(buffer, view);
53.             }
54.         }
55.     }
56. 
57.     pub fn move_left(&mut self, buffer: &RopeBuffer, view: &View) {
58.         if self.col > 0 {
59.             self.col -= 1;
60.             self.update_visual_from_logical(buffer, view);
61.         } else if self.row > 0 {
62.             // 移動到上一行末尾
63.             self.row -= 1;
64.             self.col = self.line_len(buffer, self.row);
65.             self.update_visual_from_logical(buffer, view);
66.         }
67.         self.sync_desired_visual_col(buffer, view);
68.     }
69. 
70.     pub fn move_right(&mut self, buffer: &RopeBuffer, view: &View) {
71.         let line_len = self.line_len(buffer, self.row);
72.         if self.col < line_len {
73.             self.col += 1;
74.             self.update_visual_from_logical(buffer, view);
75.         } else if self.row + 1 < buffer.line_count() {
76.             // 移動到下一行開頭
77.             self.row += 1;
78.             self.col = 0;
79.             self.visual_line_index = 0;
80.             self.desired_visual_col = 0;
81.         }
82.         self.sync_desired_visual_col(buffer, view);
83.     }
84. 
85.     pub fn move_to_line_start(&mut self) {
86.         self.col = 0;
87.         self.visual_line_index = 0;
88.         self.desired_visual_col = 0;
89.     }
90. 
91.     pub fn move_to_line_end(&mut self, buffer: &RopeBuffer, view: &View) {
92.         self.col = self.line_len(buffer, self.row);
93.         self.update_visual_from_logical(buffer, view);
94.         self.sync_desired_visual_col(buffer, view);
95.     }
96. 
97.     /// 移動到文件開頭
98.     pub fn move_to_file_start(&mut self, _view: &View) {
99.         // 設置到第一行行首，視覺狀態使用預設值
100.         self.row = 0;
101.         self.col = 0;
102.         self.visual_line_index = 0;
103.         self.desired_visual_col = 0;
104.     }
105. 
106.     /// 移動到文件末尾
107.     pub fn move_to_file_end(&mut self, buffer: &RopeBuffer, view: &View) {
108.         if buffer.line_count() > 0 {
109.             self.row = buffer.line_count() - 1;
110.             // 移動到最後一行行尾，並同步視覺狀態
111.             self.move_to_line_end(buffer, view);
112.         }
113.     }
114. 
115.     pub fn move_page_up(&mut self, buffer: &RopeBuffer, view: &View, effective_rows: usize) {
116.         let mut target_row = self.row;
117.         let mut visual_count = 0;
118. 
119.         // 向上累積視覺行直到達到約一個螢幕
120.         while target_row > 0 && visual_count < effective_rows {
121.             target_row -= 1;
122.             let vlines = view.calculate_visual_lines_for_row(buffer, target_row);
123.             visual_count += vlines.len();
124.         }
125. 
126.         self.row = target_row;
127.         self.visual_line_index = 0;
128.         self.update_logical_col_from_visual(buffer, view);
129.     }
130. 
131.     pub fn move_page_down(&mut self, buffer: &RopeBuffer, view: &View, effective_rows: usize) {
132.         let max_row = buffer.line_count().saturating_sub(1);
133.         let mut target_row = self.row;
134.         let mut visual_count = 0;
135. 
136.         // 向下累積視覺行直到達到約一個螢幕
137.         while target_row < max_row && visual_count < effective_rows {
138.             let vlines = view.calculate_visual_lines_for_row(buffer, target_row);
139.             visual_count += vlines.len();
140.             target_row += 1;
141.         }
142. 
143.         self.row = target_row.min(max_row);
144.         self.visual_line_index = 0;
145.         self.update_logical_col_from_visual(buffer, view);
146.     }
147. 
148.     #[allow(dead_code)]
149.     pub fn move_to_line(&mut self, buffer: &RopeBuffer, view: &View, line: usize) {
150.         self.row = line.min(buffer.line_count().saturating_sub(1));
151.         self.visual_line_index = 0;
152.         self.update_logical_col_from_visual(buffer, view);
153.     }
154. 
155.     /// 獲取光標在文本中的絕對字符位置
156.     pub fn char_position(&self, buffer: &RopeBuffer) -> usize {
157.         buffer.line_to_char(self.row) + self.col
158.     }
159. 
160.     /// 設置光標位置並同步視覺狀態
161.     /// 這是統一的光標位置設置方法，確保邏輯和視覺狀態一致
162.     pub fn set_position(&mut self, buffer: &RopeBuffer, view: &View, row: usize, col: usize) {
163.         self.row = row;
164.         self.col = col;
165.         self.update_visual_from_logical(buffer, view);
166.         self.sync_desired_visual_col(buffer, view);
167.     }
168. 
169.     /// 重置到行首（用於換行等操作）
170.     pub fn reset_to_line_start(&mut self) {
171.         self.col = 0;
172.         self.visual_line_index = 0;
173.         self.desired_visual_col = 0;
174.     }
175. 
176.     /// 從視覺座標更新邏輯列位置
177.     fn update_logical_col_from_visual(&mut self, buffer: &RopeBuffer, view: &View) {
178.         let visual_col = self.desired_visual_col;
179.         self.col = view.visual_to_logical_col(buffer, self.row, self.visual_line_index, visual_col);
180. 
181.         // 確保不超出行長度
182.         let line_len = self.line_len(buffer, self.row);
183.         self.col = self.col.min(line_len);
184.     }
185. 
186.     /// 從邏輯座標更新視覺座標
187.     fn update_visual_from_logical(&mut self, buffer: &RopeBuffer, view: &View) {
188.         let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
189. 
190.         if let Some(line) = buffer.line(self.row) {
191.             let line_str = line.to_string();
192.             let visual_col = view.logical_col_to_visual_col(&line_str, self.col);
193. 
194.             // 找出光標在哪個視覺行
195.             let mut accumulated = 0;
196.             for (idx, vline) in visual_lines.iter().enumerate() {
197.                 let vline_len = visual_width(vline);
198.                 if visual_col < accumulated + vline_len || idx == visual_lines.len() - 1 {
199.                     self.visual_line_index = idx;
200.                     break;
201.                 }
202.                 accumulated += vline_len;
203.             }
204.         } else {
205.             self.visual_line_index = 0;
206.         }
207.     }
208. 
209.     /// 同步期望視覺列位置
210.     fn sync_desired_visual_col(&mut self, buffer: &RopeBuffer, view: &View) {
211.         if let Some(line) = buffer.line(self.row) {
212.             let line_str = line.to_string();
213.             let visual_col = view.logical_col_to_visual_col(&line_str, self.col);
214. 
215.             // 計算在當前視覺行內的列位置
216.             let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
217.             let mut accumulated = 0;
218.             for i in 0..self.visual_line_index {
219.                 if i < visual_lines.len() {
220.                     accumulated += visual_width(&visual_lines[i]);
221.                 }
222.             }
223. 
224.             self.desired_visual_col = visual_col - accumulated;
225.         }
226.     }
227. 
228.     /// 獲取指定行的長度（不包含換行符）
229.     fn line_len(&self, buffer: &RopeBuffer, row: usize) -> usize {
230.         if let Some(line) = buffer.line(row) {
231.             let text = line.to_string();
232.             let text = text.trim_end_matches(['\n', '\r']);
233.             text.chars().count()
234.         } else {
235.             0
236.         }
237.     }
238. }
239. 
240. impl Default for Cursor {
241.     fn default() -> Self {
242.         Self::new()
243.     }
244. }
245.
```

</details>


---

<sub>⏱️ 6m 51s</sub>

### ✅ `grep`

"selection|Selection" (D:\Users\user\Documents\rust\wedi\src)

<details>
<summary>6 matches</summary>

```
D:\Users\user\Documents\rust\wedi\src\clipboard.rs
D:\Users\user\Documents\rust\wedi\src\editor.rs
D:\Users\user\Documents\rust\wedi\src\help.rs
D:\Users\user\Documents\rust\wedi\src\view.rs
D:\Users\user\Documents\rust\wedi\src\input\handler.rs
D:\Users\user\Documents\rust\wedi\src\input\keymap.rs
```

</details>


---

<sub>⏱️ 6m 51s</sub>

### ✅ `view`

**view the file at D:\Users\user\Documents\rust\wedi\src\editor.rs.**

D:\Users\user\Documents\rust\wedi\src\editor.rs

<details>
<summary>1435 lines</summary>

```
1. use crate::buffer::{EncodingConfig, RopeBuffer};
2. use crate::clipboard::ClipboardManager;
3. use crate::comment::CommentHandler;
4. use crate::cursor::Cursor;
5. use crate::input::{handle_key_event, Command, Direction};
6. use crate::search::Search;
7. use crate::terminal::Terminal;
8. use crate::utils::visual_width;
9. use crate::view::{Selection, View};
10. use anyhow::Result;
11. use std::path::Path;
12. 
13. #[cfg(feature = "syntax-highlighting")]
14. use crate::highlight::{HighlightCache, HighlightConfig, HighlightEngine};
15. 
16. pub struct Editor {
17.     buffer: RopeBuffer,
18.     cursor: Cursor,
19.     view: View,
20.     terminal: Terminal,
21.     clipboard: ClipboardManager,
22.     internal_clipboard: String, // 內部剪貼簿作為後備
23.     search: Search,
24.     search_mode: bool, // 搜尋模式開關（Ctrl+F 開啟，ESC 關閉）
25.     comment_handler: CommentHandler,
26.     should_quit: bool,
27.     selection: Option<Selection>,
28.     selection_mode: bool, // F1 選擇模式開關
29.     message: Option<String>,
30.     quit_times: u8, // 追蹤連續按 Ctrl+Q 的次數
31.     debug_mode: bool,
32. 
33.     // 語法高亮（可選功能）
34.     #[cfg(feature = "syntax-highlighting")]
35.     pub(crate) highlight_engine: Option<HighlightEngine>,
36.     #[cfg(feature = "syntax-highlighting")]
37.     pub(crate) highlight_cache: HighlightCache,
38.     #[cfg(feature = "syntax-highlighting")]
39.     #[allow(dead_code)]
40.     highlight_config: HighlightConfig,
41.     #[cfg(feature = "syntax-highlighting")]
42.     highlight_enabled: bool,
43. }
44. 
45. impl Editor {
46.     pub fn new(
47.         file_path: Option<&Path>,
48.         debug_mode: bool,
49.         encoding_config: &EncodingConfig,
50.         #[cfg(feature = "syntax-highlighting")] theme: Option<&str>,
51.     ) -> Result<Self> {
52.         let buffer = if let Some(path) = file_path {
53.             // 使用新的方法，支持指定編碼
54.             RopeBuffer::from_file_with_encoding(path, encoding_config)?
55.         } else {
56.             let mut buffer = RopeBuffer::new();
57.             // 如果指定了讀取編碼，設置編碼
58.             if let Some(enc) = encoding_config.read_encoding {
59.                 if cfg!(debug_assertions) {
60.                     eprintln!(
61.                         "[DEBUG] Editor::new() - Setting read_encoding from config: {}",
62.                         enc.name()
63.                     );
64.                 }
65.                 buffer.set_read_encoding(enc);
66.             }
67.             // 如果指定了存檔編碼，設置存檔編碼
68.             if let Some(enc) = encoding_config.save_encoding {
69.                 if cfg!(debug_assertions) {
70.                     eprintln!(
71.                         "[DEBUG] Editor::new() - Setting save_encoding from config: {}",
72.                         enc.name()
73.                     );
74.                 }
75.                 buffer.set_save_encoding(enc);
76.             }
77. 
78.             if cfg!(debug_assertions) {
79.                 eprintln!(
80.                     "[DEBUG] Editor::new() - Final buffer save_encoding: {}",
81.                     if let Some(enc) = encoding_config.save_encoding {
82.                         enc.name()
83.                     } else if let Some(enc) = encoding_config.read_encoding {
84.                         enc.name()
85.                     } else {
86.                         "system default"
87.                     }
88.                 );
89.             }
90. 
91.             buffer
92.         };
93. 
94.         let terminal = Terminal::new()?;
95.         let view = View::new(&terminal);
96.         let clipboard = ClipboardManager::new()?;
97. 
98.         let mut comment_handler = CommentHandler::new();
99.         if let Some(path) = file_path {
100.             comment_handler.detect_from_path(path);
101.         }
102. 
103.         // 語法高亮初始化
104.         #[cfg(feature = "syntax-highlighting")]
105.         let (highlight_engine, highlight_cache, highlight_config) = {
106.             let mut config = HighlightConfig::default();
107. 
108.             // 如果提供了自定義主題，使用它；否則使用默認主題
109.             if let Some(custom_theme) = theme {
110.                 config.theme = custom_theme.to_string();
111.             }
112. 
113.             let mut engine = if config.enabled {
114.                 HighlightEngine::new(Some(&config.theme), config.true_color).ok()
115.             } else {
116.                 None
117.             };
118. 
119.             // 如果有檔案，設定語法類型
120.             if let (Some(path), Some(ref mut eng)) = (file_path, engine.as_mut()) {
121.                 eng.set_file(Some(path));
122.             }
123. 
124.             (engine, HighlightCache::new(), config)
125.         };
126. 
127.         Ok(Self {
128.             buffer,
129.             cursor: Cursor::new(),
130.             view,
131.             terminal,
132.             clipboard,
133.             internal_clipboard: String::new(), // 初始化內部剪貼簿
134.             search: Search::new(),
135.             search_mode: false, // 預設關閉搜尋模式
136.             comment_handler,
137.             should_quit: false,
138.             selection: None,
139.             selection_mode: false, // 預設關閉選擇模式
140.             message: None,
141.             quit_times: 0,
142.             debug_mode,
143. 
144.             #[cfg(feature = "syntax-highlighting")]
145.             highlight_engine,
146.             #[cfg(feature = "syntax-highlighting")]
147.             highlight_cache,
148.             #[cfg(feature = "syntax-highlighting")]
149.             highlight_config,
150.             #[cfg(feature = "syntax-highlighting")]
151.             highlight_enabled: true, // 預設啟用語法高亮
152.         })
153.     }
154. 
155.     pub fn run(&mut self) -> Result<()> {
156.         Terminal::enter_raw_mode()?;
157.         Terminal::clear_screen()?;
158. 
159.         while !self.should_quit {
160.             let debug_info = if self.debug_mode {
161.                 Some(self.get_debug_info())
162.             } else {
163.                 None
164.             };
165. 
166.             // ⚠️ 重要：在計算高亮之前先更新 offset_row
167.             // 避免跳頁後 highlighted_lines 使用舊的 offset_row
168.             let has_debug_ruler = self.debug_mode;
169.             self.view
170.                 .scroll_if_needed(&self.cursor, &self.buffer, has_debug_ruler);
171. 
172.             // 獲取語法高亮行
173.             #[cfg(feature = "syntax-highlighting")]
174.             let highlighted_lines = {
175.                 if self.highlight_enabled {
176.                     let start_row = self.view.offset_row;
177.                     let end_row = start_row + self.view.screen_rows;
178.                     self.get_highlighted_lines(start_row, end_row)
179.                 } else {
180.                     std::collections::HashMap::new()
181.                 }
182.             };
183. 
184.             self.view.render(
185.                 &self.buffer,
186.                 &self.cursor,
187.                 self.selection.as_ref(),
188.                 if self.debug_mode {
189.                     debug_info.as_deref()
190.                 } else {
191.                     self.message.as_deref()
192.                 },
193.                 #[cfg(feature = "syntax-highlighting")]
194.                 Some(&highlighted_lines),
195.             )?;
196. 
197.             let key_event = Terminal::read_key()?;
198. 
199.             if let Some(command) = handle_key_event(key_event, self.selection_mode) {
200.                 self.handle_command(command)?;
201.             }
202.         }
203. 
204.         Terminal::exit_raw_mode()?;
205.         Ok(())
206.     }
207. 
208.     fn handle_command(&mut self, command: Command) -> Result<()> {
209.         // 任何非 Quit 的命令都重置 quit_times
210.         if !matches!(command, Command::Quit) {
211.             self.quit_times = 0;
212.         }
213. 
214.         match command {
215.             // 字符輸入
216.             Command::Insert(ch) => {
217.                 if self.has_selection() {
218.                     self.delete_selection();
219.                 }
220. 
221.                 let pos = self.cursor.char_position(&self.buffer);
222.                 self.buffer.insert_char(pos, ch);
223. 
224.                 // 優化：僅失效當前行（除非是換行符，需要重建整個緩存）
225.                 if ch == '\n' {
226.                     self.view.invalidate_cache(); // 換行影響多行佈局
227.                     #[cfg(feature = "syntax-highlighting")]
228.                     self.highlight_cache.clear(); // 語法高亮快取也需要清除
229.                     self.cursor.row += 1;
230.                     self.cursor.reset_to_line_start();
231.                 } else {
232.                     self.view.invalidate_line(self.cursor.row); // 僅失效當前行
233.                     #[cfg(feature = "syntax-highlighting")]
234.                     self.invalidate_highlight_cache(self.cursor.row); // 語法高亮快取失效
235.                     self.cursor.set_position(
236.                         &self.buffer,
237.                         &self.view,
238.                         self.cursor.row,
239.                         self.cursor.col + 1,
240.                     );
241.                 }
242. 
243.                 self.selection = None;
244.                 self.selection_mode = false; // 輸入後關閉選擇模式
245.             }
246. 
247.             // 刪除操作
248.             Command::Backspace => {
249.                 if self.has_selection() {
250.                     self.delete_selection();
251.                 } else if self.cursor.col > 0 {
252.                     // 行內刪除
253.                     let new_col = self.cursor.col - 1;
254.                     let pos = self.buffer.line_to_char(self.cursor.row) + new_col;
255.                     self.buffer.delete_char(pos);
256.                     self.view.invalidate_line(self.cursor.row); // 僅失效當前行
257.                     #[cfg(feature = "syntax-highlighting")]
258.                     self.invalidate_highlight_cache(self.cursor.row);
259.                     self.cursor
260.                         .set_position(&self.buffer, &self.view, self.cursor.row, new_col);
261.                 } else if self.cursor.row > 0 {
262.                     // 刪除換行符，合併到上一行
263.                     let new_row = self.cursor.row - 1;
264.                     let prev_line_len = self
265.                         .buffer
266.                         .get_line_content(new_row)
267.                         .trim_end_matches(['\n', '\r'])
268.                         .chars()
269.                         .count();
270. 
271.                     let pos = self.buffer.line_to_char(new_row) + prev_line_len;
272.                     self.buffer.delete_char(pos);
273.                     self.view.invalidate_cache(); // 行合併影響多行
274.                     #[cfg(feature = "syntax-highlighting")]
275.                     self.highlight_cache.clear();
276. 
277.                     self.cursor
278.                         .set_position(&self.buffer, &self.view, new_row, prev_line_len);
279.                 }
280.                 self.selection_mode = false; // 刪除後關閉選擇模式
281.             }
282. 
283.             Command::Delete => {
284.                 if self.has_selection() {
285.                     self.delete_selection();
286.                 } else {
287.                     let pos = self.cursor.char_position(&self.buffer);
288.                     let line_content = self.buffer.get_line_content(self.cursor.row);
289.                     let at_line_end = self.cursor.col
290.                         >= line_content.trim_end_matches(['\n', '\r']).chars().count();
291. 
292.                     self.buffer.delete_char(pos);
293. 
294.                     // 優化：如果在行尾刪除（會合併下一行），需要完全失效；否則僅失效當前行
295.                     if at_line_end {
296.                         self.view.invalidate_cache(); // 行合併影響多行
297.                         #[cfg(feature = "syntax-highlighting")]
298.                         self.highlight_cache.clear();
299.                     } else {
300.                         self.view.invalidate_line(self.cursor.row); // 僅失效當前行
301.                         #[cfg(feature = "syntax-highlighting")]
302.                         self.invalidate_highlight_cache(self.cursor.row);
303.                     }
304.                 }
305.                 self.selection_mode = false; // 刪除後關閉選擇模式
306.             }
307. 
308.             Command::DeleteLine => {
309.                 if self.has_selection() {
310.                     self.delete_selection();
311.                 } else {
312.                     // 記錄是否在最後一行
313.                     let was_last_line = self.cursor.row == self.buffer.line_count() - 1;
314. 
315.                     self.buffer.delete_line(self.cursor.row);
316.                     self.view.invalidate_cache();
317.                     #[cfg(feature = "syntax-highlighting")]
318.                     self.highlight_cache.clear();
319. 
320.                     // 如果刪除的是最後一行且不是唯一一行，光標上移
321.                     if was_last_line && self.cursor.row > 0 {
322.                         self.cursor.row -= 1;
323.                     }
324. 
325.                     // 確保光標在有效範圍內
326.                     if self.cursor.row >= self.buffer.line_count() && self.buffer.line_count() > 0 {
327.                         self.cursor.row = self.buffer.line_count() - 1;
328.                     }
329. 
330.                     self.cursor.reset_to_line_start();
331.                 }
332.                 self.selection_mode = false; // 刪除後關閉選擇模式
333.             }
334. 
335.             // 光標移動
336.             Command::MoveUp => {
337.                 self.cursor.move_up(&self.buffer, &self.view);
338.                 self.selection = None;
339.             }
340.             Command::MoveDown => {
341.                 self.cursor.move_down(&self.buffer, &self.view);
342.                 self.selection = None;
343.             }
344.             Command::MoveLeft => {
345.                 self.cursor.move_left(&self.buffer, &self.view);
346.                 self.selection = None;
347.             }
348.             Command::MoveRight => {
349.                 self.cursor.move_right(&self.buffer, &self.view);
350.                 self.selection = None;
351.             }
352.             Command::MoveHome => {
353.                 self.cursor.move_to_line_start();
354.                 self.selection = None;
355.             }
356.             Command::MoveEnd => {
357.                 self.cursor.move_to_line_end(&self.buffer, &self.view);
358.                 self.selection = None;
359.             }
360.             Command::PageUp => {
361.                 let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
362.                 self.cursor
363.                     .move_page_up(&self.buffer, &self.view, effective_rows);
364.                 self.selection = None;
365.             }
366.             Command::PageDown => {
367.                 let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
368.                 self.cursor
369.                     .move_page_down(&self.buffer, &self.view, effective_rows);
370.                 self.selection = None;
371.             }
372. 
373.             Command::MoveToFileStart => {
374.                 self.cursor.move_to_file_start(&self.view);
375.                 self.selection = None;
376.             }
377.             Command::MoveToFileEnd => {
378.                 self.cursor.move_to_file_end(&self.buffer, &self.view);
379.                 self.selection = None;
380.             }
381. 
382.             Command::JumpTenthUp => {
383.                 let total_lines = self.buffer.line_count();
384.                 let jump_distance = total_lines.max(10) / 10; // 至少跳 1 行
385.                 self.cursor.row = self.cursor.row.saturating_sub(jump_distance);
386.                 self.cursor.set_position(
387.                     &self.buffer,
388.                     &self.view,
389.                     self.cursor.row,
390.                     self.cursor.col,
391.                 );
392.                 self.selection = None;
393.             }
394. 
395.             Command::JumpTenthDown => {
396.                 let total_lines = self.buffer.line_count();
397.                 let jump_distance = total_lines.max(10) / 10;
398.                 let new_row = self
399.                     .cursor
400.                     .row
401.                     .saturating_add(jump_distance)
402.                     .min(total_lines.saturating_sub(1));
403.                 self.cursor.row = new_row;
404.                 self.cursor.set_position(
405.                     &self.buffer,
406.                     &self.view,
407.                     self.cursor.row,
408.                     self.cursor.col,
409.                 );
410.                 self.selection = None;
411.             }
412. 
413.             // 選擇操作
414.             Command::ExtendSelection(direction) => {
415.                 if self.selection.is_none() {
416.                     self.selection = Some(Selection {
417.                         start: (self.cursor.row, self.cursor.col),
418.                         end: (self.cursor.row, self.cursor.col),
419.                     });
420.                 }
421. 
422.                 match direction {
423.                     Direction::Up => self.cursor.move_up(&self.buffer, &self.view),
424.                     Direction::Down => self.cursor.move_down(&self.buffer, &self.view),
425.                     Direction::Left => self.cursor.move_left(&self.buffer, &self.view),
426.                     Direction::Right => self.cursor.move_right(&self.buffer, &self.view),
427.                     Direction::Home => self.cursor.move_to_line_start(),
428.                     Direction::End => self.cursor.move_to_line_end(&self.buffer, &self.view),
429.                     Direction::FileStart => {
430.                         self.cursor.move_to_file_start(&self.view);
431.                     }
432.                     Direction::FileEnd => {
433.                         self.cursor.move_to_file_end(&self.buffer, &self.view);
434.                     }
435.                     Direction::PageUp => {
436.                         let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
437.                         self.cursor
438.                             .move_page_up(&self.buffer, &self.view, effective_rows)
439.                     }
440.                     Direction::PageDown => {
441.                         let effective_rows = self.view.get_effective_screen_rows(self.debug_mode);
442.                         self.cursor
443.                             .move_page_down(&self.buffer, &self.view, effective_rows)
444.                     }
445.                     Direction::TenthUp => {
446.                         let total_lines = self.buffer.line_count();
447.                         let jump_distance = total_lines.max(10) / 10;
448.                         self.cursor.row = self.cursor.row.saturating_sub(jump_distance);
449.                         self.cursor.set_position(
450.                             &self.buffer,
451.                             &self.view,
452.                             self.cursor.row,
453.                             self.cursor.col,
454.                         );
455.                     }
456.                     Direction::TenthDown => {
457.                         let total_lines = self.buffer.line_count();
458.                         let jump_distance = total_lines.max(10) / 10;
459.                         let new_row = self
460.                             .cursor
461.                             .row
462.                             .saturating_add(jump_distance)
463.                             .min(total_lines.saturating_sub(1));
464.                         self.cursor.row = new_row;
465.                         self.cursor.set_position(
466.                             &self.buffer,
467.                             &self.view,
468.                             self.cursor.row,
469.                             self.cursor.col,
470.                         );
471.                     }
472.                 }
473. 
474.                 if let Some(sel) = &mut self.selection {
475.                     sel.end = (self.cursor.row, self.cursor.col);
476.                 }
477.             }
478. 
479.             Command::SelectAll => {
480.                 let last_line = self.buffer.line_count().saturating_sub(1);
481.                 let last_col = self
482.                     .buffer
483.                     .get_line_content(last_line)
484.                     .trim_end_matches(['\n', '\r'])
485.                     .chars()
486.                     .count();
487. 
488.                 self.selection = Some(Selection {
489.                     start: (0, 0),
490.                     end: (last_line, last_col),
491.                 });
492.                 self.cursor.row = last_line;
493.                 self.cursor.col = last_col;
494.             }
495. 
496.             Command::ClearSelection => {
497.                 self.selection = None;
498.             }
499. 
500.             Command::ClearMessage => {
501.                 self.selection = None;
502.                 self.selection_mode = false; // ESC 關閉選擇模式但保留選擇範圍
503.                 self.search_mode = false; // ESC 關閉搜尋模式（保留搜尋結果）
504.                 self.message = None;
505.             }
506. 
507.             // 選擇模式切換
508.             Command::ToggleSelectionMode => {
509.                 self.selection_mode = !self.selection_mode;
510. 
511.                 // 開啟選擇模式時，如果沒有選擇範圍，初始化選擇
512.                 if self.selection_mode && self.selection.is_none() {
513.                     self.selection = Some(Selection {
514.                         start: (self.cursor.row, self.cursor.col),
515.                         end: (self.cursor.row, self.cursor.col),
516.                     });
517.                 }
518. 
519.                 self.message = Some(format!(
520.                     "Selection Mode: {}",
521.                     if self.selection_mode { "ON" } else { "OFF" }
522.                 ));
523.             }
524. 
525.             // 剪貼板操作
526.             Command::Copy => {
527.                 let text = self.get_copy_text();
528.                 self.set_clipboard_text(text, true);
529.                 // 複製後關閉選擇模式並清除選擇範圍
530.                 self.selection_mode = false;
531.                 self.selection = None;
532.             }
533. 
534.             Command::Cut => {
535.                 let text = self.get_copy_text();
536.                 self.set_clipboard_text(text, true);
537. 
538.                 // 剪切後刪除內容
539.                 if self.has_selection() {
540.                     self.delete_selection();
541.                 } else {
542.                     // 記錄是否在最後一行
543.                     let was_last_line = self.cursor.row == self.buffer.line_count() - 1;
544. 
545.                     self.buffer.delete_line(self.cursor.row);
546.                     self.view.invalidate_cache();
547. 
548.                     // 如果刪除的是最後一行且不是唯一一行，光標上移
549.                     if was_last_line && self.cursor.row > 0 {
550.                         self.cursor.row -= 1;
551.                     }
552. 
553.                     // 確保光標在有效範圍內
554.                     if self.cursor.row >= self.buffer.line_count() && self.buffer.line_count() > 0 {
555.                         self.cursor.row = self.buffer.line_count() - 1;
556.                     }
557. 
558.                     self.cursor.col = 0;
559.                     self.cursor.desired_visual_col = 0;
560.                 }
561. 
562.                 // 剪切後關閉選擇模式並清除選擇
563.                 self.selection_mode = false;
564.             }
565. 
566.             Command::Paste => {
567.                 let text = self.get_clipboard_text(true);
568.                 self.paste_text(text);
569.                 self.selection_mode = false; // 貼上後關閉選擇模式
570.             }
571. 
572.             // 內部剪貼板操作（僅使用內部剪貼簿）
573.             Command::CopyInternal => {
574.                 let text = self.get_copy_text();
575.                 self.set_clipboard_text(text, false);
576.                 self.selection_mode = false; // 複製後關閉選擇模式
577.                 self.selection = None; // 複製後清除選擇範圍
578.             }
579. 
580.             Command::CutInternal => {
581.                 let text = self.get_copy_text();
582.                 self.set_clipboard_text(text, false);
583. 
584.                 // 剪切後刪除內容
585.                 if self.has_selection() {
586.                     self.delete_selection();
587.                 } else {
588.                     // 記錄是否在最後一行
589.                     let was_last_line = self.cursor.row == self.buffer.line_count() - 1;
590. 
591.                     self.buffer.delete_line(self.cursor.row);
592.                     self.view.invalidate_cache();
593. 
594.                     // 如果刪除的是最後一行且不是唯一一行，光標上移
595.                     if was_last_line && self.cursor.row > 0 {
596.                         self.cursor.row -= 1;
597.                     }
598. 
599.                     // 確保光標在有效範圍內
600.                     if self.cursor.row >= self.buffer.line_count() && self.buffer.line_count() > 0 {
601.                         self.cursor.row = self.buffer.line_count() - 1;
602.                     }
603. 
604.                     self.cursor.col = 0;
605.                     self.cursor.desired_visual_col = 0;
606.                 }
607.                 self.selection_mode = false; // 剪切後關閉選擇模式
608.             }
609. 
610.             Command::PasteInternal => {
611.                 let text = self.get_clipboard_text(false);
612.                 self.paste_text(text);
613.                 self.selection_mode = false; // 貼上後關閉選擇模式
614.             }
615. 
616.             // 文件操作
617.             Command::Save => {
618.                 if let Err(e) = self.buffer.save() {
619.                     self.message = Some(format!("Save failed: {}", e));
620.                 } else {
621.                     self.message = Some("File saved".to_string());
622.                 }
623.             }
624. 
625.             Command::Quit => {
626.                 if self.buffer.is_modified() {
627.                     if self.quit_times > 0 {
628.                         // 第二次按 Ctrl+Q，強制退出
629.                         self.should_quit = true;
630.                     } else {
631.                         // 第一次按 Ctrl+Q，顯示警告
632.                         self.quit_times = 1;
633.                         self.message = Some(
634.                             "Unsaved changes! Press Ctrl+Q again to force quit, or Ctrl+W to save"
635.                                 .to_string(),
636.                         );
637.                     }
638.                 } else {
639.                     self.should_quit = true;
640.                 }
641.             }
642. 
643.             // 視窗調整
644.             Command::Resize => {
645.                 self.view.update_size();
646.             }
647. 
648.             // 撤銷/重做
649.             Command::Undo => {
650.                 if let Some(pos) = self.buffer.undo() {
651.                     self.view.invalidate_cache();
652.                     // 將光標移動到撤銷操作的位置
653.                     let row = self.buffer.char_to_line(pos);
654.                     let line_start = self.buffer.line_to_char(row);
655.                     let col = pos - line_start;
656. 
657.                     self.cursor.row = row;
658.                     self.cursor.col = col;
659.                     self.cursor.desired_visual_col = col;
660.                     self.message = Some("Undo".to_string());
661.                 } else {
662.                     self.message = Some("Nothing to undo".to_string());
663.                 }
664.             }
665. 
666.             Command::Redo => {
667.                 if let Some(pos) = self.buffer.redo() {
668.                     self.view.invalidate_cache();
669.                     // 將光標移動到重做操作的位置
670.                     let row = self.buffer.char_to_line(pos);
671.                     let line_start = self.buffer.line_to_char(row);
672.                     let col = pos - line_start;
673. 
674.                     self.cursor.row = row;
675.                     self.cursor.col = col;
676.                     self.cursor.desired_visual_col = col;
677.                     self.message = Some("Redo".to_string());
678.                 } else {
679.                     self.message = Some("Nothing to redo".to_string());
680.                 }
681.             }
682. 
683.             // 搜索
684.             Command::Find => {
685.                 // 獲取搜索查詢，使用上次的搜索詞作為預設值
686.                 let default_query = self.search.get_query();
687.                 if let Ok(Some(query)) = crate::dialog::prompt_with_default(
688.                     "Search:",
689.                     default_query,
690.                     self.terminal.size(),
691.                 ) {
692.                     if !query.is_empty() {
693.                         self.search.set_query(query.clone());
694.                         self.search.find_matches(&self.buffer);
695.                         self.search_mode = true; // 開啟搜尋模式
696. 
697.                         if self.search.match_count() > 0 {
698.                             if let Some((row, col)) = self.search.next_match() {
699.                                 self.cursor.row = row;
700.                                 self.cursor.col = col;
701.                                 self.cursor.desired_visual_col = col;
702.                                 self.message = Some(format!(
703.                                     "Found {} matches (ESC to exit search mode)",
704.                                     self.search.match_count()
705.                                 ));
706.                             }
707.                         } else {
708.                             self.message = Some(format!("No matches found for '{}'", query));
709.                             self.search_mode = false; // 沒有結果就關閉搜尋模式
710.                         }
711.                     }
712.                 }
713.             }
714. 
715.             Command::FindNext => {
716.                 if self.search_mode && self.search.match_count() > 0 {
717.                     if let Some((row, col)) = self.search.next_match() {
718.                         self.cursor.row = row;
719.                         self.cursor.col = col;
720.                         self.cursor.desired_visual_col = col;
721.                         self.message = Some(format!(
722.                             "Match {}/{} (ESC to exit search mode)",
723.                             self.search.current_index() + 1,
724.                             self.search.match_count()
725.                         ));
726.                     }
727.                 } else {
728.                     // 沒有搜尋模式時，執行 PageDown
729.                     return self.handle_command(Command::PageDown);
730.                 }
731.             }
732. 
733.             Command::FindPrev => {
734.                 if self.search_mode && self.search.match_count() > 0 {
735.                     if let Some((row, col)) = self.search.prev_match() {
736.                         self.cursor.row = row;
737.                         self.cursor.col = col;
738.                         self.cursor.desired_visual_col = col;
739.                         self.message = Some(format!(
740.                             "Match {}/{} (ESC to exit search mode)",
741.                             self.search.current_index() + 1,
742.                             self.search.match_count()
743.                         ));
744.                     }
745.                 } else {
746.                     // 沒有搜尋模式時，執行 PageUp
747.                     return self.handle_command(Command::PageUp);
748.                 }
749.             }
750. 
751.             // 視圖控制
752.             Command::ToggleLineNumbers => {
753.                 self.view.toggle_line_numbers();
754.             }
755. 
756.             // 註解切換
757.             Command::ToggleComment => {
758.                 if !self.comment_handler.has_comment_style() {
759.                     self.message = Some("No comment style for this file type".to_string());
760.                 } else if self.has_selection() {
761.                     // 多行選擇：智能切換註解
762.                     if let Some(sel) = self.selection {
763.                         let (start_row, _) = sel.start.min(sel.end);
764.                         let (end_row, _) = sel.start.max(sel.end);
765. 
766.                         // 檢查是否有任何一行沒有註解
767.                         let mut has_uncommented = false;
768.                         for row in start_row..=end_row {
769.                             let line_content = self.buffer.get_line_content(row);
770.                             if !self.comment_handler.is_commented(&line_content) {
771.                                 has_uncommented = true;
772.                                 break;
773.                             }
774.                         }
775. 
776.                         // 如果有任何一行沒註解，全部加註解；否則全部取消註解
777.                         let should_add_comment = has_uncommented;
778. 
779.                         // 從後往前處理，避免行號變化
780.                         for row in (start_row..=end_row).rev() {
781.                             let line_content = self.buffer.get_line_content(row);
782. 
783.                             let new_line = if should_add_comment {
784.                                 // 全部加註解（即使已經有註解的也保持不變）
785.                                 if self.comment_handler.is_commented(&line_content) {
786.                                     Some(line_content.clone())
787.                                 } else {
788.                                     self.comment_handler.add_comment(&line_content)
789.                                 }
790.                             } else {
791.                                 // 全部取消註解
792.                                 self.comment_handler.remove_comment(&line_content)
793.                             };
794. 
795.                             if let Some(new_line) = new_line {
796.                                 // 計算行的起始和結束位置
797.                                 let line_start = self.buffer.line_to_char(row);
798.                                 let line_end = if row + 1 < self.buffer.line_count() {
799.                                     self.buffer.line_to_char(row + 1)
800.                                 } else {
801.                                     self.buffer.len_chars()
802.                                 };
803. 
804.                                 // 刪除舊行（包括換行符）
805.                                 self.buffer.delete_range(line_start, line_end);
806. 
807.                                 // 插入新行（保留換行符）
808.                                 let new_line_with_newline = if line_content.ends_with('\n')
809.                                     || line_content.ends_with("\r\n")
810.                                 {
811.                                     format!("{}\n", new_line.trim_end_matches(['\n', '\r']))
812.                                 } else {
813.                                     new_line.trim_end_matches(['\n', '\r']).to_string()
814.                                 };
815.                                 self.buffer.insert(line_start, &new_line_with_newline);
816.                             }
817.                         }
818. 
819.                         self.view.invalidate_cache();
820. 
821.                         // 保留選擇狀態（不清除選取）
822.                         self.cursor.row = start_row;
823.                         self.cursor.col = 0;
824.                         self.cursor.desired_visual_col = 0;
825. 
826.                         let action = if should_add_comment {
827.                             "Added"
828.                         } else {
829.                             "Removed"
830.                         };
831.                         self.message = Some(format!("{} comments", action));
832.                     }
833.                 } else {
834.                     // 單行：直接切換註解
835.                     let line_content = self.buffer.get_line_content(self.cursor.row);
836.                     if let Some(new_line) = self.comment_handler.toggle_line_comment(&line_content)
837.                     {
838.                         // 計算行的起始和結束位置
839.                         let line_start = self.buffer.line_to_char(self.cursor.row);
840.                         let line_end = if self.cursor.row + 1 < self.buffer.line_count() {
841.                             self.buffer.line_to_char(self.cursor.row + 1)
842.                         } else {
843.                             self.buffer.len_chars()
844.                         };
845. 
846.                         // 刪除舊行（包括換行符）
847.                         self.buffer.delete_range(line_start, line_end);
848. 
849.                         // 插入新行（保留換行符）
850.                         let new_line_with_newline =
851.                             if line_content.ends_with('\n') || line_content.ends_with("\r\n") {
852.                                 format!("{}\n", new_line.trim_end_matches(['\n', '\r']))
853.                             } else {
854.                                 new_line.trim_end_matches(['\n', '\r']).to_string()
855.                             };
856.                         self.buffer.insert(line_start, &new_line_with_newline);
857. 
858.                         self.view.invalidate_cache();
859. 
860.                         self.message = Some("Toggled comment".to_string());
861.                     }
862.                 }
863.             }
864. 
865.             // 縮排（Tab 鍵）
866.             Command::Indent => {
867.                 if self.has_selection() {
868.                     // 多行選擇：對每行添加 4 個空格
869.                     if let Some(sel) = self.selection {
870.                         let (start_row, _) = sel.start.min(sel.end);
871.                         let (end_row, _) = sel.start.max(sel.end);
872. 
873.                         // 從後往前處理，避免行號變化
874.                         for row in (start_row..=end_row).rev() {
875.                             let line_start = self.buffer.line_to_char(row);
876.                             self.buffer.insert(line_start, "    ");
877.                         }
878. 
879.                         self.view.invalidate_cache();
880. 
881.                         // 保留選擇狀態
882.                         self.cursor.row = start_row;
883.                         self.cursor.col = 0;
884.                         self.cursor.desired_visual_col = 0;
885.                     }
886.                 } else {
887.                     // 單行：在光標位置插入 4 個空格
888.                     let pos = self.cursor.char_position(&self.buffer);
889.                     self.buffer.insert(pos, "    ");
890.                     self.view.invalidate_cache();
891.                     self.cursor.col += 4;
892.                     self.cursor.desired_visual_col = self.cursor.col;
893.                 }
894.             }
895. 
896.             // 退位（Shift+Tab 鍵）
897.             Command::Unindent => {
898.                 if self.has_selection() {
899.                     // 多行選擇：對每行刪除最多 4 個前導空格
900.                     if let Some(sel) = self.selection {
901.                         let (start_row, _) = sel.start.min(sel.end);
902.                         let (end_row, _) = sel.start.max(sel.end);
903. 
904.                         // 從後往前處理，避免行號變化
905.                         for row in (start_row..=end_row).rev() {
906.                             let line_content = self.buffer.get_line_content(row);
907.                             let spaces_to_remove = line_content
908.                                 .chars()
909.                                 .take_while(|&c| c == ' ')
910.                                 .take(4)
911.                                 .count();
912. 
913.                             if spaces_to_remove > 0 {
914.                                 let line_start = self.buffer.line_to_char(row);
915.                                 self.buffer
916.                                     .delete_range(line_start, line_start + spaces_to_remove);
917.                             }
918.                         }
919. 
920.                         self.view.invalidate_cache();
921. 
922.                         // 保留選擇狀態
923.                         self.cursor.row = start_row;
924.                         self.cursor.col = 0;
925.                         self.cursor.desired_visual_col = 0;
926.                     }
927.                 } else {
928.                     // 單行：刪除光標前最多 4 個空格
929.                     let line_content = self.buffer.get_line_content(self.cursor.row);
930.                     let before_cursor: String =
931.                         line_content.chars().take(self.cursor.col).collect();
932.                     let spaces_to_remove = before_cursor
933.                         .chars()
934.                         .rev()
935.                         .take_while(|&c| c == ' ')
936.                         .take(4)
937.                         .count();
938. 
939.                     if spaces_to_remove > 0 {
940.                         let line_start = self.buffer.line_to_char(self.cursor.row);
941.                         let delete_start = line_start + self.cursor.col - spaces_to_remove;
942.                         self.buffer
943.                             .delete_range(delete_start, delete_start + spaces_to_remove);
944.                         self.view.invalidate_cache();
945.                         self.cursor.col -= spaces_to_remove;
946.                         self.cursor.desired_visual_col = self.cursor.col;
947.                     }
948.                 }
949.             }
950. 
951.             // 跳轉到行
952.             Command::GoToLine => {
953.                 if let Ok(Some(line_str)) =
954.                     crate::dialog::prompt("Go to line:", self.terminal.size())
955.                 {
956.                     if let Ok(line_num) = line_str.trim().parse::<usize>() {
957.                         if line_num > 0 && line_num <= self.buffer.line_count() {
958.                             self.cursor.row = line_num - 1;
959.                             self.cursor.col = 0;
960.                             self.cursor.desired_visual_col = 0;
961.                             self.message = Some(format!("Jumped to line {}", line_num));
962.                         } else {
963.                             self.message = Some(format!("Invalid line number: {}", line_num));
964.                         }
965.                     } else {
966.                         self.message = Some("Please enter a valid number".to_string());
967.                     }
968.                 }
969.             }
970. 
971.             // 編碼切換
972.             Command::ChangeEncoding => {
973.                 if let Ok(Some(encoding_str)) =
974.                     crate::dialog::prompt("Change encoding to:", self.terminal.size())
975.                 {
976.                     if let Some(encoding) = Self::parse_encoding(&encoding_str) {
977.                         // 檢查是否有檔案路徑（區分已存在檔案和新建檔案）
978.                         if self.buffer.has_file_path() {
979.                             // 已存在的檔案：需要重新載入
980.                             if self.buffer.is_modified() {
981.                                 // 有未保存的修改，顯示確認對話框
982.                                 if let Ok(confirmed) = crate::dialog::confirm(
983.                                     "Unsaved changes will be lost. Continue?",
984.                                     self.terminal.size(),
985.                                 ) {
986.                                     if confirmed {
987.                                         match self.buffer.reload_with_encoding(encoding) {
988.                                             Ok(_) => {
989.                                                 // 重新載入成功，重置游標
990.                                                 self.cursor.row = 0;
991.                                                 self.cursor.col = 0;
992.                                                 self.cursor.desired_visual_col = 0;
993.                                                 self.cursor.visual_line_index = 0;
994.                                                 self.view.invalidate_cache();
995.                                                 self.message = Some(format!(
996.                                                     "Encoding changed to {} (file reloaded)",
997.                                                     encoding.name()
998.                                                 ));
999.                                             }
1000.                                             Err(e) => {
1001.                                                 self.message =
1002.                                                     Some(format!("Failed to reload file: {}", e));
1003.                                             }
1004.                                         }
1005.                                     }
1006.                                 }
1007.                             } else {
1008.                                 // 沒有未保存的修改，直接重新載入
1009.                                 match self.buffer.reload_with_encoding(encoding) {
1010.                                     Ok(_) => {
1011.                                         self.cursor.row = 0;
1012.                                         self.cursor.col = 0;
1013.                                         self.cursor.desired_visual_col = 0;
1014.                                         self.cursor.visual_line_index = 0;
1015.                                         self.view.invalidate_cache();
1016.                                         self.message = Some(format!(
1017.                                             "Encoding changed to {} (file reloaded)",
1018.                                             encoding.name()
1019.                                         ));
1020.                                     }
1021.                                     Err(e) => {
1022.                                         self.message =
1023.                                             Some(format!("Failed to reload file: {}", e));
1024.                                     }
1025.                                 }
1026.                             }
1027.                         } else {
1028.                             // 新建檔案：只設定編碼，不重新載入
1029.                             self.buffer.change_encoding(encoding);
1030.                             self.message = Some(format!(
1031.                                 "Encoding set to {} (will be used on save)",
1032.                                 encoding.name()
1033.                             ));
1034.                         }
1035.                     } else {
1036.                         self.message = Some(format!("Unsupported encoding: {}", encoding_str));
1037.                     }
1038.                 }
1039.             }
1040. 
1041.             // 切換語法高亮
1042.             #[cfg(feature = "syntax-highlighting")]
1043.             Command::ToggleSyntaxHighlight => {
1044.                 self.highlight_enabled = !self.highlight_enabled;
1045.                 self.message = Some(format!(
1046.                     "Syntax Highlight: {}",
1047.                     if self.highlight_enabled {
1048.                         "Enabled"
1049.                     } else {
1050.                         "Disabled"
1051.                     }
1052.                 ));
1053.             }
1054. 
1055.             // 顯示幫助
1056.             Command::ShowHelp => {
1057.                 // 保存當前終端狀態
1058.                 if let Err(e) = crate::dialog::show_help(self.terminal.size()) {
1059.                     self.message = Some(format!("Failed to show help: {}", e));
1060.                 }
1061.                 // 重新繪製編輯器畫面
1062.                 self.view.invalidate_cache();
1063.             }
1064.         }
1065. 
1066.         Ok(())
1067.     }
1068. 
1069.     fn has_selection(&self) -> bool {
1070.         self.selection.is_some()
1071.     }
1072. 
1073.     /// 獲取要複製/剪切的文本
1074.     /// 如果有選擇範圍，返回選擇的文本；否則返回當前整行（帶換行符）
1075.     fn get_copy_text(&self) -> String {
1076.         if self.has_selection() {
1077.             self.get_selected_text()
1078.         } else {
1079.             // 複製當前整行（完整內容，包括尾部空格和換行符）
1080.             let line_text = self.buffer.get_line_full(self.cursor.row);
1081.             // 確保以換行符結尾（用於識別整行貼上）
1082.             if line_text.ends_with('\n') {
1083.                 line_text
1084.             } else {
1085.                 format!("{}\n", line_text)
1086.             }
1087.         }
1088.     }
1089. 
1090.     /// 設置剪貼簿內容
1091.     /// use_system: true 表示使用系統剪貼簿，false 表示僅使用內部剪貼簿
1092.     fn set_clipboard_text(&mut self, text: String, use_system: bool) {
1093.         if use_system {
1094.             // 嘗試系統剪貼簿，失敗則回退到內部剪貼簿
1095.             if self.clipboard.set_text(&text).is_err() && !self.clipboard.is_available() {
1096.                 self.message = Some("Copied (internal clipboard)".to_string());
1097.             }
1098.             self.internal_clipboard = text; // 同步到內部剪貼簿
1099.         } else {
1100.             // 僅使用內部剪貼簿
1101.             self.internal_clipboard = text;
1102.             self.message = Some("Copied (internal clipboard)".to_string());
1103.         }
1104.     }
1105. 
1106.     /// 獲取剪貼簿內容
1107.     /// use_system: true 表示優先使用系統剪貼簿，false 表示僅使用內部剪貼簿
1108.     fn get_clipboard_text(&mut self, use_system: bool) -> String {
1109.         if use_system {
1110.             // 嘗試從系統剪貼簿獲取，失敗則使用內部剪貼簿
1111.             self.clipboard.get_text().unwrap_or_else(|_| {
1112.                 if self.internal_clipboard.is_empty() {
1113.                     if !self.clipboard.is_available() {
1114.                         self.message = Some("Nothing to paste (internal clipboard)".to_string());
1115.                     }
1116.                     String::new()
1117.                 } else {
1118.                     self.internal_clipboard.clone()
1119.                 }
1120.             })
1121.         } else {
1122.             // 僅使用內部剪貼簿
1123.             if self.internal_clipboard.is_empty() {
1124.                 self.message = Some("Nothing to paste (internal clipboard)".to_string());
1125.                 String::new()
1126.             } else {
1127.                 self.internal_clipboard.clone()
1128.             }
1129.         }
1130.     }
1131. 
1132.     /// 執行貼上操作
1133.     fn paste_text(&mut self, text: String) {
1134.         if text.is_empty() {
1135.             return;
1136.         }
1137. 
1138.         if self.has_selection() {
1139.             self.delete_selection();
1140.         }
1141. 
1142.         // 檢查是否為整行貼上（文字以換行結尾）
1143.         let is_whole_line = text.ends_with('\n');
1144. 
1145.         if is_whole_line {
1146.             // 整行貼上：在光標所在行的開始處插入
1147.             let line_start = self.buffer.line_to_char(self.cursor.row);
1148.             self.buffer.insert(line_start, &text);
1149.             self.view.invalidate_cache();
1150. 
1151.             // 計算插入了多少行
1152.             let inserted_lines = text.chars().filter(|&c| c == '\n').count();
1153. 
1154.             // 光標移動到被擠下去的原行首
1155.             self.cursor.row += inserted_lines;
1156.             self.cursor.col = 0;
1157.             self.cursor.desired_visual_col = 0;
1158.         } else {
1159.             // 普通貼上：在光標位置插入
1160.             let pos = self.cursor.char_position(&self.buffer);
1161.             self.buffer.insert(pos, &text);
1162.             self.view.invalidate_cache();
1163.             // 移動到貼上內容末尾
1164.             for ch in text.chars() {
1165.                 if ch == '\n' {
1166.                     self.cursor.row += 1;
1167.                     self.cursor.col = 0;
1168.                 } else {
1169.                     self.cursor.col += 1;
1170.                 }
1171.             }
1172.             self.cursor.desired_visual_col = self.cursor.col;
1173.         }
1174.     }
1175. 
1176.     fn get_selected_text(&self) -> String {
1177.         if let Some(sel) = self.selection {
1178.             let (start_row, start_col) = sel.start.min(sel.end);
1179.             let (end_row, end_col) = sel.start.max(sel.end);
1180. 
1181.             let mut text = String::new();
1182. 
1183.             for row in start_row..=end_row {
1184.                 let line = self.buffer.get_line_content(row);
1185.                 let line = line.trim_end_matches(['\n', '\r']);
1186. 
1187.                 if row == start_row && row == end_row {
1188.                     // 單行選擇
1189.                     let chars: Vec<char> = line.chars().collect();
1190.                     text.push_str(
1191.                         &chars[start_col..end_col.min(chars.len())]
1192.                             .iter()
1193.                             .collect::<String>(),
1194.                     );
1195.                 } else if row == start_row {
1196.                     // 第一行
1197.                     let chars: Vec<char> = line.chars().collect();
1198.                     text.push_str(&chars[start_col..].iter().collect::<String>());
1199.                     text.push('\n');
1200.                 } else if row == end_row {
1201.                     // 最後一行
1202.                     let chars: Vec<char> = line.chars().collect();
1203.                     text.push_str(&chars[..end_col.min(chars.len())].iter().collect::<String>());
1204.                 } else {
1205.                     // 中間行
1206.                     text.push_str(line);
1207.                     text.push('\n');
1208.                 }
1209.             }
1210. 
1211.             text
1212.         } else {
1213.             String::new()
1214.         }
1215.     }
1216. 
1217.     fn delete_selection(&mut self) {
1218.         if let Some(sel) = self.selection {
1219.             let (start_row, start_col) = sel.start.min(sel.end);
1220.             let (end_row, end_col) = sel.start.max(sel.end);
1221. 
1222.             let start_pos = self.buffer.line_to_char(start_row) + start_col;
1223.             let end_pos = self.buffer.line_to_char(end_row) + end_col;
1224. 
1225.             self.buffer.delete_range(start_pos, end_pos);
1226.             self.view.invalidate_cache();
1227. 
1228.             self.cursor
1229.                 .set_position(&self.buffer, &self.view, start_row, start_col);
1230.             self.selection = None;
1231.         }
1232.     }
1233. 
1234.     fn get_debug_info(&self) -> String {
1235.         let total_lines = self.buffer.line_count();
1236.         let screen_rows = self.view.screen_rows;
1237.         let logical_row = self.cursor.row;
1238.         let logical_col = self.cursor.col;
1239.         let visual_line_index = self.cursor.visual_line_index;
1240. 
1241.         // 計算可用列寬度
1242.         let available_width = self.view.get_available_width(&self.buffer);
1243. 
1244.         // 計算當前行的視覺列位置和總字符數
1245.         let (
1246.             visual_col_in_line,
1247.             line_char_count,
1248.             line_visual_width,
1249.             total_visual_lines,
1250.             current_visual_line_width,
1251.         ) = if let Some(line) = self.buffer.line(logical_row) {
1252.             let line_str = line.to_string();
1253.             let line_str = line_str.trim_end_matches(['\n', '\r']);
1254.             let visual_col = self.view.logical_col_to_visual_col(line_str, logical_col);
1255.             let char_count = line_str.chars().count();
1256. 
1257.             // 計算在當前視覺行內的列位置
1258.             let visual_lines = self
1259.                 .view
1260.                 .calculate_visual_lines_for_row(&self.buffer, logical_row);
1261.             let total_visual_lines = visual_lines.len();
1262.             let mut accumulated = 0;
1263.             for line in visual_lines
1264.                 .iter()
1265.                 .take(visual_line_index.min(visual_lines.len()))
1266.             {
1267.                 accumulated += visual_width(line);
1268.             }
1269.             let col_in_visual_line = visual_col.saturating_sub(accumulated);
1270. 
1271.             // 計算整行的視覺寬度
1272.             let line_visual_width = visual_width(line_str);
1273. 
1274.             // 計算當前視覺行的寬度
1275.             let current_visual_line_width = if visual_line_index < visual_lines.len() {
1276.                 visual_width(&visual_lines[visual_line_index])
1277.             } else {
1278.                 0
1279.             };
1280. 
1281.             (
1282.                 col_in_visual_line,
1283.                 char_count,
1284.                 line_visual_width,
1285.                 total_visual_lines,
1286.                 current_visual_line_width,
1287.             )
1288.         } else {
1289.             (0, 0, 0, 0, 0)
1290.         };
1291. 
1292.         // 計算選取的邏輯字數和顯示寬度
1293.         let (selection_char_count, selection_visual_width) = if self.selection.is_some() {
1294.             let selected_text = self.get_selected_text();
1295.             let char_count = selected_text.chars().count();
1296.             let visual_width = visual_width(&selected_text);
1297.             (char_count, visual_width)
1298.         } else {
1299.             (0, 0)
1300.         };
1301. 
1302.         format!(
1303.             "DEBUG | AA:{}x{} LL:L{}/{}:C{}/{}:{} VL:L{}/{}:C{}/{} SC:{}:{}",
1304.             screen_rows,
1305.             available_width,
1306.             logical_row + 1,
1307.             total_lines,
1308.             logical_col,
1309.             line_char_count,
1310.             line_visual_width,
1311.             visual_line_index + 1,
1312.             total_visual_lines,
1313.             visual_col_in_line,
1314.             current_visual_line_width,
1315.             selection_char_count,
1316.             selection_visual_width
1317.         )
1318.     }
1319. 
1320.     /// 獲取語法高亮後的行
1321.     ///
1322.     /// 使用增量處理策略：智慧選擇起始行，維護語法狀態的正確性和效能平衡
1323.     #[cfg(feature = "syntax-highlighting")]
1324.     pub fn get_highlighted_lines(
1325.         &mut self,
1326.         start_row: usize,
1327.         end_row: usize,
1328.     ) -> std::collections::HashMap<usize, String> {
1329.         use crate::highlight::CachedLine;
1330. 
1331.         let mut result = std::collections::HashMap::new();
1332. 
1333.         // 檢查是否有語法高亮引擎
1334.         let Some(ref engine) = self.highlight_engine else {
1335.             return result;
1336.         };
1337. 
1338.         // 建立高亮器
1339.         let Some(mut highlighter) = engine.create_highlighter() else {
1340.             return result;
1341.         };
1342. 
1343.         // 增量處理策略：智慧選擇起始行
1344.         // 1. 小檔案或接近檔案開頭：從第 0 行開始（保證正確性）
1345.         // 2. 大檔案：從 start_row - BUFFER 開始，平衡效能和正確性
1346.         const BUFFER_LINES: usize = 100; // 緩衝範圍
1347.         const SMALL_FILE_THRESHOLD: usize = 500; // 小檔案閾值
1348. 
1349.         let total_lines = self.buffer.line_count();
1350.         let is_small_file = total_lines <= SMALL_FILE_THRESHOLD;
1351.         let is_near_start = start_row < BUFFER_LINES;
1352. 
1353.         // 決定處理起始行
1354.         let process_start = if is_small_file || is_near_start {
1355.             0 // 小檔案或接近開頭，從第 0 行開始確保正確性
1356.         } else {
1357.             start_row.saturating_sub(BUFFER_LINES) // 大檔案，從緩衝區開始
1358.         };
1359. 
1360.         // 循序處理（維護跨行狀態）
1361.         for row in process_start..=end_row.min(total_lines.saturating_sub(1)) {
1362.             let line_text = match self.buffer.line(row) {
1363.                 Some(line) => {
1364.                     // ⚠️ 重要：保留換行符！syntect 需要換行符才能正確解析語法狀態
1365.                     // 參考：與 cate 專案相同的修復
1366.                     let mut text = line.to_string();
1367.                     // 確保有換行符（syntect 需要）
1368.                     if !text.ends_with('\n') && !text.ends_with("\r\n") {
1369.                         text.push('\n');
1370.                     }
1371.                     text
1372.                 }
1373.                 None => continue,
1374.             };
1375. 
1376.             // 檢查快取
1377.             if self.highlight_cache.is_valid(row, &line_text) {
1378.                 if row >= start_row {
1379.                     // 在可見區域內，使用快取
1380.                     if let Some(cached) = self.highlight_cache.get(row) {
1381.                         result.insert(row, cached.highlighted.clone());
1382.                     }
1383.                 }
1384.                 // 即使不在可見區域，也要處理這一行以維護狀態
1385.                 let _ = highlighter.highlight_line(&line_text);
1386.             } else {
1387.                 // 快取失效，重新高亮
1388.                 let mut highlighted = highlighter.highlight_line(&line_text);
1389. 
1390.                 // ⚠️ 修復：去除末尾的換行符，避免在 Linux 終端產生殘影
1391.                 // syntect 需要換行符來解析語法狀態，但渲染時不應輸出換行符
1392.                 highlighted = highlighted.trim_end_matches(&['\n', '\r'][..]).to_string();
1393. 
1394.                 // 更新快取
1395.                 self.highlight_cache.insert(
1396.                     row,
1397.                     CachedLine {
1398.                         text: line_text,
1399.                         highlighted: highlighted.clone(),
1400.                     },
1401.                 );
1402. 
1403.                 // 如果在可見區域，加入結果
1404.                 if row >= start_row {
1405.                     result.insert(row, highlighted);
1406.                 }
1407.             }
1408.         }
1409. 
1410.         result
1411.     }
1412. 
1413.     /// 使語法高亮快取失效（編輯操作後調用）
1414.     #[cfg(feature = "syntax-highlighting")]
1415.     pub fn invalidate_highlight_cache(&mut self, from_line: usize) {
1416.         use crate::highlight::EditType;
1417.         self.highlight_cache
1418.             .invalidate_from_edit(from_line, EditType::CharInsert);
1419.     }
1420. 
1421.     // 解析編碼字串
1422.     fn parse_encoding(enc_str: &str) -> Option<&'static encoding_rs::Encoding> {
1423.         match enc_str.to_lowercase().as_str() {
1424.             "utf-8" | "utf8" => Some(encoding_rs::UTF_8),
1425.             "utf-16le" | "utf16le" => Some(encoding_rs::UTF_16LE),
1426.             "utf-16be" | "utf16be" => Some(encoding_rs::UTF_16BE),
1427.             "gbk" | "cp936" => Some(encoding_rs::GBK),
1428.             "shift-jis" | "shift_jis" | "sjis" => Some(encoding_rs::SHIFT_JIS),
1429.             "big5" | "cp950" => encoding_rs::Encoding::for_label(b"big5"),
1430.             "cp1252" | "windows-1252" => Some(encoding_rs::WINDOWS_1252),
1431.             _ => encoding_rs::Encoding::for_label(enc_str.as_bytes()),
1432.         }
1433.     }
1434. }
1435.
```

</details>


---

<sub>⏱️ 6m 53s</sub>

### 💬 Copilot

現在我已經完整了解了程式碼結構。以下是評估報告：

---

## 評估報告：單行/多行顯示切換整合

### 1. 整合到行號顯示 (Ctrl+L 一起 toggle)

**可行性：高** ✅

目前架構：
- `View::show_line_numbers` 控制行號顯示
- `Command::ToggleLineNumbers` (Ctrl+L) 切換
- `wrap_line()` 函數負責根據 `available_width` 切分視覺行

**實現方式：**
```rust
// view.rs 增加模式控制
pub wrap_mode: bool,  // true=多行換行, false=單行滾動

pub fn toggle_line_numbers(&mut self) {
    self.show_line_numbers = !self.show_line_numbers;
    self.wrap_mode = self.show_line_numbers; // 連動
    self.invalidate_cache();
}
```

**影響範圍：**
- `view.rs`: `LineLayout::new()`, `wrap_line()`, `render()`
- `cursor.rs`: 移動邏輯需要調整

---

### 2. 單行顯示時的水平滾動

**複雜度：中** ⚠️

需要增加 `offset_col` 追蹤水平視口：

```rust
pub struct View {
    pub offset_row: usize,
    pub offset_col: usize,  // 新增：水平偏移（僅單行模式用）
    // ...
}
```

**游標跟隨邏輯：**
- 當游標超出右邊界：`offset_col = cursor_visual_col - available_width + margin`
- 當游標超出左邊界：`offset_col = cursor_visual_col - margin`

**關鍵修改點：**
- `scroll_if_needed()`: 新增水平滾動判斷
- `render()`: 截取 `displayed_line[offset_col..offset_col+width]`
- `get_cursor_visual_position()`: 計算時減去 `offset_col`

---

### 3. 對 Syntax Highlighting 的影響

**複雜度：中** ⚠️

目前問題：
```rust
// view.rs L275-277
let use_syntax_highlight = selection.is_none()
    && visual_idx == 0  // 只在第一個 visual line 使用（簡化處理）
    && highlighted_lines.and_then(|h| h.get(&file_row)).is_some();
```

單行模式需要：
1. 完整高亮整行（不只是可見部分）
2. 渲染時只顯示 `offset_col..offset_col+width` 區間的 ANSI 序列
3. 需要正確處理 ANSI escape codes 的邊界切割

**解決方案：**
- 使用 `strip_ansi_codes` 函數計算視覺位置
- 在切割時保留/恢復 ANSI 狀態
- 或改用逐字符渲染（性能較差但正確）

---

### 4. 單行顯示時的 Selection 行為

**複雜度：高** ⚠️⚠️

目前 Selection 邏輯（`view.rs L282-323`）基於視覺列計算：

```rust
// 判斷字符是否在選擇範圍
let is_selected = current_visual_pos \>= start_col && current_visual_pos \< end_col;
```

單行模式需要：
1. **視覺列計算仍有效**：Selection 的 start/end 是視覺座標，不受影響
2. **渲染時需調整**：只渲染 `offset_col..` 範圍內的字符
3. **Selection 可能超出可見範圍**：需要正確顯示部分選取

**修改點：**
```rust
// 單行模式下的渲染邏輯
for (idx, &ch) in chars.iter().enumerate() {
    let visual_pos = /* 計算 */;
    
    // 跳過 offset_col 之前的字符
    if visual_pos + ch_width \<= self.offset_col { continue; }
    // 超出可見範圍則停止
    if visual_pos \>= self.offset_col + available_width { break; }
    
    // Selection 判斷保持不變
    let is_selected = /* ... */;
}
```

---

### 5. Cursor 移動的影響

**複雜度：中** ⚠️

目前 `cursor.rs` 的 `move_up/down` 依賴 `visual_line_index`：

```rust
// 單行模式下這些概念需要簡化
pub visual_line_index: usize,  // 單行模式永遠 = 0
pub desired_visual_col: usize, // 行為不變
```

**需要修改：**
- `move_up()`: 直接 `row -= 1`（無需處理 visual lines）
- `move_down()`: 直接 `row += 1`
- `move_left/right()`: 行為不變，只是觸發水平滾動

---

### 實施建議優先順序

| 順序 | 項目 | 工作量 | 風險 |
|-----|------|--------|-----|
| 1 | 基本 toggle 機制 | 小 | 低 |
| 2 | 水平滾動 (`offset_col`) | 中 | 中 |
| 3 | Cursor 移動適配 | 中 | 中 |
| 4 | Selection 渲染 | 中 | 中 |
| 5 | Syntax Highlighting 適配 | 高 | 高 |

**建議：** 可以先實現基本的單行+水平滾動，Syntax Highlighting 在單行模式下暫時降級為純文字顯示。

---

<sub>⏱️ 9m 18s</sub>

### 👤 User

請建立開發計畫md檔

---

<sub>⏱️ 10m 38s</sub>

### ✅ `create`

**create a new file at D:\Users\user\Documents\rust\wedi\dev\single-line-mode-plan.md.**

D:\Users\user\Documents\rust\wedi\dev\single-line-mode-plan.md

```
Created file D:\Users\user\Documents\rust\wedi\dev\single-line-mode-plan.md with 9112 characters
```


---

<sub>⏱️ 10m 42s</sub>

### 💬 Copilot

已建立開發計畫：`dev\single-line-mode-plan.md`

主要內容：
- **5 個開發階段**：基礎架構 → 水平滾動 → Cursor 適配 → Selection 適配 → 語法高亮
- **預計時程**：約 1.5-2 個工作天
- **關鍵決策**：語法高亮建議先降級為純文字，後續再優化

---

<sub>Generated by [GitHub Copilot CLI](https://github.com/features/copilot/cli)</sub>
