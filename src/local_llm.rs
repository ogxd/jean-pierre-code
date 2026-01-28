use crate::actions::{Action, Plan};
use crate::config::Config;
use crate::context::{self, ContextSnapshot};
use anyhow::{Context as _, Result};
use log::warn;

pub trait LocalPlanner {
    fn plan_actions(&self, ctx: &ContextSnapshot, query: &str, max_tokens: usize) -> Result<Plan>;
}

pub fn build_local_llm(_cfg: &Config) -> Result<Box<dyn LocalPlanner>> {
    // Always build a Kalosm-backed planner. If runtime/model init fails during inference,
    // the planner will gracefully fall back to a heuristic plan for that request.
    Ok(Box::new(KalosmPlanner))
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

// Local LLM planner using Kalosm (Llama). Always enabled.
// We keep the planner interface synchronous by spinning up a Tokio runtime per call.
// On any error (runtime/model/init/stream), we log a warning and fall back to a heuristic plan.
struct KalosmPlanner;

impl LocalPlanner for KalosmPlanner {
    fn plan_actions(&self, ctx: &ContextSnapshot, query: &str, max_tokens: usize) -> Result<Plan> {
        let ctx_txt = context::truncate_for_prompt(ctx, 40_000);
        let prompt = build_planner_prompt(query, &ctx_txt);

        // Create a small Tokio runtime for the async Kalosm call.
        // If creation fails, fall back to heuristic.
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                warn!("tokio runtime init failed: {} — falling back to heuristic planner", e);
                return HeuristicPlanner.plan_actions(ctx, query, max_tokens);
            }
        };

        let result: Result<String> = rt.block_on(async move {
            use futures_util::StreamExt;
            use kalosm::language::Llama;

            let mut llm = Llama::new().await.context("initializing Kalosm Llama model")?;

            // Feed the prompt. Kalosm's example shows `llm(prompt)` returns a stream.
            let mut stream = llm(prompt.as_str());

            // Collect streamed tokens into a String until we reach an approximate token budget.
            // If the stream type isn't a String, we convert tokens via `to_string()`.
            let mut out = String::new();
            let mut approx_tokens = 0usize;
            while let Some(tok) = stream.next().await {
                let t = tok.to_string();
                out.push_str(&t);
                approx_tokens += 1;
                if approx_tokens >= max_tokens { break; }
            }
            Ok(out)
        });

        match result {
            Ok(text) => {
                if let Some(plan) = parse_plan_from_text(&text) {
                    Ok(plan)
                } else {
                    warn!("Failed to parse JSON plan from Kalosm output. Falling back to heuristic actions.");
                    let mut plan = HeuristicPlanner.plan_actions(ctx, query, max_tokens)?;
                    plan.description = text;
                    Ok(plan)
                }
            }
            Err(e) => {
                warn!("Kalosm generation error: {} — using heuristic planner", e);
                HeuristicPlanner.plan_actions(ctx, query, max_tokens)
            }
        }
    }
}

fn build_planner_prompt(query: &str, ctx_txt: &str) -> String {
    // Keep the system instruction compact yet strict about JSON.
    format!(
        "You are Jean-Pierre, a precise code planning assistant.\n\
         Given the user's request and the project context, produce a plan strictly as a JSON object\n\
         with this schema and only this JSON as output (no markdown, no prose):\n\
         {{\n  \"description\": \"<one sentence>\",\n  \"actions\": [\n    {{ \"type\": \"write_file\", \"path\": \"<string>\", \"content\": \"<string>\", \"create_dirs\": <bool> }},\n    {{ \"type\": \"run\", \"cmd\": \"<string>\", \"args\": [\"<string>\"] }}\n  ]\n}}\n\
         Only include fields shown above. If no actions are necessary, use an empty array.\n\
         User request: \n{}\n\
         Project context (truncated):\n{}",
        query,
        ctx_txt
    )
}

fn parse_plan_from_text(text: &str) -> Option<Plan> {
    // 1) Try direct parse.
    if let Ok(plan) = serde_json::from_str::<Plan>(text) {
        return Some(plan);
    }
    // 2) Try to extract the largest JSON object substring.
    let bytes = text.as_bytes();
    let mut best: Option<(usize, usize)> = None;
    for i in 0..bytes.len() {
        if bytes[i] == b'{' {
            let mut depth = 0_i32;
            for j in i..bytes.len() {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            best = Some((i, j + 1));
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if best.is_some() { break; }
        }
    }
    if let Some((s, e)) = best {
        let slice = &text[s..e];
        if let Ok(plan) = serde_json::from_str::<Plan>(slice) {
            return Some(plan);
        }
    }
    None
}

// Future: when the `local-llm` feature is enabled and Candle is integrated, this module can
// switch from calling `ollama` to a fully in-process Candle-backed model using GGUF weights.
