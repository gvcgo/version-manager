//! 文本表格输出（无 TUI，plan 要求 9）。

/// 按列对齐打印。
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }
    let fmt_row = |cells: &[String]| -> String {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let w = if i + 1 < widths.len() { widths[i] } else { 0 };
                if w == 0 {
                    c.clone()
                } else {
                    format!("{:<w$}", c)
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_string()
    };
    println!(
        "{}",
        fmt_row(&headers.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    );
    println!(
        "{}",
        "-".repeat(widths.iter().sum::<usize>() + (widths.len().saturating_sub(1)) * 2)
    );
    for row in rows {
        println!("{}", fmt_row(row));
    }
}
