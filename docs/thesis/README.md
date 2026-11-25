# Thesis archive

Material académico relacionado con el análisis Kramers–Kronig y Lightmatter.

## original/
Tesis original (LaTeX + PDF), figuras, presentación, libros de referencia y tutorial RefFIT.

## andes/
Tesis de grado — Universidad de los Andes (ULA), Facultad de Ciencias, Departamento de Física Aplicada, Mérida, Venezuela.
Archivo principal: `andes/documento.pdf`
Fuentes: `documento.tex`, `parte1.tex`, `parte2.tex`, `parte3.tex`, `anexos.tex`, `bibliografia.bib`.

Estructura alineada a tres seminarios:
1. **Seminario I** — Maxwell (vacío → medio) → ε̂ → n, k, σ, R a incidencia normal; Kramers–Kronig (`parte1`)
2. **Seminario II** — método triangular, malla/colas actuales, LM e implementación Rust (`parte2`)
3. **Seminario III (final)** — validación A–G, caso `R1.DAT` con el binario Rust, comparación con RefFIT, conclusiones (`parte3` + anexos)

PDFs generados:
- `andes/documento.pdf` — tesis ULA (leer esta)
- `original/documento.pdf` — borrador antiguo (sin cotutor en portada)

Tras editar fuentes, recompilar (TeX Live / `pdflatex`+`bibtex`, o `tectonic`):

```
cd docs/thesis/andes
tectonic -X compile documento.tex
# o: pdflatex documento && bibtex documento && pdflatex documento && pdflatex documento
```
