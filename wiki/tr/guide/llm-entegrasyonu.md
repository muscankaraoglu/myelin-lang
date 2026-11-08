# LLM ve Otonom Ajan Entegrasyonu

Myelin, Büyük Dil Modelleri ile doğrudan donanım seviyesinde kod yürütme arasındaki köprü olmak üzere tasarlanmıştır.

## LLM Teşhis Protokolü (`--llm`)

Geleneksel derleyiciler insanların okuması için çok satırlı görsel çıktılar üretir:

```text
error[E0308]: mismatched types
  --> src/main.rs:12:18
   |
12 |     let x: i32 = "hello";
   |            ---   ^^^^^^^ expected `i32`, found `&str`
```

Bu çıktı bir LLM'e verildiğinde:
1. Modelin değerli bağlam penceresini (context window) gereksiz yere doldurur.
2. Üretim gecikmesini artırır.
3. Modelin hatayı bulup tamir etmesi için ek akıl yürütme adımları harcamasına yol açar.

### Myelin Tek Satırlık Protokolü

`myelinc` çalıştırılırken `--llm` bayrağı eklendiğinde tüm hata, uyarı ve ipuçları tek bir satıra sığdırılır:

```text
<ÖNEM> <KOD> <DOSYA>:<SATIR>:<SÜTUN> | <TÜR> <DETAY> | fix: <TAMİR_İPUCU>
```

### Gerçek Derleyici Çıktısı Örnekleri

```text
ERR E001 math.myl:5:1 | tab_indent | fix: replace tabs with 2-space multiples
ERR E002 defter.myl:14:3 | odd_indent spaces=3 | fix: align statement to 2 or 4 spaces
ERR E008 server.myl:18:7 | missing_capability cap=!net | fix: add !net to '#fn main' signature
```

Otonom bir ajan bu satırı basit bir regex ile anında çözümler:
```regex
^(?P<sev>ERR|WARN) (?P<code>E\d+) (?P<file>[^:]+):(?P<satir>\d+):(?P<sutun>\d+) \| (?P<detay>[^|]+) \| fix: (?P<tamir>.*)$
```

Ajan ek hiçbir token veya akıl yürütme adımı kaybetmeden kodunu bir sonraki iterasyonda hatasız hale getirir.

---

## 900 Tokenlik Ajan Beceri Dosyası (`SKILL.md`)

Ajanların yüzlerce sayfalık dokümantasyon okumasına gerek kalmaması için depoda [`SKILL.md`](https://github.com/can/ailang/blob/main/SKILL.md) hazır olarak sunulur.

### Yapay Zeka Ajanlarıyla Kullanım

Claude, ChatGPT veya Gemini modellerine Myelin kodu yazdırırken:
1. `SKILL.md` dosyasını sistem istemine (system prompt) veya bağlama ekleyin (~900 token).
2. Model derhal kurallara tam uyan `.myl` dosyaları üretecektir.
3. Ajan döngüsüne derleyici kontrolünü ekleyin:
   ```bash
   myelinc check uretilen.myl --llm
   ```
4. Eğer derleyici bir hata dönerse, bu tek satırlık çıktıyı ajana ileterek tek seferde tamir etmesini sağlayın.

---

## Token Verimliliği Karşılaştırması

Aynı algoritmanın farklı dillerde yazılması sonucu harcanan token sayıları:

| Programlama Dili | Ortalama Token Sayısı (o200k) | Ekstra Kalabalık (Boilerplate) |
| :--- | :--- | :--- |
| **Java** | 240 token | Sınıflar, paketler, süslü parantezler, istisnalar |
| **TypeScript** | 185 token | Tip tanımları, import/export, parantezler, async/await |
| **Rust** | 170 token | Yaşam süresi (lifetime) parametreleri, Result, use |
| **Python** | 130 token | İki nokta işaretleri, def, import, type hints |
| **Myelin (`.myl`)** | **68 token** | **%55 ile %70 arasında token tasarrufu!** |
