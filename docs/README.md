# Opalescent — Language Design Dossier (research draft)

**Status:** pre-specification. No syntax or semantic choice in this directory is implemented merely by appearing here. This branch is documentation-only. Normative guarantees are *proposals*, not current compiler guarantees.

## Reading order
1. [00 Vision and constraints](00-VISION.md)
2. [01 Landscape and source research](01-LANDSCAPE.md)
3. [02 Language charter and design principles](02-CHARTER.md)
4. [03 Syntax and surface design](03-SYNTAX.md)
5. [04 Types and effects](04-TYPES.md)
6. [05 Ownership and memory](05-OWNERSHIP.md)
7. [06 Execution, safety and interoperability](06-EXECUTION.md)
8. [07 Architecture and diagnostics](07-ARCHITECTURE.md)
9. [08 Conformance and evaluation](08-CONFORMANCE.md)
10. [09 Open decisions](09-DECISIONS.md)
11. [10 Research roadmap](10-ROADMAP.md)
12. [References](REFERENCES.md)

## Document status vocabulary
- **DECIDED-PROVISIONAL:** working rule to design against, changeable by ADR.
- **CANDIDATE:** one or more competing approaches; illustrative, non-normative.
- **OPEN:** must be resolved before stable language specification.
- **DEFERRED:** consciously out of initial scope.
- **REJECTED:** contrary to current language charter.

Decision records belong in `docs/adr/`; acceptance requires a motivating counterexample, trade-offs, effects on the spec and a conformance test. A standalone `SPEC.md` should only become normative after the research gates.
