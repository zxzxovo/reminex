use clap::{Parser, Subcommand};

fn main() {
    let app = App::parse();
    match app.commands {
        Commands::Index | Commands::I => {

        },

        Commands::Search | Commands::S => {

        }
    }
}

#[derive(Parser)]
struct App {

    #[arg(help="Input the path of the database, or it will default use './' as position.")]
    db: Option<String>,
    
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {

    #[command(about="Start indexing files.")]
    Index,

    #[command(about="Start indexing files.")]
    I,

    #[command(about="Search through database.")]
    Search,

    #[command(about="Search through database.")]
    S,
}