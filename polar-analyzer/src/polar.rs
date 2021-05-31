use polar_core::{
    error::{ErrorKind, ParseError},
    formatting::ToPolarString,
    kb::KnowledgeBase,
    parser::Line,
    polar,
    rules::Rule,
    sources::{self, SourceInfo},
    visitor::Visitor,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::JsResult;

#[wasm_bindgen]
pub struct Polar(polar::Polar);

type ErrorData = (String, usize, usize);
type UnusedRule = (String, usize, usize);

#[derive(Default, Deserialize, Serialize)]
pub struct ParseErrors {
    errors: Vec<ErrorData>,
    unused_rules: Vec<UnusedRule>,
}

fn err_string(e: impl std::error::Error) -> JsValue {
    e.to_string().into()
}

#[wasm_bindgen]
impl Polar {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        console_error_panic_hook::set_once();
        Self(polar::Polar::new())
    }

    fn find_parse_errors(&self, src: &str) -> Vec<ErrorData> {
        let parse_result = polar_core::parser::parse_file_with_errors(0, src);

        match parse_result {
            Ok((_, errors)) => errors,
            Err(e) => match e.kind {
                ErrorKind::Parse(e) => match e {
                    ParseError::IntegerOverflow { loc, .. }
                    | ParseError::InvalidTokenCharacter { loc, .. }
                    | ParseError::InvalidToken { loc, .. }
                    | ParseError::UnrecognizedEOF { loc }
                    | ParseError::UnrecognizedToken { loc, .. }
                    | ParseError::ExtraToken { loc, .. }
                    | ParseError::WrongValueType { loc, .. }
                    | ParseError::ReservedWord { loc, .. } => {
                        vec![(e.to_string(), loc, loc)]
                    }
                    _ => {
                        vec![(e.to_string(), 0, 0)]
                    }
                },
                _ => vec![(e.to_string(), 0, 0)],
            },
        }
    }

    fn find_unused_rules(&self, src: &str) -> Vec<UnusedRule> {
        let parse_result = polar_core::parser::parse_file_with_errors(0, src);
        let kb = self.0.kb.read().expect("failed to get lock on KB");

        struct UnusedRuleVisitor<'kb> {
            unused_rules: Vec<UnusedRule>,
            kb: &'kb KnowledgeBase,
        }

        impl<'kb> Visitor for UnusedRuleVisitor<'kb> {
            fn visit_term(&mut self, t: &polar_core::terms::Term) {
                match t.value() {
                    polar_core::terms::Value::Call(c) => {
                        if let Some(rules) = self.kb.rules.get(&c.name) {
                            crate::log(&format!(
                                "{} in {:#?}",
                                c.to_polar(),
                                rules
                                    .rules
                                    .iter()
                                    .map(|(_, r)| r.to_polar())
                                    .collect::<Vec<String>>()
                            ));
                            if rules.get_applicable_rules(&c.args).is_empty() {
                                let (left, right) = t.span().unwrap_or((0, 0));
                                let message = format!(
                                    r#"There are no rules matching the format:
  {}
Found:
  {}
"#,
                                    c.to_polar(),
                                    rules
                                        .rules
                                        .iter()
                                        .map(|(_, r)| r.to_polar())
                                        .collect::<Vec<String>>()
                                        .join("\n  ")
                                );
                                self.unused_rules.push((message, left, right));
                            }
                        } else {
                            let (left, right) = t.span().unwrap_or((0, 0));
                            let message =
                                format!("There are no rules with the name \"{}\"", c.name);
                            self.unused_rules.push((message, left, right));
                        }
                    }
                    _ => {}
                }
                polar_core::visitor::walk_term(self, t)
            }
        }

        let mut visitor = UnusedRuleVisitor {
            kb: &kb,
            unused_rules: vec![],
        };

        if let Ok((lines, _)) = parse_result {
            for line in lines {
                match line {
                    Line::Rule(r) => {
                        visitor.visit_term(&r.body);
                    }
                    Line::Query(q) => {
                        visitor.visit_term(&q);
                    }
                }
            }
        }

        visitor.unused_rules
    }

    #[wasm_bindgen(js_class = Polar, js_name = load)]
    pub fn wasm_load(&self, src: &str, filename: &str) -> JsResult<JsValue> {
        self.0.remove_file(filename);

        let errors = self.find_parse_errors(src);
        let mut unused_rules = vec![];
        if errors.is_empty() {
            // we'll only be able to actually load the policy if there
            // aren't any parse errors
            self.0
                .load(src, Some(filename.to_string()))
                .map_err(err_string)?; // still want to catch any remaining errors
            unused_rules = self.find_unused_rules(src);
        }
        Ok(serde_wasm_bindgen::to_value(&ParseErrors {
            errors,
            unused_rules,
        })
        .unwrap())
    }

    #[wasm_bindgen(js_class = Polar, js_name = clearRules)]
    pub fn wasm_clear_rules(&self) {
        self.0.clear_rules()
    }

    #[wasm_bindgen(js_class = Polar, js_name = getSummary)]
    pub fn get_summary(&self) -> JsValue {
        // Get a read lock on the KB
        let kb = self.0.kb.read().expect("failed to get lock on KB");

        let get_rule_signature = |r: &Rule| {
            if let sources::SourceInfo::Parser {
                src_id,
                left,
                right,
            } = r.source_info
            {
                let source = kb.sources.get_source(src_id);
                if let Some(source) = source {
                    return source.src.chars().take(right).skip(left).collect();
                }
            }
            format!(
                "{}({})",
                r.name,
                r.params
                    .iter()
                    .map(|p| p.to_polar())
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        };

        let get_rule_location = |r: &Rule| {
            if let SourceInfo::Parser {
                src_id,
                left,
                right,
            } = r.source_info
            {
                let source = kb.sources.get_source(src_id);
                if let Some(source) = source {
                    return (source.filename, left, right);
                }
            }
            (None, 0, 0)
        };

        serde_wasm_bindgen::to_value(&PolicySummary {
            rules: kb
                .rules
                .iter()
                .flat_map(|(name, generic_rule)| {
                    generic_rule.rules.iter().map(move |(_, r)| RuleInfo {
                        symbol: name.0.clone(),
                        signature: get_rule_signature(r),
                        location: get_rule_location(r),
                    })
                })
                .collect(),
        })
        .unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PolicySummary {
    rules: Vec<RuleInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleInfo {
    symbol: String,
    signature: String,
    location: (Option<String>, usize, usize),
}
