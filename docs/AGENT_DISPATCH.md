# Myelin Subagent Orchestration & Task Dispatch Guide

Use these prompt templates to launch autonomous subagents for specialized tracks.
Each subagent executes within its domain, reads its dedicated skill, and reports findings.

---

### Track 1: Compiler & Core Architecture
* **Target Role:** `Lead Compiler Engineer`
* **Skill File:** `.agent-skills/core-compiler.md`
* **Mission:** Parser completeness, Pratt expression engine, and type/capability checking (`myelin-check`).

---

### Track 2: Standard Library & Web Runtime
* **Target Role:** `Lead Runtime & Ecosystem Engineer`
* **Skill File:** `.agent-skills/stdlib-web.md`
* **Mission:** Native HTTP/WebSocket server, embedded database engine, and serialization.

---

### Track 3: Desktop Platform Vertical (Windows, macOS, Linux)
* **Target Role:** `Lead Desktop Systems Engineer`
* **Skill File:** `.agent-skills/stdlib-desktop.md`
* **Task Prompt:**
  ```text
  Task: Design and prototype `std.gui` windowing primitives for Myelin.
  1. Read .agent-skills/stdlib-desktop.md and docs/SPEC.md.
  2. Prototype a lightweight native window binding in myelinc.
  3. Create examples/desktop_app.myl demonstrating an interactive window with click events.
  4. Ensure capability `!gui` is required for window creation.
  ```

---

### Track 4: Mobile Platform Vertical (Android & iOS)
* **Target Role:** `Lead Mobile Systems Engineer`
* **Skill File:** `.agent-skills/stdlib-mobile.md`
* **Task Prompt:**
  ```text
  Task: Design the mobile cross-compilation bridge and `std.mobile` APIs.
  1. Read .agent-skills/stdlib-mobile.md.
  2. Document how Myelin code compiles to C-ABI shared objects (.so / .dylib) for Kotlin NDK and Swift FFI.
  3. Create examples/mobile_shared_logic.myl showcasing shared business logic.
  ```

---

### Track 5: Console & Terminal Vertical (CLI & TUI)
* **Target Role:** `Lead CLI & Terminal UX Engineer`
* **Skill File:** `.agent-skills/stdlib-cli.md`
* **Task Prompt:**
  ```text
  Task: Implement `std.cli` argument parsing and ANSI terminal colors in Myelin.
  1. Read .agent-skills/stdlib-cli.md.
  2. Add built-in functions: `cli.args`, `term.color`, `term.println_bold`.
  3. Create examples/cli_tool.myl showing a modern developer CLI with flags and colored status outputs.
  ```

---

### Track 6: Community, Wiki & Developer Relations (DevRel)
* **Target Role:** `Lead Community & Developer Advocate`
* **Skill File:** `.agent-skills/community-devrel.md`
* **Task Prompt:**
  ```text
  Task: Initialize the official Myelin Wiki and Getting Started guide.
  1. Read .agent-skills/community-devrel.md.
  2. Create the `wiki/` directory with:
     - wiki/getting-started.md
     - wiki/syntax-guide.md
     - wiki/capabilities-security.md
     - wiki/builtin-modules.md
  3. Ensure all examples use valid `.myl` syntax and highlight the %60 token advantage.
  ```
