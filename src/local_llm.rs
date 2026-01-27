use crate::actions::{Action, Plan};
use crate::context::ContextSnapshot;
use crate::config::Config;
use anyhow::Result;

pub trait LocalPlanner {
    fn plan_actions(&self, ctx: &ContextSnapshot, query: &str, max_tokens: usize) -> Result<Plan>;
}

pub fn build_local_llm(_cfg: &Config) -> Result<Box<dyn LocalPlanner>> {
    // For now, return a heuristic planner that does not require model weights.
    Ok(Box::new(HeuristicPlanner))
}

struct HeuristicPlanner;

impl LocalPlanner for HeuristicPlanner {
    fn plan_actions(&self, ctx: &ContextSnapshot, query: &str, _max_tokens: usize) -> Result<Plan> {
        let mut actions: Vec<Action> = Vec::new();

        // Very naive heuristic: if the query mentions README, propose creating/updating a README.md.
        if query.to_lowercase().contains("readme") {
            actions.push(Action::WriteFile {
                path: "README.md".into(),
                content: format!("# Project\n\nAutomated change requested: {}\n", query),
                create_dirs: false,
            });
        }

        let description = format!(
            "Heuristic plan for query: '{}' with {} files in context.",
            query,
            ctx.files.len()
        );

        Ok(Plan { description, actions })
    }
}

// Future: when the `local-llm` feature is enabled, wire a Candle-backed model here
// to summarize context and propose more sophisticated plans.
