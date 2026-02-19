# Open Codelabs arXiv Technical Report Kit

This folder contains a starter package for writing and submitting an arXiv-ready technical report for Open Codelabs.

## Files

- `main.tex`: technical report starter manuscript.
- `references.bib`: starter bibliography.
- `package_for_arxiv.sh`: creates a clean source tarball for arXiv upload.
- `Makefile`: local convenience commands.

## Quick Start

```bash
cd /Users/jaichang/Documents/GitHub/open-codelabs/paper/arxiv
make pdf
```

If `latexmk` is not available, you can also compile with `tectonic`:

```bash
/tmp/tectonic-bin/tectonic main.tex
```

## arXiv Submission Checklist (Verified 2026-02-18)

Sources:
- [arXiv submission policy and accepted formats](https://arxiv.org/submit/)
- [arXiv TeX submission help](https://info.arxiv.org/help/submit_tex.html)
- [arXiv PDF submission help](https://info.arxiv.org/help/submit_pdf.html)
- [arXiv support: submission size limit](https://arxiv-org.atlassian.net/wiki/spaces/AUS/pages/64852831/How+do+I+submit+a+file+that+is+larger+than+50MB)

Checklist:
1. Submit source (`.tex` + all required assets), not only a generated PDF, when using TeX workflows.
2. Keep filenames simple: letters, digits, `_`, `-`, `.`, `+`; avoid spaces and unusual symbols.
3. Do not rely on hidden files/directories (arXiv strips hidden entries).
4. Include bibliography inputs (`.bib`) and any custom `.sty` files in the source bundle.
5. Avoid TeX setups requiring shell escape or external converters.
6. For PDF-only submissions, use a single PDF with embedded fonts and no JavaScript.
7. Keep the total upload package size within arXiv limits (support docs state 50 MB cap).
8. Use the arXiv preview/compiler output and fix every warning before final submission.
9. Avoid double-spaced referee format; arXiv may reject it.
10. Submit before the daily cutoff if you want next business-day announcement (14:00 US Eastern Time per submission policy page).

## Packaging for Upload

Create a clean source archive:

```bash
cd /Users/jaichang/Documents/GitHub/open-codelabs/paper/arxiv
./package_for_arxiv.sh
```

Default output:

- `open-codelabs-arxiv-source.tar.gz`

Custom output filename:

```bash
./package_for_arxiv.sh my-arxiv-upload.tar.gz
```

## Final Pre-Upload Pass

1. Replace placeholder author names and contact emails in `main.tex`.
2. Replace placeholder sections with measured, reproducible results.
3. Rebuild PDF and verify references, figure labels, and page breaks.
4. Upload the tarball to arXiv and validate in the arXiv preview pane.
