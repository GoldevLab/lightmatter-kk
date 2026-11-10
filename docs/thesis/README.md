# Thesis archive

Material académico relacionado con el análisis Kramers–Kronig y Lightmatter.

## original/
Tesis original (LaTeX + PDF), figuras, presentación, libros de referencia y tutorial RefFIT.

## andes/
Tesis nueva — Universidad de los Andes, Facultad de Ciencias, Departamento de Física Aplicada.
Archivo principal: `andes/documento.pdf` (fuentes: `documento.tex`, `parte1.tex`, `parte2.tex`, `parte3.tex`).

Compilar Andes:
```
cd docs/thesis/andes
pdflatex documento
bibtex documento
pdflatex documento
pdflatex documento
```
