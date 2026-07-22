use anyhow::Result;
use std::path::Path;

pub fn read_text(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

pub fn read_csv(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let headers = reader
        .headers()?
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>();
    let mut out = String::new();
    out.push_str(&headers.join(","));
    out.push('\n');
    for result in reader.records() {
        let record = result?;
        out.push_str(
            &record
                .iter()
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    Ok(out)
}

pub fn read_json(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&content)?;
    Ok(serde_json::to_string_pretty(&v)?)
}

pub fn write_text(path: &Path, data: &str) -> Result<()> {
    Ok(std::fs::write(path, data)?)
}

pub fn write_csv(path: &Path, data: &str) -> Result<()> {
    Ok(std::fs::write(path, data)?)
}

pub fn write_json(path: &Path, data: &str) -> Result<()> {
    let _: serde_json::Value = serde_json::from_str(data)?;
    Ok(std::fs::write(path, data)?)
}

pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

pub fn list_directory(path: &Path) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        entries.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_write_text() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        write_text(&path, "hello world").unwrap();
        let content = read_text(&path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_read_csv() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.csv");
        write_csv(&path, "a,b,c\n1,2,3\n4,5,6").unwrap();
        let content = read_csv(&path).unwrap();
        assert!(content.contains("a,b,c"));
        assert!(content.contains("1,2,3"));
    }

    #[test]
    fn test_file_exists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exists.txt");
        assert!(!file_exists(&path));
        write_text(&path, "test").unwrap();
        assert!(file_exists(&path));
    }

    #[test]
    fn test_list_directory() {
        let dir = TempDir::new().unwrap();
        write_text(&dir.path().join("a.txt"), "a").unwrap();
        write_text(&dir.path().join("b.txt"), "b").unwrap();
        let entries = list_directory(dir.path()).unwrap();
        assert!(entries.contains(&"a.txt".to_string()));
        assert!(entries.contains(&"b.txt".to_string()));
    }
}
