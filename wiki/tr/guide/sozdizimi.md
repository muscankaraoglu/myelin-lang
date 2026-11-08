# Sözdizimi ve Girinti Kuralları

Myelin sözdizimi, minimum token harcaması ve belirsizlik içermeyen deterministik LL(1) ayrıştırma için özel olarak geliştirilmiştir.

## Direktifler (0. Sütunda `#`)

Myelin'deki tüm üst seviye bildirimler **kesinlikle 0. sütunda `#` işaretiyle başlamak zorundadır**. Bu sayede derleyici ve LLM'ler hiçbir belirsizlik yaşamadan durumu takip eder.

### `#mod <yol>`
Modül ad alanını belirler:
```myelin
#mod muhasebe.defter
```

### `#use <yol>`
Başka bir modüldeki sembolleri içe aktarır:
```myelin
#use std.io
#use std.db
```

### `#rec <Isim> [alan:Tip ...]`
Düz kayıt tipi (struct) tanımlar:
```myelin
#rec Kullanici [id:u64 aktif:bool isim:str bakiye:i64]
```

### `#sum <Isim> [Varyant ...]`
Cebirsel toplam tipi (tagged enum) tanımlar:
```myelin
#sum OdemeDurumu [Beklemede Isleniyor Basarili Basarisiz(str)]
```

### `#fn <isim> [parametreler -> DonusTipi] !yetenekler`
Fonksiyon tanımlar. Fonksiyon gövdeleri 2 boşluk içeriden başlar:
```myelin
#fn vergi_hesapla [gelir:i64 oran:f64 -> f64]
  vergi = gelir * oran
  = vergi
```

### `#test "<aciklama>" !yetenekler`
Satır içi birim testi tanımlar:
```myelin
#test "vergi hesabi dogru calisiyor"
  sonuc = vergi_hesapla 1000 0.20
  ? sonuc == 200.0
```

---

## 2-Boşluk Girinti Kuralı

Myelin'de girinti disiplini zorunludur:
- Girintiler **kesinlikle 2'nin katı sayıda boşluk** olmalıdır (2, 4, 6, 8...).
- Tek sayılı girintiler (1 veya 3 boşluk gibi) `E002` derleme hatası verir.
- Tab karakteri (`\t`) kesinlikle yasaktır ve `E001` hatası üretir.
- Boş satırlar ve yorum satırları blok yapısını bozmaz.

```myelin
#fn ornek [] !io
  -- Dogru (2 bosluk)
  a = 10
  b = 20
  @if a < b
    -- Dogru (4 bosluk)
    io.println "a kucuktur"
```

---

## Yorum Satırları

Tek satırlık yorumlar `--` ile başlar ve satır sonuna kadar devam eder:

```myelin
-- Bu bir Myelin yorum satırıdır
x = 100 -- Satır içi yorum
```

---

## İlkel Veri Tipleri

| Tip | Örnek | Açıklama |
| :--- | :--- | :--- |
| `i8` .. `i64` | `42`, `-10` | İşaretli sabit genişlikli tam sayılar |
| `u8` .. `u64` | `100`, `0` | İşaretsiz tam sayılar |
| `f32`, `f64` | `3.14`, `-0.5` | Ondalıklı kayan noktalı sayılar |
| `bool` | `true`, `false` | Mantıksal doğruluk değerleri |
| `str` | `"merhaba"` | UTF-8 metin dizgisi |
| `?T` | `none`, `42` | İsteğe bağlı değer (Optional) |
| `T!E` | `deger ? ^Hata` | Sonuç / Hata birleşimi (Result) |
