# 09 — Open questions and decision register

No language design item is approved merely because example syntax appears elsewhere.

| ID | Question | Options | Priority | Status |
|---|---|---|---|---|
| D001 | Ownership foundation | affine/borrow; mutable-value/access; hybrid | Critical | DECIDED-PROVISIONAL: affine/borrow (ADR 0001) |
| D002 | Parameter conventions | read/mut/take; &/&mut/move; other | Critical | DECIDED-PROVISIONAL: read/mut/take (ADR 0001) |
| D003 | Caller move notation | always explicit; inferred at last use | Critical | DECIDED-PROVISIONAL: callee-directed inference (ADR 0001) |
| D004 | Implicit cost policy | prohibit clones/heap promotion; narrowly permit with annotation | Critical | DECIDED-PROVISIONAL: prohibit (ADR 0001) |
| D005 | Binding syntax | let/mut; Ruby-like x=/var | High | OPEN |
| D006 | Type inference boundary | local only; return inference; public API explicit | High | OPEN |
| D007 | Value vs identity | values-first; reference objects | High | OPEN |
| D008 | Error model | Result/Option; exceptions; hybrid | High | OPEN |
| D009 | Dynamic features | exclude safe core; explicit dynamic compartment | High | DEFERRED |
| D010 | Module/package/coherence | file module; package; traits | High | OPEN |
| D011 | Arithmetic semantics | checked/wrapping policy | High | OPEN |
| D012 | Concurrency model | threads/capability; actors; async | High | DEFERRED |
| D013 | FFI/unsafe | scoped unsafe and contract | High | DEFERRED |
| D014 | String/Unicode | UTF-8 indexing, grapheme API, interpolation | Medium | OPEN |
| D015 | Backend | generated Rust bootstrap vs independent IR backend | High | DECIDED-PROVISIONAL: bootstrap Rust |
| D016 | Compatibility | Ruby familiarity vs Ruby semantic parity | Critical | DECIDED-PROVISIONAL: no parity |

## ADR acceptance template
Context; precise decision; alternatives; code examples including rejected examples; constraints preserved; explicit runtime cost; compatibility impact; implications for parser/HIR/runtime; test obligations; supersession link.

Do not close D001–D004 independently: they form a single ownership semantics decision.
