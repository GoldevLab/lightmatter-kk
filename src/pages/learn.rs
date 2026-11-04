use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Aprender — Lightmatter");
    set_page_description(
        "Cómo funciona el ajuste variacional de Kramers–Kronig: la medida, ε₁, ε₂, n, k, R, α, σ, T y la luminiscencia.",
    );
    view! {
        <main class="learn-page">
            <section class="hero">
                <p class="eyebrow">"Estudiar el laboratorio"</p>
                <h1>"Cómo funciona el análisis"</h1>
                <p class="hero-lead">
                    "Lightmatter no ajusta una curva cualquiera. Parte de que la respuesta del material es causal: la parte imaginaria de la función dieléctrica, ε₂, determina la parte real, ε₁. El programa busca el ε₂ cuyo par de Kramers–Kronig reproduce lo que mediste, y de ese par salen el resto de las constantes."
                </p>
                <div class="hero-actions">
                    <NavLink href="/lab" class="btn btn-primary">"Abrir el laboratorio"</NavLink>
                    <a href="#mapa" class="btn btn-ghost">"Ir al mapa"</a>
                </div>
            </section>

            <nav class="panel learn-toc" id="mapa" aria-label="Contenido">
                <h2>"Mapa"</h2>
                <ol>
                    <li><a href="#dato">"El dato"</a></li>
                    <li><a href="#eje">"Si el eje queda al revés"</a></li>
                    <li><a href="#medidas">"Qué puedes medir"</a></li>
                    <li><a href="#metodo">"El método"</a></li>
                    <li><a href="#ajuste">"El ajuste"</a></li>
                    <li><a href="#constantes">"Cada constante"</a></li>
                    <li><a href="#lectura">"Cómo leer el resultado"</a></li>
                    <li><a href="#luminiscencia">"Luminiscencia"</a></li>
                    <li><a href="#limites">"Qué no hace"</a></li>
                </ol>
            </nav>

            <section class="panel" id="dato">
                <h2>"El dato"</h2>
                <p>"Cada fila es un punto. La primera columna es la frecuencia ω, siempre positiva. La segunda es lo que mediste. La tercera, si está, es el error de ese punto: pesa el χ² y, si el ajuste puede, dibuja la barra ±1σ. Una línea vacía, o que empiece por #, *, ! o %, se ignora. Si subes un archivo .dat, .txt o .csv de hasta 4 MB, ese archivo manda sobre lo pegado."</p>
                <p>"Hasta 20000 filas entran en el χ². Por encima de eso el espectro se adelgaza conservando los picos y los valles."</p>
                <h3>"Unidades de ω"</h3>
                <ul>
                    <li>"cm⁻¹: el número de onda ya está en la unidad del cálculo."</li>
                    <li>"eV: el eje se queda en energía. El número de onda sale de E / 1.23984193×10⁻⁴."</li>
                    <li>"nm: primero pasa a eV con E = 1239.84193 / λ. En luminiscencia la intensidad se multiplica por λ²/hc, porque un intervalo de longitud de onda no es el mismo intervalo de energía. El error se transforma igual. Después del análisis el cuadro muestra esas energías, no los nanómetros originales."</li>
                    <li>"Otra: el eje no tiene unidad física. Salen ε, n, k, R y la pérdida. No salen α, σ₁, σ₂ ni la brecha de Tauc, porque esas fórmulas necesitan cm⁻¹."</li>
                </ul>
                <p>"Si el cociente entre la ω más alta y la más baja pasa de 20, el eje de las gráficas se dibuja en logaritmo. También puedes forzarlo con la casilla."</p>
                <h3 id="eje">"Si el eje queda al revés"</h3>
                <p class="learn-note">"Un archivo infrarrojo en cm⁻¹, por ejemplo de 80 a 2000, no es una longitud de onda. Si eliges nm, cada ω se reemplaza por 1239.84193 / ω y el espectro se da la vuelta: 1984 cm⁻¹ aparece como 0.625 y 80 cm⁻¹ como 15.5. El pico que estaba a la izquierda pasa a la derecha. ε∞ se lee entonces del extremo que ya no es el de alta frecuencia, y el ajuste puede quedarse en una recta cerca de ese R. Vuelve a cargar el archivo y elige cm⁻¹. No cambies la unidad sobre el cuadro ya reescrito."</p>
            </section>

            <section class="panel" id="medidas">
                <h2>"Qué puedes medir"</h2>
                <p>"Tú dices qué es la segunda columna. El ajuste no la reinterpreta: busca un ε₂ tal que, al calcular ε₁ por Kramers–Kronig y convertir el par a tu magnitud, coincida con la columna."</p>
                <ul>
                    <li>"ε₂. Es la magnitud del modelo. Con esta medida cada frecuencia puede ser una ancla, hasta 6000."</li>
                    <li>"ε₁. La parte real. ε∞ se ajusta junto con ε₂, salvo que lo escribas."</li>
                    <li>"n, índice de refracción. Igual: ε∞ se ajusta. El valor de partida es n² del último punto, entre 1 y 50."</li>
                    <li>"k, coeficiente de extinción. ε∞ no sale de k. Vacío usa 1, y entonces n se queda cerca de 1 donde la muestra es transparente."</li>
                    <li>"R, reflectividad a incidencia normal de un medio semi-infinito, entre 0 y 1. Si el máximo está entre 1.5 y 100.5, se toma como % y se divide entre 100. Si dejas ε∞ vacío, se estima del R del extremo alto suponiendo k pequeño: n = (1+√R)/(1−√R) y ε∞ = n², entre 1 y 50. Esa suposición falla si el extremo alto todavía absorbe."</li>
                    <li>"α, coeficiente de absorción en cm⁻¹. Necesita ω en cm⁻¹, eV o nm. ε∞ vacío usa 1."</li>
                    <li>"T, transmitancia de una lámina en aire. Necesita unidad de ω y el espesor en µm. Sin espesor el análisis no arranca."</li>
                    <li>"Luminiscencia. No pasa por Kramers–Kronig. Ver más abajo."</li>
                </ul>
                <p>"Con R, T, n, k o α el paso es denso: ε₁ en cada punto depende de todas las anclas. El modelo usa hasta 500 anclas, o 400 si hay más de 2500 puntos, y como mucho 6000 filas. Las anclas se ponen en los picos y los valles y el resto se reparte. Todos los puntos, aunque no sean ancla, entran en el χ²."</p>
            </section>

            <section class="panel" id="metodo">
                <h2>"El método"</h2>
                <p>"Un medio pasivo no responde antes de que llegue el campo. Esa causalidad ata la parte real y la parte imaginaria de cualquier función de respuesta. Para la función dieléctrica, en el convenio de este laboratorio,"</p>
                <p class="formula">"ε₁(ω) = ε∞ + (2/π) P ∫₀^∞ x ε₂(x) / (x² − ω²) dx"</p>
                <p>"P es el valor principal. No hace falta integrar numéricamente esa fórmula en cada paso. ε₂ se escribe como suma de triángulos de altura Aₖ centrados en anclas fijas. La integral de un triángulo tiene forma cerrada. Con"</p>
                <p class="formula">"g(x, y) = (x+y) ln|x+y| − (x−y) ln|x−y|"</p>
                <p>"la parte real que aporta el triángulo k, de altura uno, es"</p>
                <p class="formula">"ε₁,ₖ(ω) = (1/π) [ g(ω,ωₖ₋₁)/(ωₖ−ωₖ₋₁) − (ωₖ₊₁−ωₖ₋₁) g(ω,ωₖ) / ((ωₖ−ωₖ₋₁)(ωₖ₊₁−ωₖ)) + g(ω,ωₖ₊₁)/(ωₖ₊₁−ωₖ) ]"</p>
                <p>"El segundo término resta. Con un signo más la transformada no devuelve ε₁. El modelo completo es"</p>
                <p class="formula">"ε₂(ω) = Σ Aₖ · triánguloₖ(ω) + ε₂ de Drude, si lo pediste"</p>
                <p class="formula">"ε₁(ω) = ε∞ + Σ Aₖ · ε₁,ₖ(ω) + la parte real de Drude, sin volver a sumar ε∞"</p>
                <p>"A la izquierda de la primera ancla medida y a la derecha de la última hay un ancla extra con coeficiente cero. Entre el extremo medido y esa ancla, ε₂ baja en línea recta hasta cero. Esa cola entra en ε₁. Si ε₂ se obligara a cero justo en el primer y el último dato, faltaría peso espectral y ε₁ saldría mal. La casilla de dibujar fuera del rango muestra esa prolongación: no son datos nuevos."</p>
                <p>"Esta es la restricción variacional de Kuzmenko (2005). El triángulo es flexible: no supone que el material sea un oscilador de Lorentz, ni un Tauc–Lorentz, ni un Fano. Esos modelos siguen siendo útiles para interpretar, pero aquí no se imponen."</p>
                <h3>"ε∞"</h3>
                <p>"ε∞ es ε₁ por encima de tu ventana, la contribución de todo lo que no mediste a más alta frecuencia. No es un segundo ajuste de la curva."</p>
                <ul>
                    <li>"Si lo escribes, se usa ese número."</li>
                    <li>"Con n o con ε₁, si lo dejas vacío, se ajusta junto con los Aₖ."</li>
                    <li>"Con R, si lo dejas vacío, se estima del extremo alto y no se ajusta."</li>
                    <li>"Con k, α o T, la medida no lo fija. Vacío usa 1. Entonces, donde la muestra es transparente, n sale cerca de 1."</li>
                </ul>
                <p>"α y k fijan la extinción, porque α = 4πν̃k. No fijan el índice. ε₁, n, R, la altura de ε₂, la pérdida y σ₂ se mueven con ε∞. Si en la zona transparente conoces n, escribe ε∞ = n² y vuelve a analizar. En el ZnO visible, n ≈ 2 y ε∞ ≈ 4; con 1 esos paneles no son los del cristal, aunque α quede clavado a los datos."</p>
                <h3>"Fondo Drude"</h3>
                <p>"Si lo marcas, antes de los triángulos se resta un fondo de electrones libres:"</p>
                <p class="formula">"ε₂,Drude = ωₚ² γ ω / (ω⁴ + γ² ω²)"</p>
                <p class="formula">"ε₁,Drude = ε∞ − ωₚ² ω² / (ω⁴ + γ² ω²)"</p>
                <p>"ωₚ y γ en auto se estiman del espectro. Puedes escribirlos. El Drude no sustituye a los triángulos: les quita el fondo metálico para que las anclas describan el resto."</p>
            </section>

            <section class="panel" id="ajuste">
                <h2>"El ajuste"</h2>
                <p>"Los Aₖ, y a veces ε∞, se mueven con Levenberg–Marquardt. En cada paso se compara la magnitud que pediste, no ε₂, salvo que hayas medido ε₂."</p>
                <p class="formula">"χ² = Σ ((dato − modelo) / error)²"</p>
                <p>"Sin tercera columna el error vale 1 y todos los puntos pesan igual. Un error grande deja que ese punto se aparte. El número que ves en el resultado es solo este χ² de los datos. La suavidad y el castigo de ε₂ negativa sí empujan el ajuste, pero no se suman a ese número."</p>
                <p>"Si hay un coeficiente por punto, χ²/dof no aplica: un χ² casi cero quiere decir que la curva pasa por tus datos, no que el material haya quedado identificado. Con menos anclas que puntos, χ²/dof parte el χ² por los puntos que sobran."</p>
                <p>"La suavidad, 0.08 por defecto, frena la segunda diferencia entre anclas interiores: Aₖ₋₁ − 2 Aₖ + Aₖ₊₁. No empuja los extremos hacia cero. 0 deja solo los datos."</p>
                <p>"Si ε₂ baja de 0, el valor 100 suma un castigo proporcional a ε₂². En un medio pasivo ε₂ es mayor o igual que cero. 0 permite ε₂ negativa."</p>
                <p>"El paso mezcla el descenso por gradiente y el salto al mínimo de la curvatura. Si el paso mejora, λ se divide entre 10. Si no, se multiplica por 10. Para cuando el objetivo ya no baja un 1 %, o al llegar al máximo de pasos. 20 es el valor del formulario; 80 es el techo."</p>
                <p>"La barra ±1σ sale de la inversa de la curvatura de los datos solos, con δχ² = 1. Aparece cuando pegaste el error de cada punto. La suavidad no entra en esa barra."</p>
            </section>

            <section class="panel" id="constantes">
                <h2>"Cada constante"</h2>
                <p>"Cuando el par (ε₁, ε₂) ya está, el resto es álgebra. Nada de esto es un segundo ajuste."</p>
                <h3>"ε₂ y ε₁"</h3>
                <p>"ε₂ es lo que ajustan los triángulos. ε₁ es ε∞ más la Kramers–Kronig de ese ε₂, incluida la cola que baja a cero fuera de la ventana. Cumplen, con n y k,"</p>
                <p class="formula">"ε₁ = n² − k² ,   ε₂ = 2 n k"</p>
                <p>"k lleva el signo de ε₂. En un medio pasivo los dos son positivos."</p>
                <h3>"n y k"</h3>
                <p class="formula">"n = √( (|ε| + ε₁) / 2 ) ,   k = √( (|ε| − ε₁) / 2 )"</p>
                <p>"|ε| = √(ε₁² + ε₂²). n es el índice: cuánto se frena la fase. k es la extinción: cuánto se apaga la onda. k sale de α sin necesidad de ε∞. n no."</p>
                <h3>"R"</h3>
                <p>"Reflectividad de un medio que llena un semiespacio, luz que llega en perpendicular desde el vacío:"</p>
                <p class="formula">"R = ((n−1)² + k²) / ((n+1)² + k²)"</p>
                <p>"No es la reflectividad de una película sobre un sustrato, ni la de una lámina con las dos caras. Para eso haría falta el espesor y el índice de lo que hay detrás."</p>
                <h3>"Pérdida"</h3>
                <p class="formula">"−Im(1/ε) = ε₂ / (ε₁² + ε₂²)"</p>
                <p>"Sus picos marcan modos longitudinales y el plasmón. Con ε∞ demasiado bajo, ε es pequeño, 1/ε es grande y la pérdida sale inflada."</p>
                <h3>"α"</h3>
                <p class="formula">"α = 4 π ν̃ k"</p>
                <p>"en cm⁻¹, con ν̃ el número de onda en cm⁻¹. Es la misma magnitud que pegaste si mediste absorción. El panel de datos compara tu columna con el modelo; el otro panel es esta fórmula aplicada al k del par (ε₁, ε₂)."</p>
                <h3>"σ₁ y σ₂"</h3>
                <p>"Conductividad óptica en Ω⁻¹ cm⁻¹, pareja de la función dieléctrica:"</p>
                <p class="formula">"σ₁ = ν̃ ε₂ / 60 ,   σ₂ = ν̃ (ε∞ − ε₁) / 60"</p>
                <p>"σ₁ sigue a ε₂. σ₂ sigue a cuánto se aparta ε₁ de ε∞: es positiva cuando ε₁ está por debajo de ε∞ y negativa cuando está por encima, del lado de alta frecuencia de una resonancia. Si ε∞ es 1 y el material tiene n cerca de 2, σ₂ no es la conductividad del material."</p>
                <h3>"T y la absorbancia"</h3>
                <p>"Salen solo si escribes el espesor, en µm, y ω tiene unidad. d es ese espesor en cm. x = exp(−α d) = exp(−4π ν̃ k d). Para una lámina en aire, sumando las reflexiones múltiples incoherentes de las dos caras,"</p>
                <p class="formula">"T = (1−R)² x / (1 − R² x²)"</p>
                <p class="formula">"A = −log₁₀(T)"</p>
                <p>"A es la absorbancia decádica, la densidad óptica. No es α. Sin espesor no se inventan. Si la medida ya era T, el ajuste usa esta misma fórmula y el espesor es obligatorio."</p>
                <h3>"Brecha de Tauc"</h3>
                <p>"Solo con ω en cm⁻¹, eV o nm, y si la energía arranca en al menos 0.2 eV y la absorción empieza cerca de cero. Se mira (α E)², borde directo, y (α E)^½, borde indirecto. El corte es la tangente más empinada de esa curva antes del máximo, entre el 10 % y el 90 % de la altura, prolongada hasta el eje. Se muestra la fórmula que es una recta, con R² de la subida entre el 15 % y el 85 %. La otra, si no es recta, no es una segunda brecha. Si α sigue en su máximo al final de la ventana, el corte queda por encima del inicio visible del borde: la ventana cortó la subida."</p>
                <h3>"Picos de ε₂"</h3>
                <p>"Máximos locales cuya prominencia —la altura sobre el valle más alto a los lados— pasa del 5 % del rango de ε₂. Se listan hasta 12, con posición, ancho a media altura, área y el porcentaje de esa área sobre la suma de los picos listados. Si la mitad no se alcanza dentro de la ventana, el ancho se marca abierto. Un máximo que sigue subiendo al final del rango es el borde cortado por la ventana y no entra como pico cerrado."</p>
            </section>

            <section class="panel" id="lectura">
                <h2>"Cómo leer el resultado"</h2>
                <ol>
                    <li>"Mira el χ² y la diferencia, que es (ajuste − datos) × 100. Si el ajuste pasa por los datos, la magnitud que mediste quedó reproducida. Eso no alcanza para fijar lo que la medida no contiene."</li>
                    <li>"Si la curva del ajuste es casi una recta y los datos tienen picos, revisa la unidad antes de tocar la suavidad. Un cm⁻¹ leído como nm invierte el eje y el modelo se queda en el R de un ε∞ mal tomado."</li>
                    <li>"Lee el aviso de ε∞. Si dice 1 por defecto y mediste α, k o T, fíate de α y de k. ε₁, n, R, la pérdida, la altura de ε₂ y σ₂ esperan a que escribas ε∞."</li>
                    <li>"El panel de datos es tu columna. Los demás paneles son el par (ε₁, ε₂) y lo que se deduce de él."</li>
                    <li>"Pasa el mouse por una gráfica para el punto más cercano. Cada gráfica se baja en SVG o PNG."</li>
                    <li>"Descargar: .dat y CSV traen ω, ε₁, ε₂, n, k, R, la pérdida y, con unidad, σ₁, σ₂ y α. Con espesor añaden T y A. Datos y ajuste compara tu columna con el modelo. Picos es la tabla. JSON es el informe entero."</li>
                </ol>
                <p>"La casilla de fuera del rango dibuja ε₁ y ε₂ más allá de tus datos, con la cola lineal de ε₂. Sirve para ver la extrapolación. No añade información experimental."</p>
            </section>

            <section class="panel" id="luminiscencia">
                <h2>"Luminiscencia"</h2>
                <p>"Fotoluminiscencia, electroluminiscencia o catodoluminiscencia es emisión. No hay una Kramers–Kronig que convierta esa intensidad en ε. El laboratorio no finge una."</p>
                <ul>
                    <li>"Picos: la misma regla del 5 % de prominencia, hasta 12, con ancho a media altura y área."</li>
                    <li>"Centroide Σ x I / Σ I del espectro entero, e integral por trapecios."</li>
                    <li>"Temperatura de la cola alta, solo con ω en energía. En el tramo que baja del 50 % al 5 % del máximo se ajusta ln(I/E²). Si esa recta tiene R² de al menos 0.97 y la temperatura sale por debajo de 5000 K, se muestra. Una gaussiana o una segunda banda no pasan el corte."</li>
                    <li>"Forma de la absorción por van Roosbroeck–Shockley: α proporcional a I · e^(E/kT) / E², solo donde la intensidad pasa del 2 % del máximo, y normalizada a 1. La temperatura es la del campo T de la muestra, 300 K si la dejas vacía. Es una forma relativa, no el α en cm⁻¹."</li>
                </ul>
            </section>

            <section class="panel" id="limites">
                <h2>"Qué no hace"</h2>
                <ul>
                    <li>"No modela una película sobre un sustrato, ni elipsometría en ángulo, ni luz polarizada."</li>
                    <li>"No recupera lo que está fuera de tu ventana. La cola lineal y ε∞ son una hipótesis, no una medida."</li>
                    <li>"No distingue solo con α el índice de un material transparente. Hace falta ε∞, o medir n, ε₁ o R."</li>
                    <li>"No trata la luminiscencia como una constante óptica."</li>
                    <li>"El cálculo corre en este aparato, también instalada y sin red. No hay cuenta."</li>
                </ul>
                <p><NavLink href="/lab" class="btn btn-primary">"Probarlo en el laboratorio"</NavLink></p>
            </section>
        </main>
    }
}
