# Yerleşik Modüller ve Çalışma Zamanı

Myelin, `myelinc` derleyicisinin içerisine doğrudan gömülmüş **sürücüsüz (driverless) bir çalışma zamanı** ile gelir. Harici hiçbir paket veya C kütüphanesi kurmadan mikroservisler ve veri depolama işlemleri geliştirebilirsiniz.

---

## 1. `io` — Konsol ve Hata Ayıklama

Konsola veri yazdırma işlevleri (`!io` yetkisi gerektirir).

### `io.println [arg1, arg2, ...]`
Değerleri aralarına boşluk koyarak ekrana yazdırır ve yeni satıra geçer:

```myelin
#fn main [] !io
  io.println "Sistem baslatildi, sayac degeri:" 42
```

---

## 2. `str` — Metin Biçimlendirme

Hızlı ve güvenli metin şablonlama fonksiyonları.

### `str.fmt [sablon, arg1, arg2, ...]`
Şablon içerisindeki `{}` işaretlerini sırasıyla parametrelerin metin halleriyle değiştirir:

```myelin
#fn karsilama_metni [isim:str rol:str -> str]
  metin = str.fmt "Sayin {}, goreviniz: {}" isim rol
  = metin
```

---

## 3. `db` — Yerleşik Anahtar-Değer Veritabanı

Bellek içi (in-memory), iş parçacığı güvenli (thread-safe) ve sıfır bağımlılıklı anahtar-değer veritabanı motoru (`!db` yetkisi gerektirir).

### Kullanılabilir Fonksiyonlar:
- `db.set [anahtar:str, deger:str]`: Belirtilen anahtara değer atar.
- `db.get [anahtar:str -> str]`: Anahtarın değerini döndürür (anahtar yoksa `"none"` döner).
- `db.has [anahtar:str -> bool]`: Anahtarın veritabanında olup olmadığını mantıksal (`true`/`false`) olarak bildirir.

### Örnek:

```myelin
#mod main

#fn main [] !db !io
  -- Verileri kaydet
  _ = db.set "kullanici:1:isim" "Can"
  _ = db.set "kullanici:1:rol" "Sistem Mimari"

  -- Verileri oku
  isim = db.get "kullanici:1:isim"
  rol  = db.get "kullanici:1:rol"

  rapor = str.fmt "Kullanici: {} - Rol: {}" isim rol
  io.println rapor

  -- Varlık kontrolü
  var_mi = db.has "kullanici:1:isim"
  io.println "Kullanici veritabaninda kayitli mi:" var_mi
```

---

## 4. `http` — Yerleşik Web ve Mikroservis Sunucusu

Myelin, harici web sunucularına (Express, Nginx, Apache) ihtiyaç duymadan doğrudan HTTP isteklerini dinleyebilen bir yerel sunucuya sahiptir (`!net` yetkisi gerektirir).

### `http.serve [port:i64, dinleyici_fonksiyon:str]`
Belirtilen portta (örneğin `127.0.0.1:8080`) bir HTTP sunucusu açar ve gelen her isteği belirtilen Myelin fonksiyonuna yönlendirir.

Dinleyici fonksiyon parametre olarak istek yolunu (`path:str`) alır ve yanıt gövdesini (`str`) döndürür.

### Tam Örnek:

```myelin
#mod main

#fn istek_yonetici [path:str -> str]
  @if path == "/"
    = "Myelin Web API'sine Hos Geldiniz! Sifir harici bagimlilik."
  @elif path == "/durum"
    = "{\"durum\":\"aktif\",\"motor\":\"myelinc-v0.1.0\",\"surucu\":\"yerlesik\"}"
  @el
    = str.fmt "404 Bulunamadi: '{}' yolu tanimli degil" path

#fn main [] !net !io
  io.println "Myelin HTTP Sunucusu 8080 portunda baslatiliyor..."
  _ = http.serve 8080 "istek_yonetici"
```

Bu kodu `myelinc run web_api.myl` komutuyla çalıştırıp tarayıcınızdan veya curl ile test edebilirsiniz:
```bash
curl http://127.0.0.1:8080/durum
```
Dönen yanıt:
```json
{"durum":"aktif","motor":"myelinc-v0.1.0","surucu":"yerlesik"}
```
