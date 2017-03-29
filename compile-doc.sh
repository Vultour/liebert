#!/bin/bash
#set -e

function compile {
    for file in ./*.tex; do pdflatex -halt-on-error $file; done
}

function bib {
    for file in ./*.aux; do bibtex $file; done
}

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

compile
bib
compile
compile

mv -t dist/ *.pdf

rm -f *.bbl
rm -f *.aux
rm -f *.blg
rm -f *.lof
rm -f *.log
rm -f *.out
rm -f *.toc