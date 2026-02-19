LATEXMK ?= latexmk
TECTONIC ?= /tmp/tectonic-bin/tectonic

.PHONY: pdf pdf-tectonic clean package

pdf:
	$(LATEXMK) -pdf -interaction=nonstopmode main.tex

pdf-tectonic:
	$(TECTONIC) main.tex

clean:
	$(LATEXMK) -C

package:
	./package_for_arxiv.sh
