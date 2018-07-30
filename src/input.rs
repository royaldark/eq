use std::collections::BTreeMap;

use clap::{_clap_count_exprs, arg_enum};
use edn::parser::Parser;
use edn::Value as EdnValue;
use serde_json;
use serde_json::Result as JsonResult;
use serde_json::Value as JsonValue;

#[derive(Debug)]
crate enum ReadError {
    IOError,
    ParseError,
}

arg_enum!{
    #[derive(Debug)]
    pub enum InputFormat {
        EDN,
        JSON,
    }
}

crate struct InputOptions {
    crate format: InputFormat,
    crate path: String,
}

fn json_to_edn(json: JsonValue) -> EdnValue {
    println!("json: {:?}", json);
    match json {
        JsonValue::Null => EdnValue::Nil,
        JsonValue::Bool(b) => EdnValue::Boolean(b),
        JsonValue::String(s) => EdnValue::String(s),
        JsonValue::Number(n) => {
            if n.is_i64() {
                EdnValue::from(n.as_i64().unwrap())
            } else if n.is_u64() {
                EdnValue::from(n.as_i64().unwrap())
            } else if n.is_f64() {
                EdnValue::from(n.as_f64().unwrap())
            } else {
                unreachable!()
            }
        }
        JsonValue::Object(n) => {
            let mut acc: BTreeMap<EdnValue, EdnValue> = BTreeMap::new();

            for (k, v) in n {
                acc.insert(EdnValue::from(k), json_to_edn(v));
            }

            EdnValue::Map(acc)
        }
        JsonValue::Array(n) => {
            let mut acc: Vec<EdnValue> = Vec::new();

            for item in n {
                acc.push(json_to_edn(item));
            }

            EdnValue::Vector(acc)
        }
    }
}

fn parse_json(contents: &str) -> Result<Vec<EdnValue>, ReadError> {
    let parsed: JsonResult<JsonValue> = serde_json::from_str(contents);
    let mut forms: Vec<EdnValue> = Vec::new();

    match parsed {
        Ok(json) => forms.push(json_to_edn(json)),
        Err(_) => return Err(ReadError::ParseError),
    }

    Ok(forms)
}

fn parse_edn(contents: &str) -> Result<Vec<EdnValue>, ReadError> {
    let mut parser = Parser::new(&contents);
    let mut forms: Vec<EdnValue> = Vec::new();

    while let Some(form) = parser.read() {
        match form {
            Ok(f) => forms.push(f),
            Err(_) => return Err(ReadError::ParseError),
        }
    }

    Ok(forms)
}

crate fn read_file(opts: &InputOptions) -> Result<Vec<EdnValue>, ReadError> {
    let contents = std::fs::read(&opts.path).map_err(|_| ReadError::IOError)?;
    let as_str = String::from_utf8_lossy(&contents);

    Ok(match opts.format {
        InputFormat::JSON => parse_json(&as_str)?,
        InputFormat::EDN => parse_edn(&as_str)?,
    })
}
