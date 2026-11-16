# Thesis archive

Material académico relacionado con el análisis Kramers–Kronig y Lightmatter.

## original/
Tesis original (LaTeX + PDF), figuras, presentación, libros de referencia y tutorial RefFIT.

## andes/
Tesis de grado — Universidad de los Andes (ULA), Facultad de Ciencias, Departamento de Física Aplicada, Mérida, Venezuela.
Archivo principal: `andes/documento.pdf`
Fuentes: `documento.tex`, `parte1.tex`, `parte2.tex`, `parte3.tex`, `anexos.tex`, `bibliografia.bib`.

Estructura alineada a tres seminarios:
1. **Seminario I** — fundamentos y Kramers–Kronig (`parte1`)
2. **Seminario II** — método triangular, LM e implementación (`parte2`)
3. **Seminario III (final)** — validación A–G, comparación con RefFIT, conclusiones (`parte3` + anexos)

Compilar Andes:
```
cd docs/thesis/andes
pdflatex documento
bibtex documento
pdflatex documento
pdflatex documento
```
