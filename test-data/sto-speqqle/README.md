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

## Referencia de Speqqle

- ε∞ = 5.3145
- Osciladores (ω₀, ωₚ, γ en cm⁻¹):
  - 92.76, 1540.54, 8.61
  - 175.86, 347.75, 3.13
  - 544.89, 637.52, 19.78

## Reproducir el ajuste

Desde la raíz del repo:

```bash
cargo run --release --example spectrum_bench -- r test-data/sto-speqqle/STO_R_stitched.dat 2000 half auto 100 /tmp/sto_half.tsv
```

Argumentos: medida `r`, 2000 iteraciones, malla `half` (un ancla cada dos intervalos, N/2), solver automático,
penalización de ε₂ negativa 100, y volcado a `/tmp/sto_half.tsv`.

Resultado con esa configuración (ε∞ libre):

| Magnitud | Speqqle | Lightmatter (N/2) |
|---|---|---|
| Fonón 1, pico de ε₂ | 92.8 cm⁻¹ | 92.8 cm⁻¹ |
| Fonón 2, pico de ε₂ | 175.9 cm⁻¹ | 175.7 cm⁻¹ |
| Fonón 3, pico de ε₂ | 544.9 cm⁻¹ | 545.1 cm⁻¹ |
| ε∞ | 5.31 | 5.465 |
| R del modelo > 1 | — | no |
| RMSE | — | 2.6e-3 |

Anclas: 2345. Iteraciones: 57. Convergió.

## Notas

- **ε₂ negativa:** 882 de 2343 puntos interiores. Casi todos están por encima de ~1500 cm⁻¹, donde ε₂ ≈ 0,
  con valores de −0.006 a −0.04 frente a un pico de ~1500. Es ruido numérico en una zona transparente.
  Subir la penalización (1000, 10000) no lo elimina y empeora el RMSE (3.2e-3 y 1.2e-2).
  El único punto relevante está en −5.1 cerca de 27 cm⁻¹.
- **ε∞:** el ajuste libre da 5.465; el de Speqqle es 5.31. Fijarlo a 5.314 está pendiente.
- **Malla completa (N):** con 4685 anclas hay 4686 coeficientes para 4687 puntos, lo que tiende a sobreajustar.
  En la web, con esa malla, la tabla de picos solo reportó el de 90 cm⁻¹. Con N/2 los tres fonones aparecen
  como máximos de ε₂ en la posición de Speqqle. Por eso se usa N/2.
