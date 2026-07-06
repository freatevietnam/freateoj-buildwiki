mod build;
mod db;
mod svg;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "freateoj-wiki")]
#[command(about = "Static wiki builder - generates HTML from SQLite via Markdown")]
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
    /// Build static HTML output
    Build {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
        /// Output directory for static files
        #[arg(short, long, default_value = "build")]
        output: String,
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
        None => {
            // Default: build with defaults
            db::init_db("wiki.db")?;
            let conn = db::get_conn("wiki.db")?;
            let settings = db::get_all_settings(&conn)?;
            let sections = db::get_all_sections(&conn)?;
            let pages = db::get_all_pages(&conn)?;
            build::build(&settings, &sections, &pages, "build")?;
            println!("Build complete! Output: build/");
        }
    }

    Ok(())
}
