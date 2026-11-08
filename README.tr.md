# Myelin (`.myl`) — Türkçe Kılavuz

[Dokümantasyon & Wiki (wiki/)](./wiki/README.md) | [English README](./README.md) | [LLM Yetenek Belgesi (SKILL.md)](./SKILL.md) | [Dil Spesifikasyonu (docs/SPEC.md)](./docs/SPEC.md) | [Derleyici Mimarisi (docs/ARCHITECTURE.md)](./docs/ARCHITECTURE.md)

**Myelin**, insanların okuması için değil; **Büyük Dil Modellerinin (LLM) minimum token ve sıfır hatayla yazması, makinelerin C/Rust hızında çalıştırması ve insanların tek satır kod okumadan derleyici kanıtlarıyla güvenmesi** için sıfırdan tasarlanmış yetki-güvenlikli (capability-safe), sıfır-GC bir sistem programlama dilidir.

Nörobiyolojide *miyelin (myelin)*, sinir aksonlarını sararak sinyal kaçağını önleyen ve iletim hızını 100 kat artıran kılıftır. Yazılım mühendisliğinde **Myelin**, yapay zeka düşüncesi ile makine sinyalleri arasındaki on yıllardır birikmiş insan kaynaklı sözdizimi yükünü ortadan kaldırır.

---

## Temel Süper Güçler

1. **Düz Satır-SSA Sözdizimi (%55–65 Token Tasarrufu):** Süslü parantez `{ }`, noktalı virgül, `let/mut`, `return` veya `try/catch` yoktur. Her satır atomik bir adımdır; `git diff` ve dosya düzenleme araçlarıyla kusursuz çalışır.
2. **Renksiz Eşzamanlılık (`async/await` Yok):** Derleyici hangi fonksiyonun I/O bekleyeceğini `!net`, `!fs`, `!db` yetki imzalarından otomatik çıkarır.
3. **Perceus RC + Bölgesel Arena Bellek:** Ne Garbage Collector duraksaması vardır ne de LLM'leri yoran Rust lifetime (`'a`) savaşları.
4. **İnsan Güven Karnesi (`myelinc audit`):** İnsan kodu okumaz; derleyicinin ürettiği OS yetki manifestosunu (`!fs`, `!net`), sözleşmeleri (`req`/`ens`) ve gömülü `#test` sonuçlarını onaylar.
5. **Derleyici ↔ LLM Onarım Protokolü (`--llm`):** Derleyici hataları insanlar için uzun paragraflar halinde değil, LLM'in tek adımda onarabileceği ~15 token'lık yapısal teşhisler olarak üretilir.

---

## Hızlı Başlangıç

```bash
# Tüm derleyici birim testlerini çalıştır
cargo test

# Örnek Myelin dosyasını LLM protokolüyle doğrula
cargo run -p myelinc -- check examples/ledger.myl --llm

# Ayrıştırılan AST (Soyut Sözdizimi Ağacı) çıktısını incele
cargo run -p myelinc -- parse examples/ledger.myl
```
