use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

const MISTRAL_API_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const DEFAULT_MODEL: &str = "mistral-large-latest";

/// observistral — AI-powered observability using Mistral AI 🐈‍⬛
#[derive(Parser)]
#[command(name = "observistral", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze logs with Mistral AI and surface insights
    Analyze {
        /// Path to a log file (defaults to stdin if not provided)
        #[arg(short, long)]
        file: Option<String>,

        /// Mistral AI model to use
        #[arg(short, long, default_value = DEFAULT_MODEL)]
        model: String,

        /// Custom analysis prompt
        #[arg(short, long)]
        prompt: Option<String>,
    },
}

// ── Mistral API request / response types ──────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn read_input(file: Option<&str>) -> anyhow::Result<String> {
    match file {
        Some(path) => Ok(std::fs::read_to_string(path)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

async fn call_mistral(
    client: &Client,
    api_key: &str,
    model: &str,
    system: &str,
    user: &str,
) -> anyhow::Result<String> {
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user.to_string(),
            },
        ],
    };

    let response = client
        .post(MISTRAL_API_URL)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<ChatResponse>()
        .await?;

    let content = response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    Ok(content)
}

// ── entry-point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let api_key = std::env::var("MISTRAL_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: MISTRAL_API_KEY environment variable is not set.");
        std::process::exit(1);
    });

    let client = Client::new();

    match &cli.command {
        Commands::Analyze { file, model, prompt } => {
            let input = read_input(file.as_deref())?;
            if input.trim().is_empty() {
                eprintln!("Error: no input provided (empty file or stdin).");
                std::process::exit(1);
            }

            let system = "You are an expert site-reliability engineer and observability \
                specialist. When given log output or metrics, you identify errors, \
                anomalies, performance bottlenecks, and actionable remediation steps. \
                Be concise and structured.";

            let user = match prompt {
                Some(p) => format!("{}\n\n{}", p, input),
                None => format!(
                    "Analyze the following log output and provide:\n\
                     1. A brief summary of what is happening\n\
                     2. Any errors or anomalies detected\n\
                     3. Potential root causes\n\
                     4. Recommended remediation steps\n\n\
                     Log output:\n{}",
                    input
                ),
            };

            println!("🔍 Analyzing with {}…\n", model);
            let answer = call_mistral(&client, &api_key, model, system, &user).await?;
            println!("{}", answer);
        }
    }

    Ok(())
}

