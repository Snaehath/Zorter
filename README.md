# Zorter — Local-First AI File Organizer (v1.1)

Zorter is a lightweight, production-grade desktop application that uses locally running AI models to understand, categorize, rename, search, and organize image files and wallpapers.

The application works 100% locally on your machine without cloud dependencies or uploading personal files.

---

## Architecture Overview

```
                      ┌────────────────────────────────────────┐
                      │    React 19 + TypeScript + Tailwind    │
                      │  Dashboard  │  Organize  │   Explore   │
                      └───────────────────▲────────────────────┘
                                          │ Tauri IPC & Events
                      ┌───────────────────▼────────────────────┐
                      │             Tauri 2 Backend            │
                      └─┬──────────────┬──────────────┬──────┬─┘
                        │              │              │      │
            ┌───────────▼──┐    ┌──────▼───────┐      │   ┌──▼───────────┐
            │  Filesystem  │    │ AI Provider  │      │   │ Local Search │
            │    Engine    │    │ (Ollama)     │      │   │ Gemma Embed  │
            │ BLAKE3 / Jpeg│    │ Gemma / Qwen │      │   │ Cosine Rank  │
            └──────────────┘    └──────────────┘      │   └──────────────┘
                                ┌─────────────────────▼┐
                                │ JSON Persistence &   │
                                │ Operation Journal    │
                                └──────────────────────┘
```

### The 5 Core Engines

1. **Filesystem Engine (`src-tauri/src/filesystem/`)**
   - Recursive asynchronous scanning (`walkdir`)
   - Fast, collision-resistant content fingerprinting (`BLAKE3`)
   - Incremental scanning: size & timestamp comparison to avoid re-hashing unchanged files
   - High-performance cached thumbnail generation (`image` crate)
   - Real-time directory watching (`notify`)
   - Safe execution: dry-run path validation, non-overwriting moves, and reversible operation journal

2. **AI Intelligence Engine (`src-tauri/src/ai/`)**
   - Provider abstraction: `VisionProvider`, `EmbeddingProvider`, `RuntimeProvider`
   - Local Ollama runtime integration (`qwen3.5:4b`, `ministral-3:3b`, `gemma`)
   - Strict structured JSON response parsing (subject, style, category, lighting, colors, environment)
   - Pre-inference downscaling to max 1024px to conserve VRAM on RTX 3050 4GB Laptop GPUs
   - Bounded concurrency with async semaphores

3. **Organization Engine (`src-tauri/src/organization/`)**
   - Deterministic Rust naming rules using validated AI metadata (`clean_snake_case`, `kebab-case`, `Title_Case`)
   - Category and subcategory taxonomy normalization
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

## Project Structure

```
zorter/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── tests.rs
│       ├── core/
│       │   ├── app_state.rs
│       │   ├── job_manager.rs
│       │   ├── events.rs
│       │   └── errors.rs
│       ├── filesystem/
│       │   ├── scanner.rs
│       │   ├── metadata.rs
│       │   ├── hashing.rs
│       │   ├── thumbnail.rs
│       │   ├── watcher.rs
│       │   └── operations.rs
│       ├── ai/
│       │   ├── mod.rs
│       │   ├── provider.rs
│       │   ├── ollama.rs
│       │   ├── model_registry.rs
│       │   ├── vision.rs
│       │   ├── embedding.rs
│       │   └── prompts.rs
│       ├── organization/
│       │   ├── classifier.rs
│       │   ├── naming.rs
│       │   ├── folder_planner.rs
│       │   └── deduplication.rs
│       ├── search/
│       │   ├── semantic.rs
│       │   ├── vector_index.rs
│       │   └── filters.rs
│       ├── storage/
│       │   ├── json_store.rs
│       │   ├── migrations.rs
│       │   ├── recovery.rs
│       │   └── repository.rs
│       └── commands/
│           ├── scan.rs
│           ├── analyze.rs
│           ├── organize.rs
│           ├── search.rs
│           ├── settings.rs
│           └── undo.rs
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── index.css
│   ├── types/index.ts
│   ├── services/api.ts
│   └── components/
│       ├── Header.tsx
│       ├── DashboardScreen.tsx
│       ├── OrganizeScreen.tsx
│       ├── ExploreScreen.tsx
│       └── SettingsModal.tsx
└── package.json
```

---

## Local JSON Persistence

Application data is saved in your local OS AppData folder (`%APPDATA%/zorter` on Windows):

- `config/settings.json`: User model, naming, and concurrency settings
- `index/files.json`: Indexed file metadata and BLAKE3 hashes
- `index/analysis.json`: Structured visual descriptions and attributes
- `index/directories.json`: Monitored folders
- `index/embeddings/embeddings_001.json`: Sharded 500-vector JSON files
- `history/operations.jsonl`: Append-only journal of all operations
- `cache/thumbnails/{hash}.jpg`: Cached 256x256 image thumbnails

---

## Quickstart & Verification

### 1. Run Unit Tests (Rust Backend)
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 2. Build Frontend (React + TypeScript)
```bash
npm run build
```

### 3. Launch Desktop Application (Dev Mode)
```bash
npm run tauri dev
```
*(Ensure local Ollama service is running: `ollama serve`)*
