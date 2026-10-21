//! Closed local tool catalogue. No shell, filesystem or network tools.
use crate::MAX_TEXT;
use serde::Deserialize;
use serde_json::{Value, json};

pub const NAMES: [&str; 3] = ["text.word_count", "text.transform", "text.find"];
pub const WIRE_NAMES: [&str; 3] = ["text_word_count", "text_transform", "text_find"];

pub fn logical_name(wire: &str) -> Option<&'static str> {
    WIRE_NAMES.iter().position(|n| *n == wire).map(|i| NAMES[i])
}

pub fn definitions(allowed: &[String]) -> Vec<Value> {
    let string = json!({"type":"string","maxLength":16384});
    let specifications = [
        (
            "Count whitespace-separated words, Unicode characters and UTF-8 bytes in text.",
            json!({"type":"object","properties":{"text":string},"required":["text"],"additionalProperties":false}),
        ),
        (
            "Transform text using uppercase, lowercase or trim; returns the actual transformed text.",
            json!({"type":"object","properties":{"text":string,"operation":{"type":"string","enum":["uppercase","lowercase","trim"]}},"required":["text","operation"],"additionalProperties":false}),
        ),
        (
            "Find case-sensitive, non-overlapping occurrences; returns byte and Unicode character positions.",
            json!({"type":"object","properties":{"text":string,"query":{"type":"string","minLength":1,"maxLength":16384}},"required":["text","query"],"additionalProperties":false}),
        ),
    ];
    specifications.into_iter().enumerate().filter(|(i,_)|allowed.iter().any(|n|n==NAMES[*i])).map(|(i,(description,parameters))|json!({"type":"function","function":{"name":WIRE_NAMES[i],"description":description,"parameters":parameters}})).collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Text {
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Transform {
    text: String,
    operation: Transformation,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Transformation {
    Uppercase,
    Lowercase,
    Trim,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Find {
    text: String,
    query: String,
}

fn bounded(text: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT {
        Err("tool text exceeds 16 KiB".into())
    } else {
        Ok(())
    }
}

pub fn validate(wire: &str, args: &Value) -> Result<(), String> {
    match wire {
        "text_word_count" => {
            let p: Text = serde_json::from_value(args.clone())
                .map_err(|_| "word_count requires only {text: string}")?;
            bounded(&p.text)
        }
        "text_transform" => {
            let p: Transform = serde_json::from_value(args.clone()).map_err(
                |_| "transform requires {text: string, operation: uppercase|lowercase|trim}",
            )?;
            bounded(&p.text)
        }
        "text_find" => {
            let p: Find = serde_json::from_value(args.clone())
                .map_err(|_| "find requires only {text: string, query: nonempty string}")?;
            bounded(&p.text)?;
            bounded(&p.query)?;
            if p.query.is_empty() {
                Err("find query must not be empty".into())
            } else {
                Ok(())
            }
        }
        _ => Err("unknown tool; no function executed".into()),
    }
}

pub fn execute(wire: &str, args: &Value) -> Result<Value, String> {
    validate(wire, args)?;
    match wire {
        "text_word_count" => {
            let p: Text = serde_json::from_value(args.clone()).map_err(|_| "invalid text")?;
            Ok(
                json!({"word_count":p.text.split_whitespace().count(),"character_count":p.text.chars().count(),"utf8_bytes":p.text.len()}),
            )
        }
        "text_transform" => {
            let p: Transform =
                serde_json::from_value(args.clone()).map_err(|_| "invalid transform")?;
            let operation = args["operation"].as_str().unwrap_or_default();
            let text = match p.operation {
                Transformation::Uppercase => p.text.to_uppercase(),
                Transformation::Lowercase => p.text.to_lowercase(),
                Transformation::Trim => p.text.trim().to_owned(),
            };
            bounded(&text)?;
            Ok(json!({"operation":operation,"text":text}))
        }
        "text_find" => {
            let p: Find = serde_json::from_value(args.clone()).map_err(|_| "invalid find")?;
            let mut count = 0;
            let mut positions = Vec::new();
            for (byte, _) in p.text.match_indices(&p.query) {
                count += 1;
                if positions.len() < 128 {
                    positions.push(
                        json!({"byte_index":byte,"character_index":p.text[..byte].chars().count()}),
                    );
                }
            }
            Ok(
                json!({"found":count>0,"match_count":count,"positions":positions,"positions_truncated":count>128}),
            )
        }
        _ => Err("unknown tool".into()),
    }
}
