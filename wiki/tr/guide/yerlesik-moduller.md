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

---

## 5. `term` — Terminal Renklendirme ve Biçimlendirme

`term` modülü, sıfır çalışma zamanı ek yüküyle modern ve profesyonel komut satırı (CLI) arayüzleri oluşturmak için ANSI terminal renkleri ve stil yardımcıları sunar (`!term` yetkisi gerektirir).

### Kullanılabilir Fonksiyonlar:
- `term.green [metin:str -> str]`: Metni ANSI yeşil renge bürür (`\x1b[32m ... \x1b[0m`).
- `term.red [metin:str -> str]`: Metni ANSI kırmızı renge bürür (`\x1b[31m ... \x1b[0m`).
- `term.yellow [metin:str -> str]`: Metni ANSI sarı renge bürür (`\x1b[33m ... \x1b[0m`).
- `term.blue [metin:str -> str]`: Metni ANSI mavi renge bürür (`\x1b[34m ... \x1b[0m`).
- `term.cyan [metin:str -> str]`: Metni ANSI camgöbeği (cyan) renge bürür (`\x1b[36m ... \x1b[0m`).
- `term.magenta [metin:str -> str]`: Metni ANSI mor renge bürür (`\x1b[35m ... \x1b[0m`).
- `term.gray [metin:str -> str]` / `term.dim [metin:str -> str]`: Metni ANSI soluk gri renge bürür (`\x1b[90m ... \x1b[0m`).
- `term.bold [metin:str -> str]`: Metni ANSI kalın (bold) yazı tipine dönüştürür (`\x1b[1m ... \x1b[0m`).
- `term.reset`: Standart ANSI sıfırlama dizisini döndürür (`\x1b[0m`).

Stiller birbirinin içine rahatlıkla gömülebilir (örneğin: `term.bold (term.green "BASARILI")`).

### Örnek:

```myelin
#mod main

#fn main [] !io !term
  baslik = term.bold (term.cyan "=== MYELIN SISTEM DENETIM RAPORU ===")
  io.println baslik
  io.println (term.green "  [GECTI] Sozcuksel degismez kontrolu (0 tab karakteri)")
  io.println (term.yellow "  [UYARI] Girinti derinligi 4 bosluk olarak olculdu")
  io.println (term.red   "  [HATA] 0 yetki ihlali tespit edildi")
```

---

## 6. `cli` — Komut Satırı Argümanları ve Bayraklar

`cli` modülü, program çalıştırılırken `.myl` dosya adından sonra verilen komut satırı argümanlarını ve seçenek bayraklarını çözümlemeyi sağlar (`!cli` yetkisi gerektirir).

### Kullanılabilir Fonksiyonlar:
- `cli.args`: Tüm argüman ve bayrakları liste (`[]str`) olarak döndürür.
- `cli.args [indeks:i64]`: Belirtilen indeksteki argümanı döndürür (aralık dışındaysa hata fırlatır).
- `cli.count` / `cli.arg_count`: Toplam argüman sayısını `i64` olarak verir.
- `cli.flag_bool [bayrak:str -> bool]` / `cli.has_flag [bayrak:str -> bool]`: Belirtilen bayrağın argümanlar arasında olup olmadığını bildirir.
- `cli.flag_str [bayrak:str, varsayilan:str -> str]`: Bayrağın hemen ardından gelen metin değerini okur; bayrak yoksa varsayılan değeri döner.
- `cli.flag_int [bayrak:str, varsayilan:i64 -> i64]`: Bayrağın hemen ardından gelen tam sayı değerini okur; bayrak yoksa varsayılan değeri döner.

### Kapsamlı CLI Uygulaması Örneği:

```myelin
#mod main

#fn main [] !io !cli !term
  io.println (term.bold (term.cyan "=== Myelin Mikro-Servis Derleme Araci ==="))

  args = cli.args
  arg_count = args.len
  io.println (str.fmt "Alinan komut satiri arguman sayisi: {}" arg_count)

  -- Bayraklari cozumle
  is_release = cli.flag_bool "--release"
  target     = cli.flag_str "--target" "x86_64"
  workers    = cli.flag_int "--jobs" 4

  io.println (str.fmt "Hedef Mimari:        {}" target)
  io.println (str.fmt "Isci Is Parcaciklari:{}" workers)

  @if is_release
    io.println (term.green "Profil: RELEASE (Optimizasyon ve LTO aktif)")
  @el
    io.println (term.yellow "Profil: DEV (Hata yakalama ve yetenek izleri aktif)")
```

Bu betiği argümanlar vererek çalıştırın:

```bash
myelinc run derleme_araci.myl --release --target arm64 --jobs 8
```

Çıktı:
```text
=== Myelin Mikro-Servis Derleme Araci ===
Alinan komut satiri arguman sayisi: 5
Hedef Mimari:        arm64
Isci Is Parcaciklari:8
Profil: RELEASE (Optimizasyon ve LTO aktif)
-- finished in 329 µs
```

---

## 6. `json` — Saf ve Deterministik JSON Motoru

`json` modülü, sıfır harici bağımlılıkla doğrudan Myelin çalışma zamanına gömülü saf (pure) bir JSON parser ve serileştiricisidir. Hiçbir yan etki veya sistem sızıntısı üretmediği için **sıfır yetki** gerektirir (100% pür).

### Metotlar:
- `json.parse [json_str -> doc]`: JSON dizgesini Myelin harita/nesne yapısına ayrıştırır.
- `json.stringify [val -> str]`: Myelin değerini standart JSON dizgesine dönüştürür.
- `json.get [doc, path_str -> val]`: Noktalı yol (path) araması yapar (`json.get user "adres.sehir"`).
- `json.set [doc, path_str, new_val -> doc]`: Harita alanını günceller.
- `json.has [doc, key:str -> bool]`: Anahtarın varlığını sorgular.

### Örnek:

```myelin
#mod main

#fn main [] !io
  raw = "{\"kullanici\":\"Can\",\"roller\":[\"mimar\",\"kurucu\"],\"aktif\":true}"
  doc = json.parse raw

  ad = json.get doc "kullanici"
  ilk_rol = json.get doc "roller.0"
  io.println "Kullanici:" ad "Birincil Rol:" ilk_rol

  guncel = json.set doc "versiyon" "v1.0"
  io.println "Serilestirilmis JSON:" (json.stringify guncel)
```

---

## 7. `gui` — Yerel Hafif Masaüstü Arayüzü (Native Desktop GUI)

Myelin, sıfır Electron şişkinliğiyle doğrudan işletim sisteminin yerel pencere ve kontrol API'lerini kullanır (`!gui` yetkisi gerektirir). Windows üzerinde harici hiçbir DLL veya MSVC derleyicisi gerektirmeden doğrudan Win32 kontrolleri (Pencere, Düğme, Metin, Dialog) oluşturur; 0ms başlangıç süresi sunar.

### Metotlar:
- `gui.alert [title:str, msg:str]`: Yerel işletim sistemi uyarı kutusu gösterir.
- `gui.confirm [title:str, msg:str -> bool]`: Yerel Evet/Hayır onay kutusu açar.
- `gui.window [title:str, width:i64, height:i64 -> win_id]`: Yeni bir yerel pencere tanımlar.
- `gui.text [win, text:str, x:i64, y:i64, w:i64, h:i64]`: Pencere içine statik metin/etiket ekler.
- `gui.button [win, text:str, x:i64, y:i64, w:i64, h:i64, on_click_fn:str]`: Tıklama olayına bağlı yerel düğme ekler.
- `gui.show [win]`: Pencereyi görüntüler ve yerel olay döngüsünü (event loop) çalıştırır.

### Örnek:

```myelin
#mod main

#fn kaydet [] !gui !db
  _ = db.set "durum" "aktif"
  gui.alert "Myelin App" "Veri tabanina kaydedildi!"

#fn main [] !gui !db !io
  io.println "Masaustu arayuzu baslatiliyor..."
  win = gui.window "Myelin Yerel Pencere" 450 260
  gui.text win "Sifir Electron, Saf Yerel Hiz!" 25 25 400 24
  gui.button win "Kaydet" 25 80 130 36 "kaydet"
  gui.show win
```
