use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "jean-pierre-code", about = "CLI agentic AI code assistant")] 
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize configuration files
    Init { 
        /// Overwrite existing config if present
        #[arg(long)]
        force: bool,
    },

    /// Print gathered project context as JSON
    Context {
        /// Max number of files to include
        #[arg(long)]
        max_files: Option<usize>,
        /// Max total bytes to include
        #[arg(long)]
        max_bytes: Option<usize>,
    },

    /// Create an action plan for a query, using local LLM
    Plan {
        /// The user query (what to change/build/test)
        #[arg()]
        query: String,
        /// Token/length limit for the plan
        #[arg(long)]
        max_tokens: Option<usize>,
    },

    /// Apply a JSON plan file of actions (writes, commands)
    Apply {
        /// Path to the JSON plan file
        #[arg()]
        plan_file: String,
        /// Do not actually perform changes, just show
        #[arg(long)]
        dry_run: bool,
    },

    /// Lightweight chat with remote model (context-aware)
    Chat {
        /// Prompt to send
        #[arg()]
        prompt: String,
    },

    /// Run helper commands (build, test, or any shell command)
    Run {
        /// What to run: "build", "test" or an arbitrary program
        #[arg()]
        what: String,
    },
}
