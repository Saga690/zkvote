use clap::{Parser, Subcommand};

mod commands;
mod utils;
mod zk;

#[derive(Parser)]
#[command(name = "chain")]
#[command(about = "Anonymous Voting CLI powered by ZKPs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Register,
    Create {
        #[arg(short, long)]
        question: String,
    },
    Vote {
        #[arg(short, long)]
        proposal_id: u32,
        #[arg(short, long)]
        choice: u8,
    },
    Tally {
        #[arg(short, long)]
        proposal_id: u32,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Register => commands::register::handle_register().await,
        Commands::Create { question } => commands::create::handle_create(question).await,
        Commands::Vote { proposal_id, choice } => commands::vote::handle_vote(proposal_id, choice).await,
        Commands::Tally { proposal_id } => commands::tally::handle_tally(proposal_id).await,
    }
}
