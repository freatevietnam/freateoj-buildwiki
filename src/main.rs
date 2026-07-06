mod build;
mod db;
mod gui;
mod server;
mod svg;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "freateoj-wiki")]
#[command(about = "FreateOJ Wiki Builder - Static wiki with GUI, SQLite, and HTML output")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build static HTML output
    Build {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,
    },
    /// Start preview server
    Serve {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,
        /// Port number
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Launch GUI
    Gui {
        /// Database path
        #[arg(short, long, default_value = "wiki.db")]
        db: String,
        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build { db, output }) => {
            println!("Building wiki...");
            db::init_db(&db)?;
            let conn = db::get_conn(&db)?;
            let settings = db::get_all_settings(&conn)?;
            let sections = db::get_all_sections(&conn)?;
            let pages = db::get_all_pages(&conn)?;
            build::build(&settings, &sections, &pages, &output)?;
            println!("Build complete!");
        }
        Some(Commands::Serve { db, output, port }) => {
            println!("Building and serving...");
            db::init_db(&db)?;
            let conn = db::get_conn(&db)?;
            let settings = db::get_all_settings(&conn)?;
            let sections = db::get_all_sections(&conn)?;
            let pages = db::get_all_pages(&conn)?;
            build::build(&settings, &sections, &pages, &output)?;
            println!("Starting server on port {}...", port);
            server::serve(&output, port)?;
        }
        Some(Commands::Gui { db, output }) => {
            gui::run_gui(&db, &output)?;
        }
        None => {
            // Default: launch GUI
            db::init_db("wiki.db")?;
            gui::run_gui("wiki.db", "build")?;
        }
    }

    Ok(())
}
