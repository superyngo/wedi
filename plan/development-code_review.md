先說結論：
你現在「大檔 + 一次跳很多頁就卡死」的主嫌，幾乎可以確定是在 `View::scroll_if_needed` + `count_visual_lines` 這組，複雜度直接從 O(N) 變成 O(N²)；檔案一大、游標一次跳很遠，就開始瘋狂重算整段畫面高度，所以看起來像當機。

我先重點說「問題在哪」，再給你「怎麼改最划算」，最後補一些其它可優化點。

---

## 一、主要問題點：`scroll_if_needed` O(N²) 行為

### 1. 目前邏輯長這樣

在 `view.rs`：

```rust
fn scroll_if_needed(
    &mut self,
    cursor: &Cursor,
    buffer: &RopeBuffer,
    has_debug_ruler: bool,
) {
    // 向上滾動
    if cursor.row < self.offset_row {
        self.offset_row = cursor.row;
        return;
    }

    // 向下滾動：基於視覺行數判斷
    let effective_rows = self.get_effective_screen_rows(has_debug_ruler);
    let visual_offset =
        self.count_visual_lines(buffer, self.offset_row, cursor.row) + cursor.visual_line_index;

    // 如果光標視覺位置超出螢幕，向下調整 offset_row
    if visual_offset >= effective_rows {
        while self.offset_row < cursor.row {
            self.offset_row += 1;
            let new_offset = self.count_visual_lines(buffer, self.offset_row, cursor.row)
                + cursor.visual_line_index;
            if new_offset < effective_rows {
                break;
            }
        }
    }
}
```

`count_visual_lines` 裡面又是這樣：

```rust
fn count_visual_lines(
    &self,
    buffer: &RopeBuffer,
    start_row: usize,
    end_row: usize,
) -> usize {
    let mut total = 0;
    for row in start_row..end_row.min(buffer.line_count()) {
        let visual_lines = self.calculate_visual_lines_for_row(buffer, row);
        total += visual_lines.len();
    }
    total
}
```

而 `calculate_visual_lines_for_row`：

* `buffer.line(row).to_string()`
* `trim_end_matches(['\n', '\r'])`
* 展開 `\t` → `"    "`
* `wrap_line` 做折行，回傳 `Vec<String>`

也就是說：

* 每算一行視覺高度就會：

  * 從 Rope 拿一行 → `to_string` → 建一個 `String`
  * 展開 tab → 再組一個 `displayed_line: String`
  * `wrap_line` 再產一個 `Vec<String>`（視覺行）

### 2. 跳很多頁時的複雜度

假設：

* 目前 `offset_row = 0`
* 游標跳到 `cursor.row = 50_000`
* 螢幕大概只有幾十行

流程會變成：

1. 先算一次
   `visual_offset = count_visual_lines(0, 50_000) + cursor.visual_line_index`
   → 掃 50k 行，每行都做 tab 展開 + 折行 + 配置 `Vec<String>`
2. 發現 `visual_offset >= effective_rows`，進入 `while`：

   ```rust
   while self.offset_row < cursor.row {
       self.offset_row += 1;
       let new_offset = self.count_visual_lines(buffer, self.offset_row, cursor.row)
           + cursor.visual_line_index;
       if new_offset < effective_rows {
           break;
       }
   }
   ```

   這裡超可怕：

   * 第一次：`count_visual_lines(1, 50_000)` → 掃 49,999 行
   * 第二次：`count_visual_lines(2, 50_000)` → 掃 49,998 行
   * …
   * 直到 offset 接近 cursor.row 才停

所以：

* 距離越遠（cursor.row - offset_row 越大），你就 **每次減 1** 去重算整段的視覺高度
* 每次重算都要重新從 Rope 把這段所有行 `to_string`、展開 tab、折行

結果：**O(D² * 行長)**，D 是游標與 offset 差距。
大檔 + 直接跳到檔尾 = 直接把 CPU 打爆，看起來就像整個 editor 卡死。

---

## 二、最值得先改的地方（優先順序）

### 1. 先把 `scroll_if_needed` 改成「只看邏輯行」版本（最簡單救命版）

如果你現在只是要「先不要卡死」，可以先放棄精準的「視覺行」滾動，改回比較粗略的「以邏輯行數」判斷是否需要捲動。

比方：

```rust
fn scroll_if_needed(
    &mut self,
    cursor: &Cursor,
    _buffer: &RopeBuffer,
    has_debug_ruler: bool,
) {
    let effective_rows = self.get_effective_screen_rows(has_debug_ruler);

    // 向上：直接貼齊
    if cursor.row < self.offset_row {
        self.offset_row = cursor.row;
        return;
    }

    // 向下：如果游標超出可視邏輯行範圍，就把 offset 拉上來
    if cursor.row >= self.offset_row + effective_rows {
        // +1 是讓游標出現在畫面最下方
        self.offset_row = cursor.row + 1 - effective_rows;
    }
}
```

特性：

* 複雜度變成 **O(1)**，完全不掃那些大範圍的行。
* 捲動邏輯改成「一行算一行」，不管折行高度。

  * 對長行會有一點「視覺上位置不完全貼齊」的現象，但絕對比卡死還好很多。

這是最快能止血的改法，建議你先試這個版看體感。

---

### 2. 如果你想保留「視覺行精準定位」：改成 O(D) 而不是 O(D²)

現在最致命的是這一段：

```rust
let visual_offset =
    self.count_visual_lines(buffer, self.offset_row, cursor.row) + cursor.visual_line_index;

if visual_offset >= effective_rows {
    while self.offset_row < cursor.row {
        self.offset_row += 1;
        let new_offset = self.count_visual_lines(buffer, self.offset_row, cursor.row)
            + cursor.visual_line_index;
        if new_offset < effective_rows {
            break;
        }
    }
}
```

可以改成 **一次算完，再「累減」而不是每次重算整段**：

```rust
fn scroll_if_needed(
    &mut self,
    cursor: &Cursor,
    buffer: &RopeBuffer,
    has_debug_ruler: bool,
) {
    // 向上滾動
    if cursor.row < self.offset_row {
        self.offset_row = cursor.row;
        return;
    }

    let effective_rows = self.get_effective_screen_rows(has_debug_ruler);

    // 先算目前 offset_row ~ cursor.row 的視覺高度
    let mut visual_offset =
        self.count_visual_lines(buffer, self.offset_row, cursor.row) + cursor.visual_line_index;

    // 如果沒超出螢幕，就不用動
    if visual_offset < effective_rows {
        return;
    }

    // 向下推 offset_row，每次只扣掉「剛剛那一行」的視覺高度
    while self.offset_row < cursor.row && visual_offset >= effective_rows {
        let height_of_first_row =
            self.calculate_visual_lines_for_row(buffer, self.offset_row).len();

        self.offset_row += 1;

        // 防止 underflow
        if visual_offset > height_of_first_row {
            visual_offset -= height_of_first_row;
        } else {
            visual_offset = 0;
        }
    }
}
```

這樣複雜度就變成：

* 一開始 `count_visual_lines`：O(D)
* while 迴圈每次只算一行 `calculate_visual_lines_for_row`：最多 D 次 → O(D)
* 總共 **O(D)**，不會再 O(D²)

---

## 三、其它可以明顯優化的地方（中期改版）

### 1. 每一 frame 中同一行被重複 `to_string` + 重複計算

在 `render()` 裡你有多個地方對同一行做類似操作，例如：

* 主體渲染：

```rust
if let Some(line) = buffer.line(row) {
    let line_str = line.to_string();
    let line_str = line_str.trim_end_matches(['\n', '\r']);
    // 展開 tab → displayed_line
    // wrap_line(...)
}
```

* 計算游標 `cursor_screen_y` 的時候，又來一次：

```rust
for row in self.offset_row..cursor.row {
    if let Some(line) = buffer.line(row) {
        let line_str = line.to_string();
        let line_str = line_str.trim_end_matches(['\n', '\r']);
        // 展開 tab → displayed_line
        // wrap_line(...)
    }
}
```

* `calculate_visual_lines_for_row` 裡也做一模一樣的事。

**建議：**

* 把「取一行 + 去掉換行 + 展開 tab + wrap」這一套，收成一個 helper：

  * 回傳 `Vec<String>` 或最少 `line_without_crlf: String` + `visual_lines: Vec<VisualLineInfo>`.
* 在一次 `render()` 中，同一個 `row` 最多算一次，其他地方直接重用。

更進階一點，可以在 `View` 裡面做一個 per-row cache：

```rust
struct LineLayout {
    // 例如:
    // original_hash: u64,   // 用來判斷這行有沒有變
    // visual_lines: Vec<String>,
    visual_height: usize,
}

struct View {
    line_layouts: Vec<Option<LineLayout>>,
    // ...
}
```

* 螢幕寬度變了就清 cache。
* 該行被編輯時才清掉那一行的 cache。
* `count_visual_lines` / `cursor_screen_y` / `render` 通通吃這個 cache。

這就可以從「每 frame 都掃 Rope + `to_string` + 折行」變成「只有有變動的行才重算」。

---

### 2. 避免 `Vec<char>` 重複配置

你在 `view.rs` / `editor.rs` 有不少這種：

```rust
let chars: Vec<char> = visual_line.chars().collect();
for (idx, &ch) in chars.iter().enumerate() { ... }
```

這在長行 + 多行的情況下，配置超多 `Vec<char>`。

大多數情況可以改成直接用 iterator，避免每次多配一個 Vec：

```rust
for (idx, ch) in visual_line.chars().enumerate() {
    // ch 已經是 char，直接用就好
}
```

只有真需要「多次隨機 index」的地方才真的用 `Vec<char>`。

---

### 3. rope → `String` 的頻率可以再降

`RopeBuffer::line()` 目前是：

```rust
pub fn line(&self, idx: usize) -> Option<RopeSlice<'_>> {
    if idx < self.line_count() {
        Some(self.rope.line(idx))
    } else {
        None
    }
}
```

然後上層幾乎都是立刻 `to_string()`。

如果你願意多改一點，可以考慮：

* 在 wrap / visual width 計算那層直接吃 `RopeSlice`，避免中間多一層 `String`。
* 或者至少共用 `String`，例如在迴圈裡重用一個可變 `String`，每行 `clear()` 再 `push_str`，不要每行都重新分配。

---

## 四、建議的實作順序

我幫你排個「改一輪就能感受差很多」的順序：

1. **第一步（強烈建議馬上做）：**
   把 `scroll_if_needed` 裡的 `count_visual_lines` + while-loop 版本拿掉，
   先換成「純邏輯行版」或是上面那個 O(D) 版。
   → 這一步就很可能直接解決「大檔跳多頁就卡死」的問題。

2. **第二步：**
   把 `render()` 裡對「同一行」的操作整理一下，避免同一 frame 重複 `to_string` / 展開 tab / wrap。
   至少把 `cursor_screen_y` 那段改成重用 `calculate_visual_lines_for_row` 的結果，不要再算一次。

3. **第三步（需要多一點 refactor）：**
   做 per-row layout cache（visual height / 視覺行列表），只在螢幕寬度變更 / 那一行被編輯時才重建。
   `scroll_if_needed`、`cursor_screen_y`、主渲染都吃這個 cache。
