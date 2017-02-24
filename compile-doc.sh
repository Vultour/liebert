#!/bin/bash
set -e

DIR=$(dirname $0);

cd $DIR/doc;

rm -f *.bbl
rm -f *.aux
rm -f *.blg
rm -f *.lof
rm -f *.log
rm -f *.out
rm -f *.toc
rm -f *.pdf

pdflatex -halt-on-error *.tex
bibtex *.aux
pdflatex -halt-on-error *.tex
pdflatex -halt-on-error *.tex

mv -t dist/ *.pdf

rm -f *.bbl
rm -f *.aux
rm -f *.blg
rm -f *.lof
rm -f *.log
rm -f *.out
rm -f *.toc