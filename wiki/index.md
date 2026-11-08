---
layout: home

hero:
  name: "Myelin Lang"
  text: "The LLM-Native Systems Programming Language"
  tagline: "Designed from first principles to be written by AI agents, executed at bare-metal speed, and audited by humans without reading a single line of code."
  actions:
    - theme: brand
      text: Get Started →
      link: /guide/getting-started
    - theme: alt
      text: Quick Syntax Tour
      link: /guide/syntax-guide
    - theme: alt
      text: 🇹🇷 Türkçe Dokümantasyon
      link: /tr/

features:
  - icon: ⚡
    title: 55-65% Fewer Tokens
    details: Eliminates human-ergonomic boilerplate (braces, async wrappers, nested types). Token-aligned with modern LLM tokenizers (o200k, cl100k).
  - icon: 🛡️
    title: Zero Ambient Authority
    details: Pure functions cannot perform network, disk, or memory operations without explicit capability flags (!net, !db, !fs, !unsafe.mem).
  - icon: 🎯
    title: Single-Line LLM Diagnostics
    details: Instant compiler feedback formatted specifically for LLM autoregressive context repair (--llm wire format).
  - icon: 🌐
    title: Built-in Microservices & DB
    details: Native zero-dependency embedded database (db.set, db.get) and high-performance HTTP web server (http.serve).
  - icon: 📏
    title: Flat Line-SSA & Deterministic LL(1)
    details: Top-level items start with '#' at column 0; control-flow starts with '@'. Fully parsed with zero ambiguous lookahead.
  - icon: 🚀
    title: Microsecond Compilation
    details: Sub-millisecond parser and interpreter written in Rust for lightning-fast agent test-and-repair loops.
---

## A Taste of Myelin

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
