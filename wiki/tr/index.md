---
layout: home

hero:
  name: "Myelin Dili"
  text: "Büyük Dil Modelleri İçin Yerel Sistem Dili"
  tagline: "Yapay zeka ajanları tarafından yazılmak, makinelerce makine hızında çalıştırılmak ve insanlar tarafından tek satır kod okunmadan denetlenmek üzere sıfırdan tasarlandı."
  actions:
    - theme: brand
      text: Başlangıç Rehberi →
      link: /tr/guide/getting-started
    - theme: alt
      text: Sözdizimi Turu
      link: /tr/guide/sozdizimi
    - theme: alt
      text: 🇬🇧 English Documentation
      link: /

features:
  - icon: ⚡
    title: "%55-65 Daha Az Token Tüketimi"
    details: İnsan ergonomisine ait süslü parantez ve basmakalıp sözcükleri kaldırarak LLM tokenizerları (o200k, cl100k) ile tam uyum sağlar.
  - icon: 🛡️
    title: Sıfır Gizli Yetki (Zero Ambient Authority)
    details: Saf fonksiyonlar açıkça izin almadan (!net, !db, !fs) disk, ağ veya bellek operasyonları gerçekleştiremez.
  - icon: 🎯
    title: Tek Satır LLM Hata Onarımı
    details: Derleyici hata ve uyarıları doğrudan LLM'lerin kendi kodlarını onarabilmesi için optimize edilmiş tek satırlık formattadır (--llm).
  - icon: 🌐
    title: Yerleşik HTTP & Anahtar-Değer Veritabanı
    details: Hiçbir harici bağımlılığa ihtiyaç duymayan yerleşik veritabanı (db.set, db.get) ve yüksek performanslı HTTP sunucusu (http.serve).
  - icon: 📏
    title: Düz Satır-SSA & Deterministik LL(1)
    details: Tüm üst seviye tanımlar 0. sütunda '#' ile, kontrol akışları '@' ile başlar. Belirsizlik içermeyen kusursuz dilbilgisi.
  - icon: 🚀
    title: Mikrosaniye Düzeyinde Derleme
    details: Rust ile yazılmış sıfır-kopyalama lexer ve parser sayesinde ajan döngülerinde anında sonuç.
---

## Myelin Koduna Hızlı Bir Bakış

```myelin
#mod bank.ledger
#use std.db

#rec Account [id:u64 active:bool bal:i64]
#sum TxErr   [BadAmt Inactive LowBal(i64) Db(str)]

#fn withdraw [uid:u64 amt:i64 -> i64!TxErr] !db.rw
  req amt > 0
  ens ret >= 0
  acc = db.get_acc uid ? ^TxErr.Db
  ? acc.active         | ^TxErr.Inactive
  ? acc.bal >= amt     | ^TxErr.LowBal(acc.bal)
  nxt = acc.bal - amt
  _   = db.set_bal uid nxt ? ^TxErr.Db
  = nxt

#test "withdraw updates balance" !mock.db
  db.seed_acc 1 true 500
  rem = withdraw 1 200 ?
  ? rem == 300
```
