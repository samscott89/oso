use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use polar_core::{
    kb::KnowledgeBase,
    rules::Rule,
    sources::SourceInfo,
    terms::{Symbol, ToPolarString},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct RuleInfo {
    symbol: String,
    signature: String,
    body: HashSet<Symbol>,
    location: (Option<String>, usize, usize),
}

/// Get the string formatted signature of the rule
///
/// Either uses the source directly if it's available
/// (should usually be the case). Otherwise, construct it
/// from the name and parameters.
fn get_rule_signature(kb: &KnowledgeBase, r: &Rule) -> String {
    if let SourceInfo::Parser {
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
}

/// Get the location of the rule
fn get_rule_location(kb: &KnowledgeBase, r: &Rule) -> (Option<String>, usize, usize) {
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
}

pub fn get_rule_info(kb: &KnowledgeBase) -> Vec<RuleInfo> {
    let get_term_value = |r: &Rule| {
        let mut variable_set: HashSet<Symbol> = HashSet::new();
        r.body.variables(&mut variable_set);
        variable_set
    };

    kb.rules
        .iter()
        .flat_map(|(name, generic_rule)| {
            generic_rule.rules.iter().map(move |(_, r)| RuleInfo {
                symbol: name.0.clone(),
                signature: get_rule_signature(kb, r),
                body: get_term_value(r),
                location: get_rule_location(kb, r),
            })
        })
        .collect()
}
