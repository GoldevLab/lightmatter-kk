# SrTiO₃ — reflectividad de Speqqle

Datos de reflectividad de SrTiO₃ (bulk, incidencia normal) exportados de la sesión de Speqqle
«Reflectivity analysis STO», con la referencia de su ajuste Drude–Lorentz.

## Archivos

| Archivo | Puntos | Rango (cm⁻¹) | Contenido |
|---|---|---|---|
| `STO_Detector1.dat` | 277 | 27.5 – 95.0 | Detector 1 |
| `STO_Detector2.dat` | 1207 | 70.4 – 651.8 | Detector 2 |
| `STO_Detector3.dat` | 3303 | 632.5 – 7000.1 | Detector 3 |
| `STO_R_stitched.dat` | 4687 | 27.5 – 7000.1 | Los tres unidos (entrada de Lightmatter) |
| `STO_Speqqle_Lorentz_ref.txt` | — | — | ε∞ y osciladores de Speqqle |

Formato: dos columnas, `ω` en cm⁻¹ y `R` adimensional (0–1). Sin tercera columna de error.

## Cómo se unieron

Los solapes son 70–95 cm⁻¹ (detectores 1 y 2) y 632–652 cm⁻¹ (detectores 2 y 3). El corte está en el
punto medio de cada solape: **82.686 cm⁻¹** y **642.181 cm⁻¹**. Del detector 1 se toma lo que queda por
debajo del primer corte, del 2 lo que queda entre cortes y del 3 lo que queda por encima.

En el corte 82.7 cm⁻¹ la reflectividad salta de 0.902 (detector 1) a 0.887 (detector 2). Ese escalón
es una discrepancia entre detectores y el ajuste lo recoge como ondulaciones (ver notas).

## Referencia de Speqqle

- ε∞ = 5.3145
- Osciladores (ω₀, ωₚ, γ en cm⁻¹):
  - 92.76, 1540.54, 8.61
  - 175.86, 347.75, 3.13
  - 544.89, 637.52, 19.78

Su modelo reproduce los RMS por detector que reporta Speqqle (0.0095, 0.037 y 0.013 en los tres rangos),
así que la referencia está bien transcrita. Frente a los datos, el modelo es malo en el detector 2
(entre 400 y 650 cm⁻¹ se separa hasta 0.2 cerca de 477 cm⁻¹).

## Reproducir el ajuste

Desde la raíz del repo, configuración recomendada (N/2 y ε∞ fijada a la de Speqqle):

```bash
cargo run --release --example spectrum_bench -- r test-data/sto-speqqle/STO_R_stitched.dat 2000 half auto 100 /tmp/sto_half_fixed.tsv tails 5.314487644981076
```

Argumentos: medida `r`, 2000 iteraciones, malla `half` (un ancla cada dos intervalos, N/2), solver automático,
penalización de ε₂ negativa 100, volcado a `/tmp/sto_half_fixed.tsv`, colas `tails` y ε∞ = 5.314487644981076.

Sin ε∞ fija (libre, misma malla):

```bash
cargo run --release --example spectrum_bench -- r test-data/sto-speqqle/STO_R_stitched.dat 2000 half auto 100 /tmp/sto_half.tsv
```

## Resultados

| Magnitud | Speqqle | Lightmatter N/2, ε∞ fija | Lightmatter N/2, ε∞ libre |
|---|---|---|---|
| Fonón 1, pico de ε₂ | 92.7 cm⁻¹ (ε₂ ≈ 2972, FWHM 8.6) | 92.8 cm⁻¹ (ε₂ ≈ 1506, FWHM ≈ 17) | 92.8 cm⁻¹ (ε₂ ≈ 1512) |
| Fonón 2, pico de ε₂ | 175.9 cm⁻¹ (ε₂ ≈ 227) | 175.7 cm⁻¹ (ε₂ ≈ 136) | 175.7 cm⁻¹ (ε₂ ≈ 140) |
| Fonón 3, pico de ε₂ | 544.9 cm⁻¹ (ε₂ ≈ 38) | 545.1 cm⁻¹ (máximo local, ε₂ ≈ 53) | 545.1 cm⁻¹ (ε₂ ≈ 54) |
| ε∞ | 5.3145 | 5.3145 (fijada) | 5.465 |
| χ² | — | 3.01e-2 | 3.11e-2 |
| RMSE frente a R | 2.1e-2 | 2.5e-3 | 2.6e-3 |
| ε₂ negativa (puntos interiores) | 0 | 678 de 2343 | 882 de 2343 |
| Iteraciones | — | 42, convergió | 57, convergió |

El fonón 3 es un máximo local débil: su prominencia queda por debajo del 5 % del rango y no entra en la
tabla de picos de la web.

## Notas

- **Malla completa (N) en la web:** con 4685 anclas hay 4686 coeficientes para 4687 puntos, lo que tiende a
  sobreajustar. Da χ² 3.39e-2 y 1653 puntos con ε₂ negativa. Por eso se usa N/2.
- **Picos:** la búsqueda de picos ya no suaviza la curva. Antes, con una ventana de n/200 puntos, el pico de
  92.8 salía en 89.9 cm⁻¹ con altura 1096 y FWHM 30. Ahora sale en 92.8 con altura real.
- **Ondulaciones en 82–98 cm⁻¹:** el ajuste genera máximos espurios en 82.6, 86.1, 89.0 y 97.6 cm⁻¹, alrededor
  del corte entre detectores y del ruido del detector 1. Aparecen en la tabla de picos. No son fonones.
- **ε₂ negativa:** con N/2 quedan 678 puntos (ε₂ ≈ −2.5 mínimo), casi todos por encima de ~1500 cm⁻¹, en una
  zona transparente. Subir la penalización (1000, 10000) no lo elimina y empeora el RMSE.
- **Rango de los gráficos:** los límites de los ejes añaden un margen del 8 % del rango. Por eso la web
  muestra valores como R 1.07 o ε₂ 1634 que no son datos del ajuste (R del modelo ≤ 0.993).
