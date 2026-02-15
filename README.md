# Aella

Your friendly neighborhood grammar checker, built for speed and simplicity.

## What's This?

Aella is a blazing fast grammar checking app powered by `harper-core`. Whether you prefer a sleek desktop experience or the command line, we've got you covered.

## Features

- **Real-time Grammar Checks**: Catch mistakes as you type.
- **Markdown Support**: Write in your favorite format.
- **Auto-Save History**: Never lose a thought (backed by Turso).
- **Search**: Find old conversations instantly.

## Look and Feel

![Main View](./.github/assets/main.avif)

![Shortcuts Modal](./.github/assets/shortcuts.avif)

![Trashed View](./.github/assets/trashView.avif)

## The Goods

- **`aella-desktop`**: The shiny GUI (built with `iced`).
- **`aella-cli`**: For the terminal users.

## Getting Started

Make sure you have Rust installed, then:

```bash
# Run the desktop app
cargo run -p aella-desktop

# Run the CLI
cargo run -p aella-cli
```

### MacOS

You can use cargo-bundle:

```bash
cargo bundle -p aella-desktop --release
```

That's it. Happy writing! 📝
