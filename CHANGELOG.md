## [0.1.1] - 2026-02-16

### ⚙️ Miscellaneous Tasks

- Update cargo-bundle version in GitHub Actions workflow to 0.9.0
## [0.1.0] - 2026-02-16

### 🚀 Features

- Implement initial Iced desktop application featuring a collapsible sidebar and a markdown editor.
- Implement grammar checking, markdown rendering, and editor view modes.
- Add cursor position display and implement a collapsible errors panel.
- Implement local SQLite database for conversation persistence and management.
- Implement conversation soft-delete with a trash feature, including database schema, operations, and UI updates.
- Implement compact conversation badges in the collapsed sidebar view, replacing the mode button.
- Add tooltips to sidebar buttons for improved usability.
- Store database in platform-specific data directory for release builds and add `dirs` dependency.
- Implement applying lint suggestions and visual highlighting for lints in the editor.
- Add lint highlighting to the editor in both views mode.
- Add platform-specific application icons and update bundle metadata.
- Add global keyboard shortcuts for view modes, UI toggles, and search input focus.
- Add a keyboard shortcuts help modal accessible via keybindings.
- Update shortcuts help toggle to `Cmd/Ctrl + K`, enable closing by clicking backdrop, and centralize primary modifier detection.
- Add platform-aware shortcut hint to editor footer alongside cursor position.
- Animate the keyboard shortcuts help modal's appearance and disappearance with a new animation state and tick-based updates.
- Add Geist font to the application
- Implement delayed sidebar search focus when uncollapsing the sidebar via keyboard shortcut.
- Add GitHub Actions workflow for testing Rust projects (#5)

### 🐛 Bug Fixes

- Ensure only one lint suggestion is applied per unique span during auto-fix.
- Building on trunk push

### 💼 Other

- Expand icon paths in Cargo.toml bundle metadata to include more specific references.

### 🚜 Refactor

- Centralize plus icon SVG path into `PLUS_ICON_PATH` constant using `CARGO_MANIFEST_DIR`.
- Adjust button content alignment, padding, and height for improved layout.
- Extract application logic, state, and UI into a new 'app' module.
- Extract sidebar and editor UI components into a new `ui` module.
- Extract common UI container and tooltip styling into a new `styles` module.

### ⚙️ Miscellaneous Tasks

- Init claude
- Add macos gitignore thingy
- Update README and replace asset images
