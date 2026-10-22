# QEL / BEST docs

Standalone browser documentation and demos for the QEL/PGES temporal query engine. Everything in this folder is static HTML, CSS, and JavaScript. The pages use synthetic records only, run entirely in the browser, and do not call the Rocket/MongoDB backend.

## Quick start

From the repository root:

```bash
python3 -m http.server 8123 --directory docs
```

Open:

- <http://localhost:8123/qel-reproducibility.html> — narrative guide
- <http://localhost:8123/qel-playground.html> — interactive cohort playground
- <http://localhost:8123/qel-algorithm.html> — Algorithms 1–3 walkthrough

`file://` works for basic viewing, but the playground needs HTTP to fetch `pges-manual-eval.json`. There is no build step, package install, CDN dependency, or backend requirement.

## Files

| File | Purpose |
| --- | --- |
| `qel-reproducibility.html` | Self-contained guide to interval ensembles, endpoint arithmetic, PGES formulas, strict semantics, and backend mappings. |
| `qel-playground.html` | Interactive cohort builder with 10 synthetic BEST records, GTC/PGES presets, a suppression filter, two solver paths, and manual evaluation loading. |
| `qel-algorithm.html` | Interactive trace of Algorithms 1–3 on record P01: primitives, model checking, and finite-evidence scanning. |
| `qel-query-core.js` | Shared JavaScript engine. Exposes `window.QELCore` and is loaded by the playground and algorithm page. |
| `pges-manual-eval.json` | 54-case manual evaluation set, version 3.0.0, with expected base and strict outcomes. |

## What each page covers

### Reproducibility guide

`qel-reproducibility.html` explains the data model and formulas without requiring the backend:

- BEST labels: `S` seizure, `T` tonic, `C` clonic, `P` EEG suppression.
- Canonical interval ensembles with open and closed endpoints.
- Displacement, dilation, erosion, complement, intersection, and union.
- The permissive base PGES formula and the exact strict PGES formula.
- Worked P01 witness and timeline.
- Mapping of interval primitives to backend functions in `eeg_query_api.rs`.
- Evaluation-set counts and inspection workflow.

### Playground

`qel-playground.html` is the interactive cohort query page:

- Built-in dataset: 10 synthetic BEST records over 0–180 s.
- Presets: GTC, PGES base, PGES strict.
- `c >= n` witness filter for suppression duration.
- Solver selector:
  - **Optimized witness search** — endpoint-derived candidates verified with interval support.
  - **Paper finite-cell (Algorithm 3)** — finite-cell style candidate generation, also verified with interval support.
- Audit panel with the selected witness, per-conjunct checks, and matched pattern details.
- Manual evaluation mode: load `pges-manual-eval.json` or use the fetch button.

### Algorithms 1–3

`qel-algorithm.html` runs the three paper algorithms on record P01:

- **Algorithm 1**: apply one interval-ensemble primitive and inspect the normalized result.
- **Algorithm 2**: translate a candidate, substitute `q = x`, eliminate auxiliary variables, and evaluate every conjunct.
- **Algorithm 3**: enumerate observed boundary quadruples and verify the full PGES body support.

The page uses the new cool-gray visual system. The playground and reproducibility pages retain their existing visual style.

## Core semantics

`qel-query-core.js` implements a canonical interval-ensemble algebra:

```text
atom        -> stored support
and         -> intersection
or          -> union
not         -> complement
box(t)      -> erosion by t
diamond(t)  -> dilation by t
shift(u)    -> displacement by u
```

Every operation returns a normalized ensemble. Normalization removes empty intervals and merges overlapping or adjacent compatible intervals.

The PGES body is:

```text
□_{a+b}S ∧ ◊_aT ∧ (◊_bC)_a ∧ (□_cP)_{a+b}
```

The strict body adds a 10 s no-event prefix and the exclusions `S∧¬P`, `T∧¬C`, `C∧¬T`, and `P∧¬S`. Strict supports are evaluated as half-open intervals, so adjacent boundaries meet rather than overlap.

A candidate is accepted when the formula support contains the reference point `x`.

## Solver paths

`QELCore` exports two search paths:

| Function | Role |
| --- | --- |
| `findMatches(record, query)` | Optimized endpoint-candidate search used by default. |
| `findMatchesFiniteCell(record, query)` | Paper finite-cell path, using `finiteCellPoints` candidates. |
| `finiteCellPoints(record, labels)` | Builds affine boundary candidates for Algorithm 3. |

Both paths use the same `support` verifier. The finite-cell path is a specialized Algorithm 3 implementation, not a generic quantifier-elimination engine.

Numeric comparisons use `EPS = 1e-6` and `TOL = 1e-9`. They are not exact rational arithmetic, so filtering very close to a boundary can differ by a tolerance-scale amount.

## Manual evaluation set

`pges-manual-eval.json` contains one dataset evaluated by two formulas:

- `queries.base`: permissive PGES reference query.
- `queries.strict`: exact strict PGES formula with a 10 s quiet prefix and in-formula exclusions.
- `cases[]`: 54 cases, each with `events`, `expected`, and `expected_strict`.
- `strict_quiet_seconds`: `10`.

At `c >= 0`, both formulas should agree with every expected case. The playground reports patient and pattern counts separately. The set is synthetic and is not the paper's clinical cohort.

