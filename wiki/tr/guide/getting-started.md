# Giriş ve Felsefe

## Myelin Nedir?

**Myelin (`.myl`)**; yetenek tabanlı güvenlik (capability-safe), düz SSA (flat-SSA) ve sıfır çöp toplayıcı (zero-GC) mimarisine sahip, doğrudan aşağıdaki hedefler doğrultusunda sıfırdan geliştirilmiş bir sistem programlama dilidir:
- **Büyük Dil Modelleri (LLM) ve Otonom Ajanlar tarafından yazılmak**,
- **Makineler tarafından mikrosaniye düzeyinde derlenip yalın donanım hızında çalıştırılmak**,
- **İnsanlar tarafından tek bir satır kod bile okunmadan denetlenebilmek (Audit).**

Nörobiyolojide *miyelin kılıfı*, sinir aksonlarını sararak elektrik sinyallerinin dışarı sızmasını engeller ve sinirsel iletim hızını 100 katına kadar çıkarır. Yazılım dünyasında ise **Myelin**, yapay zeka akıl yürütmesi ile makine komutları arasındaki onlarca yıllık insan odaklı görsel karmaşayı ortadan kaldırır.

---

## Neden Yeni Bir Dile İhtiyaç Vardı?

Mevcut ana akım programlama dilleri (C#, TypeScript, Python, Rust, Go vb.) insan biyolojisinin sınırlarına göre tasarlandı:
- Dakikada ortalama 60 kelime yazabilen parmaklar,
- Süslü parantez eşleştirme zorunluluğu (`{ ... }`),
- Çok katmanlı iç içe bloklar ve sayfalarca süren derleyici hata çıktıları,
- Fonksiyonların habersizce dosya silmesine, ağa çıkmasına veya bellek sızdırmasına izin veren örtük yetkiler (ambient authority).

Yazılımı otonom kodlama ajanları (LLM) ürettiğinde, insanlar için yapılmış bu kolaylıklar büyük birer engele dönüşür:

| Klasik Dillerdeki Darboğaz | Myelin'in Çözümü |
| :--- | :--- |
| **%55–65 Token İsrafı:** Kapanış süslü parantezleri, gereksiz `function`, `return`, `public static void` gibi kelimeler. | **Düz Satır-SSA & Token Optimizasyonu:** Tüm anahtar sözcükler modern LLM tokenizer'larına (o200k, cl100k) göre hizalanmıştır. Kapanış parantezleri yoktur. |
| **Gizli Yan Etki Hataları:** Bir hesaplama fonksiyonunun gizlice diskteki dosyaları okuması veya dışarıya veri göndermesi. | **Sıfır Örtük Yetki (Zero Ambient Authority):** Fonksiyonlar varsayılan olarak tamamen saftır. Yan etki için `!net`, `!db`, `!fs` yetkileri zorunludur. |
| **Hatalarda Bağlam Penceresinin Dolması:** 20 satırlık C++/Rust hata mesajları LLM'in hafızasını tüketir. | **Tek Satırlık Teşhis Protokolü (`--llm`):** Modellerin tek bir token adımında anlayıp düzeltebileceği tek satırlık hata çıktısı. |
| **İnsan İnceleme Darboğazı:** İnsanlar her saat üretilen 10.000 satır kodu güvenle okuyup denetleyemez. | **Güven ve Yetenek Manifestosu (`myelinc audit`):** İnsanlar kodu değil, derleyicinin doğruladığı matematiksel yetenek ve sözleşmeleri inceler. |

---

## Temel Garantiler

1. **Deterministik LL(1) Ayrıştırma:** Her üst seviye tanım 0. sütunda `#` ile başlar. Her kontrol akışı `@` ile başlar. Asla belirsiz bir ayrıştırma durumu oluşmaz.
2. **2-Boşluk Girinti Kuralı:** Tab (`\t`) kullanımı kesinlikle yasaktır (`E001`). Girintiler daima 2'nin katı olmalıdır.
3. **Sıfır Null & Sıfır Yakalanmamış İstisna:** İsteğe bağlılık (`?T`) ve hata durumları (`T!E`), tek karakterlik operatörlerle (`?`, `^`) yönetilen birinci sınıf değer tipleridir.
4. **Sürücüsüz Yerleşik Çalışma Zamanı:** Harici kütüphanelere ihtiyaç duymadan doğrudan çalışan bir anahtar-değer veritabanı ve HTTP sunucusu içerir.

---

## Sonraki Adımlar

- `myelinc` derleyicisini kurup ilk programınızı çalıştırmak için [Kurulum ve Derleyici](/tr/guide/kurulum) bölümüne geçin.
- Temel direktifleri ve veri tiplerini öğrenmek için [Sözdizimi ve Girinti](/tr/guide/sozdizimi) sayfasını inceleyin.
