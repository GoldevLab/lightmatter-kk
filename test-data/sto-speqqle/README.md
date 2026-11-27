# SrTiO₃ — reflectividad de Speqqle

Datos de reflectividad de SrTiO₃ (bulk, incidencia normal) exportados de la sesión de Speqqle
«Reflectivity analysis STO», con la referencia de su ajuste Drude–Lorentz.

## Archivos

| Archivo | Puntos | Rango (cm⁻¹) | Contenido |
|---|---|---|---|
| `STO_Detector1.dat` | 277 | 27.5 – 95.0 | Detector 1 |
| `STO_Detector2.dat` | 1207 | 70.4 – 651.8 | Detector 2 |
| `STO_Detector3.dat` | 3303 | 632.5 – 7000.1 | Detector 3 |
| `STO_R_stitched.dat` | 4687 | 27.5 – 7000.1 | Los tres unidos **sin escalar** (referencia de la prueba de unión) |
| `STO_Speqqle_Lorentz_ref.txt` | — | — | ε∞ y osciladores de Speqqle |

Formato: dos columnas, `ω` en cm⁻¹ y `R` adimensional (0–1). Sin tercera columna de error.

## Cómo se unieron

Los solapes son 70–95 cm⁻¹ (detectores 1 y 2) y 632–652 cm⁻¹ (detectores 2 y 3). El corte está en el
punto medio de cada solape: **82.686 cm⁻¹** y **642.181 cm⁻¹**. Del detector 1 se toma lo que queda por
debajo del primer corte, del 2 lo que queda entre cortes y del 3 lo que queda por encima.

Los detectores no coinciden en sus solapes. Por eso `merge_spectra` reescala cada archivo al anterior con
la mediana del cociente de R en su solape: **D2 × 1.0054** (sobre D1) y **D3 × 1.0333** (sobre D2). Sin
ese escalado, el salto de ~3 % entre D2 y D3 aparece como un escalón a 652 cm⁻¹ que el ajuste no puede
reproducir. `STO_R_stitched.dat` queda sin escalar a propósito, para que la prueba de unión siga siendo
una comparación exacta con el archivo original.

## Referencia de Speqqle

- ε∞ = 5.3145
- Osciladores (ω₀, ωₚ, γ en cm⁻¹):
  - 92.76, 1540.54, 8.61
  - 175.86, 347.75, 3.13
  - 544.89, 637.52, 19.78

Su modelo reproduce los RMS por detector que reporta Speqqle (0.0095, 0.037 y 0.013 en los tres rangos),
así que la referencia está bien transcrita. Frente a los datos, el modelo es malo en el detector 2
(entre 400 y 650 cm⁻¹ se separa hasta 0.2 cerca de 477 cm⁻¹).

Estos valores son los guardados en la sesión, que no están convergidos (Speqqle paró en 20 iteraciones).
Lightmatter **no usa ε∞ de Speqqle como entrada**: ajusta ε∞ con los datos.

## Reproducir el ajuste

Desde la raíz del repo, la ruta por defecto (malla N/2, extrapolación `auto`, solver automático,
ε₂ ≥ 0 como cota, tolerancia final 1e-4). Con este espectro el extremo bajo no refleja como un conductor,
así que `auto` lleva ε₂ a cero fuera del rango. ε∞ vacío no se ajusta: se fija en n² de R en el extremo
alto (≈ 5.65). Para reproducir eso hay que unir los tres detectores con `merge_spectra` (escala el solape)
y no pasar ε∞:

```bash
cargo run --release --example spectrum_bench -- r test-data/sto-speqqle/STO_R_stitched.dat 2000 half auto 100 /tmp/sto_auto.tsv auto
```

Ese archivo está unido sin escalar, así que no es lo que hace la app. Con los tres detectores unidos por
`merge_spectra` (con escala) la ruta `auto` da, en el bench: ε∞ = 5.646, RMSE 1.39e-3, ε₂(28 cm⁻¹) ≈ 9,
pico del fonón 1 ≈ 2159, ningún ε₂ negativo. La tabla de abajo es una medida anterior, con ε∞ libre.

## Resultados

Medidas con la unión escalada, la tolerancia 1e-4 y ε∞ libre (versión anterior del código, en la que ε∞ sí se
ajustaba; la ruta actual fija ε∞ y no se ha vuelto a medir la tabla completa):

| Magnitud | Speqqle (guardado) | Lightmatter |
|---|---|---|
| Fonón 1, pico de ε₂ | 92.7 cm⁻¹ (ε₂ ≈ 2972, FWHM 8.6) | 92.8 cm⁻¹ (ε₂ ≈ 1806) |
| Fonón 2, pico de ε₂ | 175.9 cm⁻¹ (ε₂ ≈ 227) | 175.7 cm⁻¹ (ε₂ ≈ 147) |
| Fonón 3, pico de ε₂ | 544.9 cm⁻¹ (ε₂ ≈ 38) | no sale como pico con el criterio de la web |
| ε∞ | 5.3145 | 5.64 |
| RMSE frente a R | 2.1e-2 | 2.95e-3 |
| ε₂ negativa (puntos interiores) | 0 | 995 de 2343 |
| Iteraciones | — | 63, «convergió» |

**Sensibilidad del ajuste.** Con los mismos datos y los mismos parámetros, el resultado cambia más de lo que
debería:

| Caso | ε∞ | RMSE | ε₂ negativa |
|---|---|---|---|
| Sin escalar, tolerancia 1e-3 (versión anterior) | 5.46 | 2.58e-3 | 882 |
| Escalado, tolerancia 1e-3 | 5.64 | 2.95e-3 | 996 |
| Escalado, tolerancia 1e-4 (datos unidos en Python) | 5.62 | 1.94e-3 | 729 |
| Escalado, tolerancia 1e-4 (datos unidos con `merge_spectra`) | 5.64 | 2.95e-3 | 995 |

Los dos últimos usan datos que difieren en 5e-7 en R y dan resultados distintos: 167 iteraciones y RMSE
1.9e-3 en uno, 63 y 2.9e-3 en el otro. El optimizador cae en mínimos locales distintos según cambios
diminutos. Por eso no hay una configuración que deba darse por mejor sin repetir las pruebas. Un castigo
de ε₂ negativa sobre las anclas (en vez de solo sobre los datos) se probó y empeoró el resultado
(RMSE 3.2e-3, 1027 negativos); no está en el código.

## Notas

- **Inestabilidad:** `converged=true` no garantiza que el residuo esté dentro del ruido. Hubo ajustes marcados
  como convergidos con un error de 0.07 en 800 cm⁻¹, donde R cae de 0.63 a 0.03 entre 770 y 830 cm⁻¹.
- **Malla completa (N) en la web:** con 4685 anclas hay 4686 coeficientes para 4687 puntos, lo que tiende a
  sobreajustar. Da χ² 3.39e-2 y 1653 puntos con ε₂ negativa. Por eso se usa N/2.
- **ε₂ negativa:** la mayoría de los 995 puntos tienen valores de unas milésimas en la zona transparente
  (1500–7000 cm⁻¹), que son ruido de la reconstrucción. Pero el ajuste también deja un mínimo de **−4.7 en
  28.6 cm⁻¹**, en el borde de baja frecuencia, que no es físico. Ese punto no se ve en el residuo de R.
- **Sesgo del residuo:** entre 700 y 4000 cm⁻¹ el modelo queda por encima de los datos (+0.0028 de media).
- **Picos:** la búsqueda de picos no suaviza la curva. Con prominencia ≥ 5 % del rango, el fonón 1 sale en
  92.8 cm⁻¹ con altura real.
- **Ondulaciones en 82–98 cm⁻¹:** el ajuste genera máximos espurios en 82.6, 86.1, 89.0 y 97.6 cm⁻¹, cerca
  del corte entre detectores 1 y 2 y del ruido del detector 1. Aparecen en la tabla de picos. No son fonones.
- **Rango de los gráficos:** los límites de los ejes añaden un margen del 8 % del rango. Por eso la web
  muestra valores como R 1.07 o ε₂ 1634 que no son datos del ajuste (R del modelo ≤ 0.993).
