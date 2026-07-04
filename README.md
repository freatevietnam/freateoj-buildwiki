# FreateOJ Wiki Builder

Single-file wiki builder with GUI, SQLite database, and static HTML output.

## Features

- **Single file** — entire app in `wiki.py` (~1400 lines)
- **SQLite database** — pages, sections, SEO settings stored in `wiki.db`
- **Static HTML output** — generates `build/` with clean, fast pages
- **Light mode UI** — customtkinter GUI matching web design
- **Per-page SEO** — OG images, Twitter cards, robots per page
- **SVG → ICO** — auto-convert SVG logo to favicon during build
- **Page transitions** — fade/slide animations via htmx
- **Search** — client-side page search in sidebar
- **MathJax + highlight.js** — LaTeX math and syntax highlighting
- **Responsive** — mobile-friendly with sidebar toggle

## Requirements

```
pip install customtkinter markdown Pillow cairosvg
```

## Usage

### GUI
```bash
python wiki.py
```

### CLI
```bash
# Build only
python wiki.py --cli

# Build and serve
python wiki.py --cli --serve

# Custom port
python wiki.py --cli --serve --port 3000
```

## Keyboard Shortcuts

- `Ctrl+B` — Build

## Project Structure

```
freateoj-buildwiki/
├── wiki.py          # App (GUI + DB + build + serve)
├── wiki.db          # SQLite database (auto-created)
├── build/           # Generated static output
├── requirements.txt
├── .gitignore
└── README.md
```

## License

MIT
