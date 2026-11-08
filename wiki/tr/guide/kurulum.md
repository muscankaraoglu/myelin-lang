# Kurulum ve Derleyici Araçları

Myelin araç seti, tek bir birleşik çalıştırılabilir dosya olarak sunulur: `myelinc`.

## Gereksinimler

- [Rust ve Cargo](https://rustup.rs/) (Sürüm 1.80 veya üzeri)
- Windows, macOS veya Linux işletim sistemi

---

## Kaynak Koddan Derleme

Depoyu yerel bilgisayarınıza klonlayın ve release modunda derleyin:

```bash
git clone https://github.com/can/ailang.git
cd ailang

# Birim testlerini çalıştırın
cargo test

# Optimize edilmiş derleyiciyi üretin
cargo build --release -p myelinc
```

Derlenen ikili dosya şu dizinde hazır olacaktır:
- **Windows:** `target/release/myelinc.exe`
- **Linux / macOS:** `target/release/myelinc`

Bu ikili dosyayı sisteminizin `$PATH` ortam değişkeninde yer alan herhangi bir dizine (örneğin `/usr/local/bin` veya `C:\Windows\System32`) kopyalayabilirsiniz.

---

## CLI Komutları Genel Bakış

`myelinc`, hem geliştiriciler hem de otonom yapay zeka ajanları için optimize edilmiş komutlar sunar:

```bash
myelinc <KOMUT> <dosya.myl> [--llm] [argümanlar...]
```

| Komut | Açıklama |
| :--- | :--- |
| `myelinc run <dosya.myl> [arg...]` | Myelin kaynak dosyasını yerleşik çalışma motoru ile anında çalıştırır; ek argümanları `cli.args` modülüne aktarır. |
| `myelinc check <dosya.myl>` | Kodu çalıştırmadan sözcük (lexing), sözdizimi (parsing) ve anlamsal doğrulamadan geçirir. |
| `myelinc audit <dosya.myl>` | Yetenek güvenliğini ve sözleşmeleri denetleyip kriptografik SHA-256 İnsan Güven Manifestosu üretir. |
| `myelinc parse <dosya.myl>` | Kaynak kodun Soyut Sözdizim Ağacını (AST) ekrana yazdırır. |
| `myelinc lex <dosya.myl>` | Kaynak koddaki token akışını satır ve sütun bilgileriyle listeler. |
| `--llm` bayrağı | Tüm hata ve denetim kararlarını LLM'lerin doğrudan işleyebileceği tek satırlık formata dönüştürür. |

---

## İlk Programınızı Çalıştırın

`merhaba.myl` adında bir dosya oluşturun:

```myelin
#mod main

#fn main [] !io
  io.println "Myelin Dünyasına Hoş Geldiniz!"
  io.println "Yapay Zeka İçin Yerel Sistem Programlama Dili"
```

Programı çalıştırın:

```bash
myelinc run merhaba.myl
```

Çıktı:
```text
Myelin Dünyasına Hoş Geldiniz!
Yapay Zeka İçin Yerel Sistem Programlama Dili
Execution completed in 398 µs
```

---

## Otonom Ajan Modu (`--llm`)

Bir yapay zeka modeli (Claude, GPT, Gemini) kod yazıp derleyiciyi çalıştırdığında her zaman `--llm` bayrağı kullanılır:

```bash
myelinc check merhaba.myl --llm
```

Dosyada bir hata varsa, `myelinc` tek bir satırda kesin tamir talimatını üretir:

```text
ERR E002 merhaba.myl:4:3 | odd_indent spaces=3 | fix: align statement to 2 or 4 spaces
```

Model bu çıktıyı anında yorumlayıp hatayı tek adımda düzeltebilir.
