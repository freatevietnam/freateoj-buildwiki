# FreateOJ Wiki Builder (Rust)

Single-file wiki builder with GUI, SQLite database, and static HTML output.

## Features

- **Cross-platform** — runs on Linux, macOS, and Windows
- **SQLite database** — pages, sections, SEO settings stored in `wiki.db`
- **Static HTML output** — generates `build/` with clean, fast pages
- **Desktop GUI** — egui-based interface with 6 tabs
- **Per-page SEO** — OG images, Twitter cards, robots per page
- **SVG → ICO** — auto-convert SVG logo to favicon during build
- **Page transitions** — fade/slide animations via htmx
- **Search** — client-side page search in sidebar
- **MathJax + highlight.js** — LaTeX math and syntax highlighting
- **Responsive** — mobile-friendly with sidebar toggle

## Requirements

- Rust 1.75+ (with cargo)
- System libraries:
  - Linux: `libxcb`, `libx11`, `libxkbcommon`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools

## Installation

```bash
# Clone and build
git clone https://github.com/freatevietnam/freateoj-buildwiki
cd freateoj-buildwiki
cargo build --release

# The binary will be at target/release/freateoj-wiki
```

## Usage

### GUI (default)
```bash
./target/release/freateoj-wiki
```

### CLI
```bash
# Build only
./target/release/freateoj-wiki build

# Build and serve
./target/release/freateoj-wiki serve

# Custom port
./target/release/freateoj-wiki serve --port 3000

# Custom paths
./target/release/freateoj-wiki build --db mywiki.db --output mybuild
```

### Keyboard Shortcuts

- `Ctrl+B` — Build

## Project Structure

```
freateoj-buildwiki/
├── Cargo.toml          # Rust dependencies
├── src/
│   ├── main.rs         # Entry point + CLI
│   ├── db.rs           # SQLite database
│   ├── build.rs        # Static site generation
│   ├── server.rs       # HTTP preview server
│   ├── svg.rs          # SVG → ICO conversion
│   └── gui/
│       ├── mod.rs      # GUI entry
│       └── app.rs      # egui application
├── wiki.db             # SQLite database (auto-created)
├── build/              # Generated static output
├── .gitignore
└── README.md
```

## Cross-Platform Build

### Linux
```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libssl-dev libgtk-3-dev

cargo build --release
```

### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

cargo build --release
```

### Windows
```bash
# Install Visual Studio Build Tools
# Then build with cargo
cargo build --release
```

## License

MIT
