# Zorter — Local-First AI File Organizer (v1.1)

Zorter is a lightweight, production-grade local application that uses locally running AI vision models (via Ollama) to understand, categorize, rename, and organize image files and wallpapers.

The application works 100% locally on your machine without cloud dependencies or uploading personal files.

---

## CLI & GUI Usage

Zorter can be run as a lightning-fast command-line tool or as a Tauri desktop app.

### CLI Commands

```bash
# Organize a directory of images using AI vision
zorter organize "D:\Downloads\Desktop Wallpapers"

# Shorthand organize syntax
zorter "D:\Downloads\Desktop Wallpapers"

# Dry run: preview proposed AI renames and categories without moving files
zorter "D:\Downloads\Desktop Wallpapers" --dry-run

# Run non-interactively (auto-confirm execution)
zorter "D:\Downloads\Desktop Wallpapers" --yes

# Specify organization structure and naming convention
zorter "D:\Downloads\Desktop Wallpapers" --style by-category --naming snake-case
zorter "D:\Downloads\Desktop Wallpapers" --style flat --naming kebab-case

# Choose a specific Ollama vision model
zorter "D:\Downloads\Desktop Wallpapers" --model ministral-3:3b

# Scan a directory, compute BLAKE3 hashes, and report duplicates in < 1 second
zorter scan "D:\Downloads\Desktop Wallpapers"

# Undo the last batch of file moves and renames
zorter undo

# Check Ollama service connectivity and installed models
zorter status

# Launch the desktop graphical interface
zorter gui
```

### CLI Command Options

| Command | Option | Description |
|---|---|---|
| `organize` | `<PATH>` | Target directory containing images |
| | `--dry-run`, `-n` | Preview renames and folder changes without modifying disk |
| | `--yes`, `-y` | Execute operations without interactive confirmation |
| | `--style`, `-s` | Organization folder layout: `by-category` (default) or `flat` |
| | `--naming` | Naming convention: `snake-case` (default), `kebab-case`, or `title-case` |
| | `--model`, `-m` | Ollama vision model (default: `ministral-3:3b`) |
| `scan` | `<PATH>` | Scans images and BLAKE3 hashes, reporting count, size, and duplicates |
| `undo` | | Reverts the last file rename/move batch using the operation journal |
| `status` | | Shows Ollama connection status and installed vision/embedding models |
| `gui` | | Opens the Tauri desktop GUI |

---

## Architecture Overview

```
                      ┌────────────────────────────────────────┐
                      │    CLI Interface  /  React Desktop     │
                      │  indicatif   clap │ Dashboard Organize │
                      └───────────────────▲────────────────────┘
                                          │
                      ┌───────────────────▼────────────────────┐
                      │              Rust Core                 │
                      └─┬──────────────┬──────────────┬──────┬─┘
                        │              │              │      │
            ┌───────────▼──┐    ┌──────▼───────┐      │   ┌──▼───────────┐
            │  Filesystem  │    │ AI Provider  │      │   │ Local Search │
            │    Engine    │    │ (Ollama)     │      │   │ Gemma Embed  │
            │ BLAKE3 / Jpeg│    │  Ministral / │      │   │ Cosine Rank  │
            │              │    │  Qwen / Gemma│      │   │              │
            └──────────────┘    └──────────────┘      │   └──────────────┘
                                ┌─────────────────────▼┐
                                │ JSON Persistence &   │
                                │ Operation Journal    │
                                └──────────────────────┘
```

### The 5 Core Engines

1. **Filesystem Engine (`src-tauri/src/filesystem/`)**
   - Recursive scanning (`walkdir`)
   - Fast, collision-resistant content fingerprinting (`BLAKE3`)
   - Incremental scanning: size & timestamp comparison to avoid re-hashing unchanged files
   - Cached thumbnail generation (`image` crate)
   - Real-time directory watching (`notify`)
   - Safe execution: dry-run path validation, non-overwriting moves, and reversible operation journal

2. **AI Intelligence Engine (`src-tauri/src/ai/`)**
   - Provider abstraction: `VisionProvider`, `EmbeddingProvider`, `RuntimeProvider`
   - Local Ollama runtime integration (`ministral-3:3b`, `qwen3.5:4b`, etc.)
   - Strict structured JSON response parsing (subject, style, category, lighting, colors, environment)
   - Optimized downscaling to 768px with fast filtering to prevent VRAM spikes and run smoothly on 4GB GPUs
   - Decoupled embedding generation so Ollama doesn't thrash/swap models during vision passes

3. **Organization Engine (`src-tauri/src/organization/`)**
   - Deterministic Rust naming rules using validated AI metadata (`clean_snake_case`, `kebab-case`, `Title_Case`)
   - Rejects corrupt or hallucinated repeated-character hashes (e.g., `tvydfetvydfe...`)
   - Category and subcategory taxonomy normalization (e.g. `Wallpapers/Nature`, `Wallpapers/Cyberpunk`)
   - Destination folder hierarchy planning (`by_category`, `category_and_style`, `flat_rename`)
   - Automatic collision avoidance appending numbered suffixes (`_001.png`, etc.)
   - BLAKE3 exact duplicate detection and wasted storage calculation

4. **File Operation Engine & Journal (`src-tauri/src/filesystem/operations.rs`)**
   - Preview before execution
   - Atomic moves/renames with fallback
   - Append-only operation journal (`history/operations.jsonl`)
   - Complete 1-click Undo / Rollback

5. **Local Semantic Search Engine (`src-tauri/src/search/`)**
   - Local vector index using Gemma Embeddings
   - Fast cosine similarity ranking
   - Natural language search queries ("Show dark cyberpunk wallpapers with neon lights")
   - "Find Similar Images" discovery
   - Combined metadata filtering (category, style, orientation)

---

## Local JSON Persistence

Application data is saved in your local OS AppData folder (`%APPDATA%/zorter` on Windows):

- `config/settings.json`: User model, naming, and concurrency settings
- `index/files.json`: Indexed file metadata and BLAKE3 hashes
- `index/analysis.json`: Structured visual descriptions and attributes
- `index/directories.json`: Monitored folders
- `index/embeddings/embeddings_001.json`: Sharded 500-vector JSON files
- `history/operations.jsonl`: Append-only journal of all operations
- `cache/thumbnails/{hash}.jpg`: Cached image thumbnails

---

## Development & Verification

### 1. Run Unit Tests (Rust Backend)
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 2. Build Release CLI
```bash
cargo build --release --manifest-path src-tauri/Cargo.toml
```
The binary will be located at `src-tauri/target/release/zorter.exe`.
Add it to your `PATH` or invoke it directly.

### 3. Build Desktop Frontend
```bash
npm run build
```
