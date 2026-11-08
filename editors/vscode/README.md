# Myelin Language Support for Visual Studio Code

Official VS Code extension providing rich syntax highlighting, snippets, file icons, and language configuration for the **Myelin (`.myl`)** programming language — the token-minimal, capability-safe, flat-SSA systems language.

![Myelin VS Code Extension](icons/myelin-icon.png)

---

## ✨ Features

- **Review-Centric Syntax Highlighting:**
  - **Directives:** `#mod`, `#fn`, `#rec`, `#sum`, `#ext`, `#test` are styled cleanly at column 0.
  - **Control Tags:** `@if`, `@elif`, `@el`, `@for`, `@loop`, `@match`, `@brk`, `@next` highlight flat control flow without curly braces.
  - **Verification Contracts:** `req` (precondition), `ens` (postcondition), and `ret` stand out as first-class verification primitives.
  - **Capability Tags:** Effect tags (`!io`, `!net`, `!db`, `!fs`, `!time`, `!rand`, `!unsafe.*`) are given high visual distinction for instant security auditing.
  - **Type System:** Rich parsing for primitives (`i64`, `u64`, `str`, `bool`, etc.), optional types (`?T`), fallible types (`T!E`), slice types (`[]T`), and sum constructors (`Ok`, `Err`, `Some`).
  - **Guards & Returns:** Early return errors (`^`), guard pipes (`|`), guard assertions (`?`), error propagation unwrap (`?`), and SSA returns (`=`).
- **2-Space Indentation Rules:**
  - Automatically enforces Myelin's canonical 2-space indentation rule for `.myl` files.
  - Auto-indents on `#fn`, `#test`, `@if`, `@loop`, `@match`, and auto-dedents `@elif`, `@el`.
- **Comment Toggling:** Standard `--` line comments via `Ctrl+/` (`Cmd+/` on macOS).
- **Productivity Snippets:** Fast tabstop completions for `#fn`, `#rec`, `#sum`, `@if`, `@loop`, `@match`, `contract`, `guard`, `#test`.
- **Dedicated File Icons:** Branded Myelin file icons for `.myl` source files in the file explorer and tab bar.

---

## 🚀 Installation

### Option 1: Direct Copy (Local Development)

You can install the extension directly by copying `editors/vscode` to your local VS Code extensions directory:

#### **Windows (PowerShell):**
```powershell
Copy-Item -Recurse -Path "editors\vscode" -Destination "$HOME\.vscode\extensions\myelin-lang"
```

#### **Linux / macOS (Bash):**
```bash
cp -r editors/vscode ~/.vscode/extensions/myelin-lang
```

Then reload Visual Studio Code:
- Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
- Run `Developer: Reload Window`

---

### Option 2: Package & Install via VSIX

If you have Node.js and `@vscode/vsce` installed:

1. Navigate to the extension folder:
   ```bash
   cd editors/vscode
   ```
2. Package the extension:
   ```bash
   npx @vscode/vsce package
   ```
3. Install the generated `.vsix` file:
   ```bash
   code --install-extension myelin-lang-0.1.0.vsix
   ```
   *Alternatively, in VS Code, open the Extensions view (`Ctrl+Shift+X`), click the **`...`** (Views and More Actions) menu in the top right, and select **Install from VSIX...**.*

---

### Option 3: Cursor IDE / VSCodium

This extension is 100% compatible with Cursor IDE and VSCodium:

- **Cursor:** Copy to `~/.cursor/extensions/myelin-lang` (or `%USERPROFILE%\.cursor\extensions\myelin-lang` on Windows).
- **VSCodium:** Copy to `~/.vscode-oss/extensions/myelin-lang` (or `%USERPROFILE%\.vscode-oss\extensions\myelin-lang` on Windows).

---

## ⌨️ Snippets Cheatsheet

Type any of the following prefixes in a `.myl` file and press `Tab` or `Enter`:

| Prefix | Output | Purpose |
| :--- | :--- | :--- |
| `#fn` | `#fn name [args -> ret] !cap` | Function with contracts & capabilities |
| `#rec` | `#rec Name [field:type ...]` | Struct record declaration |
| `#sum` | `#sum Name [VariantA VariantB(type)]` | Tagged union declaration |
| `@if` | `@if cond ... @el` | Conditional branching |
| `@loop` | `@loop ? idx < 10 \| @brk` | Loop with exit guard |
| `@for` | `@for item : collection` | Slice/collection iteration |
| `@match` | `@match res Ok(v) => ...` | Exhaustive pattern match |
| `contract` | `req ... ens ...` | Precondition & postcondition contract |
| `guard` | `? condition \| ^ErrVariant` | Guard condition with early return |
| `#test` | `#test test_name` | Embedded unit test block |
| `#mod` | `#mod namespace` | Module declaration |
| `#use` | `#use module` | Module import |

---

## ⚙️ Configuration

The extension activates automatically for all files matching `*.myl`. It sets the following workspace defaults for Myelin files:

```json
{
  "[myelin]": {
    "editor.tabSize": 2,
    "editor.insertSpaces": true,
    "editor.detectIndentation": false
  }
}
```

---

## 📄 License

Licensed under the [MIT License](../../LICENSE).
