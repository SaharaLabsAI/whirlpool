This project is a fun experiment and a playful sandbox for trying ideas quickly.
It is built for learning by tinkering, testing, and enjoying the process.

## Nightly ERC20 benchmark report (single node)

The nightly benchmark workflow publishes the latest report to GitHub Pages at a stable path:

- `https://<owner>.github.io/<repo>/benchmarks/latest.html`

Once Pages is enabled for this repository and `gh-pages` is configured, replace `<owner>` and `<repo>` with your repository values.

Policy summary:
- single-node nightly benchmark signal
- coarse regression gate: fail when `current_tps < 0.5 * rolling_7_night_median_tps`
- warm-up mode: no failure until at least 7 prior nightly samples exist
- not used for PR/merge blocking
