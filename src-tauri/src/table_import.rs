use calamine::{Reader, Xlsx};
use serde_json::{json, Value};
use std::io::Cursor;

fn rows_to_json(rows: Vec<Vec<String>>) -> Value {
    if rows.is_empty() {
        return json!({"headers": [], "rows": []});
    }
    let headers = rows.first().cloned().unwrap_or_default();
    let data = rows.into_iter().skip(1).filter(|row| row.iter().any(|cell| !cell.trim().is_empty())).collect::<Vec<_>>();
    json!({"headers": headers, "rows": data})
}

fn parse_delimited(bytes: &[u8], filename: &str) -> Result<Value, String> {
    let text = String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned());
    let lower = filename.to_ascii_lowercase();
    let delimiter = if lower.ends_with(".tsv") || text.chars().take(4096).filter(|c| *c == '\t').count() > text.chars().take(4096).filter(|c| *c == ',').count() {
        b'\t'
    } else if text.chars().take(4096).filter(|c| *c == ';').count() > text.chars().take(4096).filter(|c| *c == ',').count() {
        b';'
    } else {
        b','
    };
    let mut reader = csv::ReaderBuilder::new().delimiter(delimiter).flexible(true).has_headers(false).from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| format!("Could not read table: {e}"))?;
        rows.push(record.iter().map(|value| value.trim().to_owned()).collect());
    }
    Ok(rows_to_json(rows))
}

fn parse_xlsx(bytes: Vec<u8>) -> Result<Value, String> {
    let mut workbook: Xlsx<Cursor<Vec<u8>>> = Xlsx::new(Cursor::new(bytes)).map_err(|e| format!("Could not open XLSX: {e}"))?;
    let sheet = workbook.sheet_names().first().cloned().ok_or_else(|| "The XLSX workbook has no sheets.".to_string())?;
    let range = workbook.worksheet_range(&sheet).map_err(|e| format!("Could not read XLSX sheet {sheet}: {e}"))?;
    let rows = range.rows().map(|row| row.iter().map(|cell| cell.to_string()).collect::<Vec<_>>()).collect::<Vec<_>>();
    Ok(rows_to_json(rows))
}

#[tauri::command]
pub fn import_table(filename: String, bytes: Vec<u8>) -> Result<Value, String> {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".xlsx") {
        parse_xlsx(bytes)
    } else if lower.ends_with(".csv") || lower.ends_with(".tsv") || lower.ends_with(".txt") {
        parse_delimited(&bytes, &filename)
    } else {
        Err("Choose CSV, TSV, TXT, or XLSX data.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_csv_rows() {
        let value = import_table("cards.csv".into(), b"Title,Badge\nOne,1\nTwo,2\n".to_vec()).unwrap();
        assert_eq!(value["headers"][0], "Title");
        assert_eq!(value["rows"].as_array().unwrap().len(), 2);
    }
}
