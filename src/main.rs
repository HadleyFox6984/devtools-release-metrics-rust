mod infrai;

use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct ReleaseInput { build_id: String, release_id: String, diagnostics: Vec<String> }

fn release_metric(input: &ReleaseInput) -> (&'static str, f64) {
    if input.diagnostics.is_empty() { ("release.completed", 1.0) } else { ("release.blocked", 1.0) }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = ReleaseInput { build_id: "build-42".into(), release_id: "release-42".into(), diagnostics: Vec::new() };
    let (metric, value) = release_metric(&input);
    infrai::metrics::report(metric, value, "counter", json!({"build_id": input.build_id, "release_id": input.release_id}), "release-42-metric").await?;
    if !input.diagnostics.is_empty() {
        infrai::errors::capture("release diagnostics require attention", &input.diagnostics.join("; "), "release-42-error").await?;
    }
    println!("{metric}=reported");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_block_release() {
        let input = ReleaseInput { build_id: "b".into(), release_id: "r".into(), diagnostics: vec!["lint".into()] };
        assert_eq!(release_metric(&input), ("release.blocked", 1.0));
    }
}
