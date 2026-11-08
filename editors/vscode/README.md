# Myelin VS Code Extension

Visual Studio Code extension providing syntax highlighting, snippets, and language configuration for the **Myelin (`.myl`)** programming language.

## Features

- **Review-Centric Syntax Highlighting:**
  - Capabilities (`!io`, `!net`, `!fs`, `!unsafe.*`) are styled with high visual priority to enable fast security audits.
  - Contracts (`req`, `ens`, `ret`) stand out as first-class verification invariants.
  - Guard expressions (`? ... | ...`), early returns (`^`), and SSA returns (`=`) are distinctively colored.
  - Directives (`#fn`, `#rec`, `#sum`, `#test`) and control flow tags (`@if`, `@for`, `@match`) follow modern editor token semantics.
- **2-Space Indentation Rules:** Enforces clean, bracketless SSA indentation.
- **Snippets:** Quick expansion for `#fn`, `#rec`, `#sum`, `req`/`ens` contracts, guards, and `#test` blocks.

## Manual Installation (Local Development)

To load this extension directly into VS Code:

1. Copy the `editors/vscode` folder to your VS Code extensions directory:
   - **Windows:** `%USERPROFILE%\.vscode\extensions\myelin-lang`
   - **Linux / macOS:** `~/.vscode/extensions/myelin-lang`
2. Restart or reload Visual Studio Code (`Ctrl+Shift+P` -> `Developer: Reload Window`).
3. Open any `.myl` file (e.g., `examples/ledger.myl`) to see the review-optimized syntax highlighting in action!
