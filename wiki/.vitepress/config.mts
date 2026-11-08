import { defineConfig } from 'vitepress';

const myelinLanguage = {
  name: 'myelin',
  aliases: ['myl'],
  scopeName: 'source.myelin',
  displayName: 'Myelin',
  patterns: [
    { name: 'comment.line.double-dash.myelin', match: '--.*$' },
    { name: 'keyword.control.directive.myelin', match: '^#(mod|use|rec|sum|ext|fn|test)\\b' },
    { name: 'keyword.control.flow.myelin', match: '@(if|elif|el|for|loop|match|next|brk)\\b' },
    { name: 'keyword.control.contract.myelin', match: '\\b(req|ens)\\b' },
    { name: 'entity.name.tag.capability.myelin', match: '![a-zA-Z0-9_.]+' },
    { name: 'constant.language.boolean.myelin', match: '\\b(true|false|none)\\b' },
    { name: 'string.quoted.double.myelin', match: '"([^"\\\\]|\\\\.)*"' },
    { name: 'constant.numeric.myelin', match: '\\b\\d+(\\.\\d+)?\\b' },
    { name: 'storage.type.myelin', match: '\\b(i8|i16|i32|i64|u8|u16|u32|u64|usz|f32|f64|bool|str|bytes)\\b' },
    { name: 'support.function.builtin.myelin', match: '\\b(io\\.println|str\\.fmt|db\\.(set|get|has)|http\\.serve|term\\.(green|red|cyan|bold|yellow|blue|magenta|gray|dim|reset)|cli\\.(args|count|arg_count|flag_bool|has_flag|flag_str|flag_int))\\b' },
    { name: 'keyword.operator.myelin', match: '(\\?|\\^|=|==|!=|<=|>=|<|>|\\+|\\-|\\*|\\/|%|\\|)' },
  ],
};

export default defineConfig({
  title: 'Myelin',
  description: 'The LLM-Native Systems Programming Language',
  lastUpdated: true,
  cleanUrls: true,

  markdown: {
    languages: [myelinLanguage as any],
  },

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      title: 'Myelin Lang',
      description: 'The LLM-Native Systems Programming Language',
      themeConfig: {
        nav: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Syntax Guide', link: '/guide/syntax-guide' },
          { text: 'Built-in Modules', link: '/guide/builtin-modules' },
          { text: 'LLM Protocol', link: '/guide/llm-integration' },
          { text: 'Architecture', link: '/guide/architecture' },
        ],
        sidebar: [
          {
            text: '🚀 Getting Started',
            items: [
              { text: 'Introduction & Philosophy', link: '/guide/getting-started' },
              { text: 'Installation & Toolchain', link: '/guide/installation' },
            ],
          },
          {
            text: '📖 Language Fundamentals',
            items: [
              { text: 'Syntax & Indentation', link: '/guide/syntax-guide' },
              { text: 'Control Flow & Guards', link: '/guide/control-flow' },
              { text: 'Capabilities & Security', link: '/guide/capabilities-security' },
            ],
          },
          {
            text: '⚡ Runtime & Built-ins',
            items: [
              { text: 'Standard Modules (io, str, db, http, term, cli)', link: '/guide/builtin-modules' },
            ],
          },
          {
            text: '🤖 LLM & Autonomous Agents',
            items: [
              { text: 'LLM Diagnostic Protocol & SKILL.md', link: '/guide/llm-integration' },
            ],
          },
          {
            text: '⚙️ Architecture',
            items: [
              { text: 'Compiler Crates & Roadmap', link: '/guide/architecture' },
            ],
          },
        ],
        editLink: {
          pattern: 'https://github.com/myelin-lang/ailang/edit/main/wiki/:path',
          text: 'Edit this page on GitHub',
        },
        footer: {
          message: 'Released under the MIT License.',
          copyright: 'Copyright © 2026 Can & Myelin Language Contributors',
        },
      },
    },
    tr: {
      label: 'Türkçe',
      lang: 'tr',
      link: '/tr/',
      title: 'Myelin Dili',
      description: 'LLM ve Otonom Ajanlar İçin Sistem Programlama Dili',
      themeConfig: {
        nav: [
          { text: 'Başlangıç', link: '/tr/guide/getting-started' },
          { text: 'Sözdizimi', link: '/tr/guide/sozdizimi' },
          { text: 'Dahili Modüller', link: '/tr/guide/yerlesik-moduller' },
          { text: 'LLM Protokolü', link: '/tr/guide/llm-entegrasyonu' },
          { text: 'Mimari', link: '/tr/guide/mimari' },
        ],
        sidebar: [
          {
            text: '🚀 Başlangıç',
            items: [
              { text: 'Giriş ve Felsefe', link: '/tr/guide/getting-started' },
              { text: 'Kurulum ve Derleyici', link: '/tr/guide/kurulum' },
            ],
          },
          {
            text: '📖 Dil Temelleri',
            items: [
              { text: 'Sözdizimi ve Girinti', link: '/tr/guide/sozdizimi' },
              { text: 'Akış Kontrolü ve Guard Yapısı', link: '/tr/guide/akis-kontrolu' },
              { text: 'Yetenekler ve Güvenlik Modeli', link: '/tr/guide/guvenlik-ve-yetenekler' },
            ],
          },
          {
            text: '⚡ Çalışma Zamanı ve Dahili Modüller',
            items: [
              { text: 'Standart Modüller (io, str, db, http, term, cli)', link: '/tr/guide/yerlesik-moduller' },
            ],
          },
          {
            text: '🤖 LLM ve Otonom Ajanlar',
            items: [
              { text: 'LLM Teşhis Protokolü ve SKILL.md', link: '/tr/guide/llm-entegrasyonu' },
            ],
          },
          {
            text: '⚙️ Mimari ve Geliştirme',
            items: [
              { text: 'Crate Mimarisi ve Yol Haritası', link: '/tr/guide/mimari' },
            ],
          },
        ],
        editLink: {
          pattern: 'https://github.com/myelin-lang/ailang/edit/main/wiki/:path',
          text: 'Bu sayfayı GitHub üzerinde düzenleyin',
        },
        docFooter: {
          prev: 'Önceki Sayfa',
          next: 'Sonraki Sayfa',
        },
        outline: {
          label: 'Bu Sayfada',
        },
        lastUpdated: {
          text: 'Son Güncelleme',
        },
        footer: {
          message: 'MIT Lisansı altında yayınlanmıştır.',
          copyright: 'Telif Hakkı © 2026 Can & Myelin Dili Geliştiricileri',
        },
      },
    },
  },

  themeConfig: {
    logo: '⚡',
    siteTitle: 'Myelin',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/can/ailang' },
    ],
    search: {
      provider: 'local',
      options: {
        locales: {
          tr: {
            translations: {
              button: {
                buttonText: 'Ara...',
                buttonAriaLabel: 'Arama yap',
              },
              modal: {
                noResultsText: 'Sonuç bulunamadı',
                resetButtonTitle: 'Aramayı sıfırla',
                footer: {
                  selectText: 'seçmek için',
                  navigateText: 'gezinmek için',
                  closeText: 'kapatmak için',
                },
              },
            },
          },
        },
      },
    },
  },
});
