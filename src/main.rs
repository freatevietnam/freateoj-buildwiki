mod build;
mod db;
mod server;
mod svg;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "freateoj-wiki")]
#[command(about = "Static wiki builder - edit, build, and preview locally")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wiki database with defaults
    Init {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
    },
    /// Build static HTML output (no server)
    Build {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
        /// Output directory for static files
        #[arg(short, long, default_value = "build")]
        output: String,
    },
    /// Start dev server with web editor + live preview
    Dev {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
        /// Output directory for static files
        #[arg(short, long, default_value = "build")]
        output: String,
        /// Port number
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { db }) => {
            db::init_db(&db)?;
            println!("Initialized database: {}", db);
        }
        Some(Commands::Build { db, output }) => {
            db::init_db(&db)?;
            let conn = db::get_conn(&db)?;
            let settings = db::get_all_settings(&conn)?;
            let sections = db::get_all_sections(&conn)?;
            let pages = db::get_all_pages(&conn)?;
            build::build(&settings, &sections, &pages, &output)?;
            println!("Build complete! Output: {}", output);
        }
        Some(Commands::Dev { db, output, port }) => {
            db::init_db(&db)?;
            server::serve(&db, &output, port)?;
        }
        None => {
            // Default: dev server
            db::init_db("wiki.db")?;
            server::serve("wiki.db", "build", 8080)?;
        }
    }

    Ok(())
}
