pub mod prompts;

use anyhow::Result;
use futures::StreamExt;

pub fn default_model() -> String {
    std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "qwen2.5-coder:7b".to_string())
}

pub fn detect_ollama() -> bool {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return false,
    };
    rt.block_on(async {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };
        client
            .get("http://localhost:11434/api/tags")
            .send()
            .await
            .ok()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    })
}

pub async fn generate(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "prompt": user_prompt,
        "system": system_prompt,
        "stream": false,
    });
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()
        .await?;
    let val: serde_json::Value = response.json().await?;
    Ok(val["response"].as_str().unwrap_or("").to_string())
}

pub async fn generate_stream(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    mut on_token: impl FnMut(String),
) -> Result<String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "prompt": user_prompt,
        "system": system_prompt,
        "stream": true,
    });
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut buf = String::new();
    let mut full_response = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        buf.push_str(&chunk_str);

        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].to_string();
            buf = buf[pos + 1..].to_string();
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(token) = val.get("response").and_then(|v| v.as_str()) {
                    on_token(token.to_string());
                    full_response.push_str(token);
                }
            }
        }
    }

    Ok(full_response)
}

pub async fn list_models() -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await?;
    let val: serde_json::Value = response.json().await?;
    let models = val["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model() {
        let model = default_model();
        assert_eq!(model, "qwen2.5-coder:7b");
    }

    #[test]
    fn test_detect_ollama() {
        // Verify the function exists and can be called
        let _ = detect_ollama();
    }
}
