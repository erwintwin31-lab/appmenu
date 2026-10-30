use serde_json::{json, Value};
use std::collections::HashSet;

/// Conservative local-model recommendation.
/// These are provisional memory budgets, not measured performance guarantees.
pub fn recommend_model(available_ram_mb: Option<u64>, installed: &[String]) -> Value {
    let Some(ram) = available_ram_mb else {
        return json!({
            "model": null,
            "reason": "Available memory could not be determined. Select a model manually."
        });
    };

    let model = match ram {
        0..=2047 => {
            return json!({
                "model": null,
                "reason": "Not enough free memory for a conservative recommendation."
            })
        }
        2048..=4095 => "qwen3:0.6b",
        4096..=12287 => "qwen3:1.7b",
        _ => "qwen3:4b",
    };

    let installed = installed.iter().any(|name| name == model);

    json!({
        "model": model,
        "installed": installed,
        "requires_download": !installed,
        "reason": "Provisional recommendation based on available system memory. Speed and model quality have not been evaluated.",
        "gpu_verified": false,
        "automatically_selected": false
    })
}

/// Parse and validate the model list returned by Ollama's local /api/tags endpoint.
pub fn parse_ollama_tags(body: &str) -> Result<Vec<String>, String> {
    if body.len() > 1_048_576 {
        return Err("Ollama response is too large.".into());
    }

    let value: Value = serde_json::from_str(body).map_err(|_| "Invalid Ollama JSON.")?;

    let models = value
        .get("models")
        .and_then(Value::as_array)
        .ok_or("Ollama response must contain a models array.")?;

    if models.len() > 1000 {
        return Err("Too many models in Ollama response.".into());
    }

    let mut names = Vec::new();

    for model in models {
        let name = model
            .get("name")
            .and_then(Value::as_str)
            .ok_or("Ollama model is missing its name.")?;

        if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
            return Err("Invalid Ollama model name.".into());
        }

        if !names.iter().any(|existing| existing == name) {
            names.push(name.to_owned());
        }
    }

    Ok(names)
}

/// A source fetched by Appmenu itself. Never construct this from model output.
#[derive(Debug, Clone)]
pub struct TrustedSource {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub text: String,
}

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Validate literal quotations against sources fetched by Appmenu.
///
/// This checks provenance only. It does not verify the semantic correctness of
/// generated prose, and it deliberately rejects free-form summaries.
pub fn validate_quotes(raw: &str, sources: &[TrustedSource]) -> Result<Value, String> {
    if raw.len() > 65_536 || sources.is_empty() || sources.len() > 16 {
        return Err("Invalid response or source count.".into());
    }

    let mut ids = HashSet::new();

    for source in sources {
        if !ids.insert(source.id) || source.text.is_empty() || source.text.len() > 200_000 {
            return Err("Invalid or duplicate trusted source.".into());
        }

        if !source.url.starts_with("https://") || source.url.chars().any(char::is_control) {
            return Err("Trusted source must use an HTTPS URL.".into());
        }
    }

    let value: Value = serde_json::from_str(raw).map_err(|_| "Invalid overview JSON.")?;

    let object = value.as_object().ok_or("Expected an overview object.")?;

    if object.len() != 1 || !object.contains_key("quotes") {
        return Err(
            "Only the quotes field is accepted. Generated summaries are not verified.".into(),
        );
    }

    let quotes = object["quotes"]
        .as_array()
        .ok_or("Expected a quotes array.")?;

    if quotes.is_empty() || quotes.len() > 10 {
        return Err("Expected between one and ten quotations.".into());
    }

    let mut verified = Vec::new();

    for item in quotes {
        let fields = item.as_object().ok_or("Expected a quotation object.")?;

        if fields.len() != 2 || !fields.contains_key("source_id") || !fields.contains_key("quote") {
            return Err("Quotation must contain only source_id and quote.".into());
        }

        let id = fields["source_id"]
            .as_u64()
            .and_then(|id| u32::try_from(id).ok())
            .ok_or("Invalid source ID.")?;

        let quote = fields["quote"].as_str().ok_or("Invalid quotation text.")?;

        if quote.trim().chars().count() < 12 || quote.len() > 2000 {
            return Err("Quotation is empty, too short or too long.".into());
        }

        let source = sources
            .iter()
            .find(|source| source.id == id)
            .ok_or("Unknown source ID.")?;

        if !normalized(&source.text).contains(&normalized(quote)) {
            return Err("Quotation does not occur in the fetched source.".into());
        }

        verified.push(json!({
            "source_id": id,
            "quote": quote,
            "title": source.title,
            "url": source.url,
            "quotation_verified": true,
            "claim_verified": false
        }));
    }

    Ok(json!({
        "quotes": verified,
        "experimental": true,
        "semantic_validation": false
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> TrustedSource {
        TrustedSource {
            id: 1,
            title: "Original title".into(),
            url: "https://example.com/source".into(),
            text: "Appmenu does not automatically install models.".into(),
        }
    }

    #[test]
    fn recommends_only_within_memory_budget() {
        assert_eq!(recommend_model(Some(2048), &[])["model"], "qwen3:0.6b");
        assert_eq!(recommend_model(Some(4095), &[])["model"], "qwen3:0.6b");
        assert_eq!(recommend_model(Some(4096), &[])["model"], "qwen3:1.7b");
        assert_eq!(recommend_model(Some(8192), &[])["model"], "qwen3:1.7b");
        assert_eq!(recommend_model(Some(12287), &[])["model"], "qwen3:1.7b");
        assert_eq!(recommend_model(Some(12288), &[])["model"], "qwen3:4b");
        assert!(recommend_model(None, &[])["model"].is_null());
        assert!(recommend_model(Some(2047), &[])["model"].is_null());
    }

    #[test]
    fn installed_model_does_not_override_memory_budget() {
        let result = recommend_model(Some(4096), &["qwen3:32b".to_string()]);
        assert_eq!(result["model"], "qwen3:1.7b");
        assert_eq!(result["automatically_selected"], false);
        let result = recommend_model(Some(4096), &["qwen3:1.7b".to_string()]);
        assert_eq!(result["requires_download"], false);
    }

    #[test]
    fn parses_and_deduplicates_ollama_models() {
        let raw = json!({
            "models": [{"name": "qwen3:1.7b"}, {"name": "qwen3:1.7b"}]
        })
        .to_string();
        assert_eq!(
            parse_ollama_tags(&raw).unwrap(),
            vec!["qwen3:1.7b".to_string()]
        );
        assert!(parse_ollama_tags("not json").is_err());
        assert!(parse_ollama_tags("{}").is_err());
        let missing_name = json!({"models": [{}]}).to_string();
        assert!(parse_ollama_tags(&missing_name).is_err());
    }

    #[test]
    fn validates_literal_quote_against_trusted_source() {
        let raw = json!({
            "quotes": [{"source_id": 1, "quote": "does not automatically install models."}]
        })
        .to_string();
        let result = validate_quotes(&raw, &[source()]).unwrap();
        assert_eq!(result["quotes"][0]["title"], "Original title");
        assert_eq!(result["quotes"][0]["claim_verified"], false);
        assert_eq!(result["semantic_validation"], false);
    }

    #[test]
    fn rejects_fabricated_quote_and_unknown_source() {
        let fabricated = json!({
            "quotes": [{"source_id": 1, "quote": "Appmenu automatically installs every model."}]
        })
        .to_string();
        assert!(validate_quotes(&fabricated, &[source()]).is_err());
        let unknown = json!({
            "quotes": [{"source_id": 99, "quote": "does not automatically install models."}]
        })
        .to_string();
        assert!(validate_quotes(&unknown, &[source()]).is_err());
    }

    #[test]
    fn rejects_summary_and_model_supplied_metadata() {
        let summary = json!({"summary": "Invented claim", "quotes": []}).to_string();
        assert!(validate_quotes(&summary, &[source()]).is_err());
        let extra_field = json!({
            "quotes": [{
                "source_id": 1,
                "quote": "does not automatically install models.",
                "url": "https://wrong.example"
            }]
        })
        .to_string();
        assert!(validate_quotes(&extra_field, &[source()]).is_err());
    }

    #[test]
    fn rejects_empty_output_and_duplicate_sources() {
        let empty = json!({"quotes": []}).to_string();
        assert!(validate_quotes(&empty, &[source()]).is_err());
        let sources = vec![source(), source()];
        let raw = json!({
            "quotes": [{"source_id": 1, "quote": "does not automatically install models."}]
        })
        .to_string();
        assert!(validate_quotes(&raw, &sources).is_err());
    }
}
