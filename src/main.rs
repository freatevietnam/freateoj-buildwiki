mod build;
mod db;
mod server;
mod svg;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "freateoj-wiki")]
#[command(about = "FreateOJ Wiki Builder - Build and serve a wiki with web admin interface")]
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
        /// Rebuild after changes (watch mode)
        #[arg(short, long)]
        watch: bool,
    },
    /// Start web server with admin interface
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build { db, output, watch: _ }) => {
            println!("Building wiki...");
            db::init_db(&db)?;
            let conn = db::get_conn(&db)?;
            let settings = db::get_all_settings(&conn)?;
            let sections = db::get_all_sections(&conn)?;
            let pages = db::get_all_pages(&conn)?;
            build::build(&settings, &sections, &pages, &output)?;
            println!("Build complete! Output: {}", output);
        }
        Some(Commands::Serve { db, output, port }) => {
            println!("Initializing...");
            db::init_db(&db)?;
            {
                let conn = db::get_conn(&db)?;
                let settings = db::get_all_settings(&conn)?;
                let sections = db::get_all_sections(&conn)?;
                let pages = db::get_all_pages(&conn)?;
                if let Err(e) = build::build(&settings, &sections, &pages, &output) {
                    eprintln!("Initial build warning: {}", e);
                }
            }
            server::serve(&db, &output, port)?;
        }
        None => {
            // Default: serve with defaults
            println!("Starting default server...");
            db::init_db("wiki.db")?;
            {
                let conn = db::get_conn("wiki.db")?;
                let settings = db::get_all_settings(&conn)?;
                let sections = db::get_all_sections(&conn)?;
                let pages = db::get_all_pages(&conn)?;
                if let Err(e) = build::build(&settings, &sections, &pages, "build") {
                    eprintln!("Initial build warning: {}", e);
                }
            }
            server::serve("wiki.db", "build", 8080)?;
        }
    }

    Ok(())
}
