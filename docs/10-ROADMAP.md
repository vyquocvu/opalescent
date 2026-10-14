# 10 — Research-first roadmap (no feature coding in this phase)

## R0 Source review
Compare primary references and reproduce distinctive examples. Output: 01-LANDSCAPE, REFERENCES and explicit disagreements. Gate: every adopted behavior has a primary source or a reasoned original proposal.

## R1 Language charter
Priorities, target domain, non-goals, cost transparency and success measurements. Gate: incompatible goals are ranked rather than hand-waved.

## R2 Memory-model decision (critical path)
Write 3 competing ownership proposals and counterexamples; run 50+ written program judgments; select D001–D004 via ADR. Gate: no hidden cost and viable lowering story.

## R3 Core semantics
Type judgments, evaluation order, error flow, object/value identity, drop/overflow semantics; define positive/negative corpus. Gate: independent reviewer predicts outcomes consistently.

## R4 Surface grammar and diagnostics
Formal grammar + formatting + source span examples only after semantics stabilize. Gate: parsing and semantics do not depend on unspecified Ruby behavior.

## R5 Language specification candidate
Consolidate accepted ADRs into versioned normative SPEC.md; identify implementation-defined behavior. Gate: contradictory examples resolved.

## R6 Implementation planning
Decompose compiler, milestones, tests and benchmark harness. Only then update compiler source.

## Explicit stop conditions
Any essential safety guarantee dependent on unbounded runtime inference, unannounced cloning, or irreducible ambiguous type semantics sends design back to R2/R3.
