# Myelin Lang Documentation & Wiki

Official documentation and wiki for **Myelin Programming Language**, built with [VitePress](https://vitepress.dev/) and powered by full bilingual support (English & Turkish), offline client-side search, and custom Shiki syntax highlighting for Myelin (`.myl`).

---

## 🚀 Quick Start (Local Development)

To run the documentation site locally with live reloading:

```bash
cd wiki
npm install
npm run dev
```

Open your browser at `http://localhost:5173` to browse the interactive documentation.

---

## 📦 Production Build

To build the static site (optimized HTML, CSS, JS):

```bash
cd wiki
npm run build
```

The output will be generated inside `wiki/.vitepress/dist/`.

To preview the production build locally:

```bash
npm run preview
```

---

## 📂 Project Structure

```text
wiki/
├── .vitepress/
│   ├── config.mts              # VitePress configuration, i18n locales, navbars, sidebars & Shiki syntax highlighting
│   └── dist/                   # Production build output (gitignored)
├── package.json
├── index.md                    # English Landing / Hero Page
├── guide/                      # English Documentation Pages
│   ├── getting-started.md      # Introduction, motivation, core guarantees
│   ├── installation.md         # Building myelinc, CLI commands, agent mode (--llm)
│   ├── syntax-guide.md         # #mod, #rec, #sum, #fn, #test, 2-space rule, types
│   ├── control-flow.md         # @if, @elif, @el, guards (? ... |), @for, @loop, @match
│   ├── capabilities-security.md# Zero Ambient Authority, !net, !db, !fs, req/ens, audit
│   ├── builtin-modules.md      # io.println, str.fmt, db.set/get/has, http.serve
│   ├── llm-integration.md      # --llm wire format, SKILL.md, token efficiency benchmarks
│   └── architecture.md         # Crates (diag, ast, lexer, parser, myelinc), roadmap
└── tr/                         # Türkçe Dokümantasyon
    ├── index.md                # Türkçe Karşılama Sayfası
    └── guide/
        ├── getting-started.md  # Giriş ve Felsefe
        ├── kurulum.md          # Kurulum ve Derleyici Komutları
        ├── sozdizimi.md        # Sözdizimi, Direktifler ve 2-Boşluk Kuralı
        ├── akis-kontrolu.md    # @if, Guard (?) yapısı, Döngüler, @match
        ├── guvenlik-ve-yetenekler.md # Yetenekler (!), req/ens, Denetim (audit)
        ├── yerlesik-moduller.md# io, str, db (anahtar-değer deposu), http (web sunucusu)
        ├── llm-entegrasyonu.md # Tek satırlık LLM protokolü, SKILL.md, token tasarrufu
        └── mimari.md           # Crate mimarisi, yeni özellik ekleme, yol haritası
```

---

## ✍️ Adding New Features & Documentation Pages

When you develop a new feature in Myelin:
1. **English page:** Add a new `.md` file in `wiki/guide/<feature-name>.md` or update the relevant existing file (e.g. `builtin-modules.md`).
2. **Turkish page:** Add a matching `.md` file in `wiki/tr/guide/<ozellik-adi>.md`.
3. **Register in Navbar & Sidebar:** Open `wiki/.vitepress/config.mts`, and add the new link under both `root` (English) and `tr` (Turkish) sidebar sections.
4. **Test Build:** Run `npm run build` in `wiki/` to ensure all links and markdown syntax are valid.

---

## 🌐 Automatic GitHub Pages Deployment

A GitHub Actions workflow is preconfigured at `.github/workflows/deploy-wiki.yml`. Any commit pushed to the `main` branch touching `wiki/**` automatically builds and deploys the wiki to GitHub Pages.
