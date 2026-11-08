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
    { name: 'keyword.control.contract.myelin', match: '\\b(req|ens|ret)\\b' },
    { name: 'invalid.illegal.unsafe.myelin', match: '(?<=[\\s\\]])!unsafe(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'keyword.control.capability.security.myelin entity.name.tag.capability.security.myelin', match: '(?<=[\\s\\]])!(?:net|db|fs|proc|env|cloud)(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'entity.name.tag.capability.cli.myelin entity.name.tag.capability.myelin', match: '(?<=[\\s\\]])!(?:term|cli)(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'entity.name.tag.capability.host.myelin entity.name.tag.capability.myelin', match: '(?<=[\\s\\]])!(?:io|time|rand|hw)(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'entity.name.tag.capability.ai.myelin entity.name.tag.capability.myelin', match: '(?<=[\\s\\]])!(?:ai)(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'entity.name.tag.capability.gui.myelin entity.name.tag.capability.myelin', match: '(?<=[\\s\\]])!(?:gui|mobile)(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'entity.name.tag.capability.general.myelin entity.name.tag.capability.myelin', match: '(?<=[\\s\\]])![a-zA-Z_][a-zA-Z0-9_]*(?:\\.[a-zA-Z0-9_]+)?\\b' },
    { name: 'constant.language.boolean.myelin', match: '\\b(true|false|none)\\b' },
    { name: 'string.quoted.double.myelin', match: '"([^"\\\\]|\\\\.)*"' },
    { name: 'constant.numeric.myelin', match: '\\b\\d+(\\.\\d+)?\\b' },
    { name: 'storage.type.myelin', match: '\\b(i8|i16|i32|i64|u8|u16|u32|u64|usz|f32|f64|bool|str|bytes)\\b' },
    {
      match: '\\b(term)\\.(green|red|cyan|bold|yellow|blue|magenta|gray|dim|reset)\\b',
      captures: {
        '1': { name: 'support.class.builtin.term.myelin support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.term.myelin support.function.builtin.myelin' },
      },
    },
    {
      match: '\\b(cli)\\.(args|count|arg_count|flag_bool|has_flag|flag_str|flag_int)\\b',
      captures: {
        '1': { name: 'support.class.builtin.cli.myelin support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.cli.myelin support.function.builtin.myelin' },
      },
    },
    {
      match: '\\b(io)\\.(println|print|read_line|read_all)\\b',
      captures: {
        '1': { name: 'support.class.builtin.io.myelin support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.io.myelin support.function.builtin.myelin' },
      },
    },
    {
      match: '\\b(str)\\.(fmt|len|split|join|trim|to_upper|to_lower|starts_with|ends_with|contains)\\b',
      captures: {
        '1': { name: 'support.class.builtin.str.myelin support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.str.myelin support.function.builtin.myelin' },
      },
    },
    {
      match: '\\b(db)\\.(set|get|has|del)\\b',
      captures: {
        '1': { name: 'support.class.builtin.db.myelin support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.db.myelin support.function.builtin.myelin' },
      },
    },
    {
      match: '\\b(http)\\.(serve|get|post|request)\\b',
      captures: {
        '1': { name: 'support.class.builtin.http.myelin support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.http.myelin support.function.builtin.myelin' },
      },
    },
    {
      match: '\\b(io|str|db|http|term|cli|ai|gui|mobile|hw|cloud)\\.([a-zA-Z_][a-zA-Z0-9_]*)\\b',
      captures: {
        '1': { name: 'support.class.builtin.myelin' },
        '2': { name: 'support.function.builtin.myelin' },
      },
    },
    { name: 'support.class.builtin.myelin', match: '\\b(io|str|db|http|term|cli|ai|gui|mobile|hw|cloud)\\b' },
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
