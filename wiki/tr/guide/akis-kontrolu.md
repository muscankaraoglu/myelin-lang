# Akış Kontrolü ve Guard Yapısı

Myelin'de kontrol akışları, iç içe geçmiş kod piramitlerini ve süslü parantez kalabalığını ortadan kaldırmak için tasarlanmıştır. Tüm kontrol akışı anahtar kelimeleri `@` ile başlar.

## If / Elif / El Koşul Yapısı

Koşullu dallanma `@if`, `@elif` ve `@el` ile yapılır:

```myelin
#fn not_harfi [puan:i64 -> str]
  @if puan >= 90
    = "AA"
  @elif puan >= 80
    = "BA"
  @elif puan >= 60
    = "CC"
  @el
    = "FF"
```

Özellikler:
- Koşulun etrafında parantez kullanılmaz (`(puan >= 90)` yerine doğrudan `puan >= 90`).
- Süslü parantez yoktur; blok kapsamı 2 boşluk girinti ile belirlenir.
- Fonksiyondan değer döndürmek için `= <değer>` ifadesi kullanılır.

---

## Guard Yapısı (`?`)

Guard ifadeleri, kodunuzu derin `@if` bloklarına boğmadan erken çıkış (early return) ve doğrulama yapmanızı sağlar.

Sözdizimi:
```myelin
? <kosul> | <eylem>
```

Kullanılabilecek eylemler:
- `^ <Hata>`: Hata fırlatarak erken dön
- `= <Deger>`: Başarılı değer ile erken dön
- `@brk`: Döngüyü kır ve sonlandır
- `@next`: Döngünün bir sonraki adımına geç

### Örnek: Bakiye ve Tutar Kontrolü

```myelin
#fn transfer [bakiye:i64 tutar:i64 -> i64!str]
  -- Tutarın pozitif olduğunu doğrula, değilse hata fırlat
  ? tutar > 0 | ^"tutar sifirdan buyuk olmalidir"

  -- Yeterli bakiye olduğunu doğrula
  ? bakiye >= tutar | ^"yetersiz bakiye"

  kalan = bakiye - tutar
  = kalan
```

Koşul `true` ise yürütme bir sonraki satırdan kesintisiz devam eder. Koşul `false` ise boru simgesinin (`|`) sağındaki eylem derhal çalıştırılır.

---

## Döngüler (`@for` ve `@loop`)

Myelin iki temel döngü biçimi sunar:

### Yineleme Döngüsü (`@for`)

```myelin
#fn sayilari_yaz [] !io
  @for i : [1, 2, 3, 4, 5]
    io.println i
```

### Sonsuz / Koşullu Döngü (`@loop`)

```myelin
#fn geri_sayim [baslangic:i64] !io
  n = baslangic
  @loop
    ? n > 0 | @brk
    io.println n
    n = n - 1
```

Döngüden çıkmak için `@brk`, sonraki adıma atlamak için `@next` kullanılır.

---

## Desen Eşleme (`@match`)

Cebirsel tiplerin kapsamlı ve kesin eşlenmesi:

```myelin
#fn durum_metni [durum:OdemeDurumu -> str]
  @match durum
    OdemeDurumu.Beklemede => "Onay bekleniyor"
    OdemeDurumu.Isleniyor => "Banka islemi suruyor"
    OdemeDurumu.Basarili => "Islem basariyla tamamlandi"
    OdemeDurumu.Basarisiz(sebep) => str.fmt "Hata olustu: {}" sebep
```
