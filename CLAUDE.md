# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Aella is a grammar checking application built on harper-core (private grammar checker). The project consists of:
- **Desktop GUI**: iced-rs application with real-time grammar checking and markdown support
- **CLI**: Command-line interface for grammar checking (to be implemented as separate crate)
- **Storage**: Turso database for persisting conversations/texts
- **Search**: Simple search for navigating conversations

## Architecture

### Workspace Structure
The project uses a Cargo workspace with two crates:
- `aella-desktop`: iced GUI application with markdown editor, grammar checking, and sidebar navigation
- `aella-cli`: CLI tool for grammar checking

Both crates share core functionality through harper-core. Workspace dependencies are defined in the root `Cargo.toml`.

### Key Components
- **Grammar Engine**: harper-core handles all grammar checking logic
- **UI Framework**: iced with tokio async runtime, canvas, and markdown features enabled
- **Database**: Turso for storing and retrieving conversations
- **Search**: Simple text-based search for conversation navigation in the sidebar

### Data Flow
1. User types text in markdown editor (desktop) or provides input (CLI)
2. Text is processed through harper-core for grammar checking
3. Suggestions are displayed in real-time (desktop) or as output (CLI)
4. Conversations/texts are saved to Turso database
5. Sidebar search uses simple text matching to filter through stored conversations

## Development Commands

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p aella-desktop
cargo build -p aella-cli

# Run desktop app
cargo run -p aella-desktop

# Run CLI
cargo run -p aella-cli

# Run tests
cargo test

# Run tests for specific crate
cargo test -p aella-desktop

# Run specific test
cargo test test_name

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Technical Notes

### iced Application Structure
iced apps follow an Elm-like architecture with:
- **State**: Application state struct
- **Message**: Enum of all possible events
- **update()**: Handles messages and updates state
- **view()**: Renders UI based on current state

The markdown feature is already enabled in dependencies and should be used for the text editor component.

### Turso Integration
Turso requires async operations. All database calls should be wrapped in tokio async context. Connection pooling and error handling are critical for reliable conversation storage.

### Search Implementation
Simple text-based search filters conversations by matching search terms against conversation titles and content. The search operates on pre-normalized text fields for better performance.
