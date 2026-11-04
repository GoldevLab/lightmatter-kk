use lightmatter::i18n::tx;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    let lang = crate::locale::current();
    crate::stats::record_visit("learn");
    set_page_title(crate::seo::learn_title(lang));
    set_page_description(crate::seo::learn_description(lang));
    set_page_canonical(crate::seo::canonical("/learn"));
    set_page_json_ld(crate::seo::webpage_json_ld(
        crate::seo::learn_title(lang),
        crate::seo::learn_description(lang),
        "/learn",
        lang,
    ));
    view! {
        <main class="learn-page">
            <div class="learn-docs">
                {guide_nav(lang)}
                <article class="learn-doc">
            <section class="hero">
                <p class="eyebrow">{tx(lang, "Estudiar el laboratorio", "Study the laboratory", "了解实验室", "Estudar o laboratório")}</p>
                <h1>{tx(lang, "Cómo funciona el análisis", "How the analysis works", "分析如何进行", "Como a análise funciona")}</h1>
                <p class="hero-lead">{tx(lang, "Lightmatter no ajusta una curva cualquiera. Parte de que la respuesta del material es causal: la parte imaginaria de la función dieléctrica, ε₂, determina la parte real, ε₁. El programa busca el ε₂ cuyo par de Kramers–Kronig reproduce lo que mediste, y de ese par salen el resto de las constantes.", "Lightmatter does not fit an arbitrary curve. It starts from causality: the imaginary part of the dielectric function, ε₂, determines the real part, ε₁. The program looks for the ε₂ whose Kramers–Kronig pair reproduces what you measured, and the other constants come from that pair.", "Lightmatter 不是随便拟合一条曲线。它从因果性出发：介电函数的虚部 ε₂ 决定实部 ε₁。程序寻找其克拉默斯–克罗尼格配对能再现测量的 ε₂，其余常数由这对量得到。", "O Lightmatter não ajusta uma curva qualquer. Parte de que a resposta do material é causal: a parte imaginária da função dielétrica, ε₂, determina a parte real, ε₁. O programa procura o ε₂ cujo par de Kramers–Kronig reproduz o que você mediu, e desse par saem as outras constantes.")}</p>
                {figure(
                    "/images/learn-kk.jpg",
                    tx(lang, "Pico de absorción arriba y curva dispersiva abajo, unidas por la misma frecuencia.", "An absorption peak above and a dispersive curve below, tied to the same frequency.", "上方是吸收峰，下方是色散曲线，由同一频率连在一起。", "Pico de absorção em cima e curva dispersiva embaixo, ligadas pela mesma frequência."),
                    tx(lang, "Arriba está ε₂, lo que el material absorbe. Abajo está ε₁, que baja antes del pico y sube después. No son dos ajustes: una fija a la otra. El destello llega primero y la respuesta viene después.", "Above is ε₂, what the material absorbs. Below is ε₁, which falls before the peak and rises after it. They are not two fits: one fixes the other. The flash arrives first and the response comes after.", "上面是 ε₂，材料吸收的部分。下面是 ε₁，在峰之前下降、在峰之后上升。这不是两次拟合：一个决定另一个。闪光先到，响应后到。", "Em cima está ε₂, o que o material absorve. Embaixo está ε₁, que desce antes do pico e sobe depois. Não são dois ajustes: um fixa o outro. O clarão chega primeiro e a resposta vem depois."),
                    true,
                )}
                <div class="hero-actions">
                    <NavLink href="/lab" class="btn btn-primary">{tx(lang, "Abrir el laboratorio", "Open the laboratory", "打开实验室", "Abrir o laboratório")}</NavLink>
                </div>
            </section>
            <details class="learn-jump">
                <summary>{tx(lang, "En esta guía", "In this guide", "本指南", "Neste guia")}</summary>
                {guide_links(lang)}
            </details>

            <section id="dato">
                <h2>{tx(lang, "El dato", "The data", "数据", "O dado")}</h2>
                {figure(
                    "/images/learn-data.jpg",
                    tx(lang, "Filas de un archivo que se convierten en una curva con barras de error.", "Rows of a file turning into a curve with error bars.", "文件中的行变成带误差棒的曲线。", "Linhas de um arquivo que viram uma curva com barras de erro."),
                    tx(lang, "Cada fila es un punto: la frecuencia y lo que mediste. Esos puntos dibujan el espectro. La barrita es la incertidumbre de ese punto.", "Each row is a point: the frequency and what you measured. Those points draw the spectrum. The little bar is that point’s uncertainty.", "每一行是一个点：频率和你测到的值。这些点画出光谱。小竖线是该点的不确定度。", "Cada linha é um ponto: a frequência e o que você mediu. Esses pontos desenham o espectro. A barrinha é a incerteza desse ponto."),
                    false,
                )}
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
                {axis_warning(lang)}
            </section>

            <section id="medidas">
                <h2>{tx(lang, "Qué puedes medir", "What you can measure", "可以测量什么", "O que você pode medir")}</h2>
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

            <section id="metodo">
                <h2>{tx(lang, "El método", "The method", "方法", "O método")}</h2>
                {figure(
                    "/images/learn-triangles.jpg",
                    tx(lang, "Triángulos de colores que, sumados, forman un espectro, con una segunda curva al lado.", "Colored triangles that add up to a spectrum, with a second curve beside it.", "彩色三角相加形成一条光谱，旁边还有第二条曲线。", "Triângulos coloridos que, somados, formam um espectro, com uma segunda curva ao lado."),
                    tx(lang, "ε₂ se construye sumando triángulos. La montaña de arriba es esa suma. La onda dorada es la ε₁ que les corresponde, no un segundo dibujo libre.", "ε₂ is built by adding triangles. The mountain above is that sum. The gold wave is the ε₁ that belongs to them, not a second freehand drawing.", "ε₂ 由三角相加而成。上面的山是这个和。金色的波是与之对应的 ε₁，不是另画的一条。", "ε₂ é construído somando triângulos. A montanha de cima é essa soma. A onda dourada é o ε₁ que lhes corresponde, não um segundo desenho livre."),
                    false,
                )}
                <p>"Un medio pasivo no responde antes de que llegue el campo. Esa causalidad ata dos partes de la misma respuesta: ε₂, lo que el material absorbe, y ε₁, lo que cambia la fase de la onda. Conocida una en todas las frecuencias, la otra queda fijada. No son dos curvas libres."</p>
                <p>"En la práctica nunca se mide todo el eje. Lo que queda por encima de tu ventana se junta en un número, ε∞. Dentro de la ventana, el laboratorio usa"</p>
                <p class="formula">"ε₁(ω) = ε∞ + (2/π) P ∫₀^∞ x ε₂(x) / (x² − ω²) dx"</p>
                {open_proof(
                    "proof-kk",
                    tx(lang, "Ver por qué van juntas", "See why they are tied", "查看为什么绑在一起", "Ver por que ficam juntas"),
                    tx(lang, "De la causalidad a Kramers–Kronig", "From causality to Kramers–Kronig", "从因果性到克拉默斯–克罗尼格", "Da causalidade a Kramers–Kronig"),
                    proof_kk(lang),
                )}
                <p>"Esa integral no se calcula numéricamente en cada paso. ε₂ se construye sumando triángulos de altura Aₖ, uno por ancla. Cada triángulo vale 1 en su ancla y 0 en las vecinas. Su Kramers–Kronig tiene forma cerrada, así que ε₁ es la misma suma, con otros pesos."</p>
                <p class="formula">"ε₂(ω) = Σ Aₖ · triánguloₖ(ω) + ε₂ de Drude, si lo pediste"</p>
                <p class="formula">"ε₁(ω) = ε∞ + Σ Aₖ · ε₁,ₖ(ω) + la parte real de Drude, sin volver a sumar ε∞"</p>
                {open_proof(
                    "proof-g",
                    tx(lang, "Ver el triángulo y g(x, y)", "See the triangle and g(x, y)", "查看三角和 g(x, y)", "Ver o triângulo e g(x, y)"),
                    tx(lang, "La integral cerrada de un triángulo", "The closed integral of a triangle", "一个三角的封闭积分", "A integral fechada de um triângulo"),
                    proof_g(lang),
                )}
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
                {open_proof(
                    "proof-drude",
                    tx(lang, "Ver el electrón libre", "See the free electron", "查看自由电子", "Ver o elétron livre"),
                    tx(lang, "De Lorentz a Drude", "From Lorentz to Drude", "从洛伦兹到 Drude", "De Lorentz a Drude"),
                    proof_drude(lang),
                )}
            </section>

            <section id="ajuste">
                <h2>{tx(lang, "El ajuste", "The fit", "拟合", "O ajuste")}</h2>
                {figure(
                    "/images/learn-fit.jpg",
                    tx(lang, "Puntos de una medida y una curva suave que pasa por ellos.", "Measured points and a smooth curve passing through them.", "测量点，以及一条穿过它们的平滑曲线。", "Pontos de uma medida e uma curva suave que passa por eles."),
                    tx(lang, "Los puntos son tu medida. La curva es el modelo. El programa mueve la altura de los triángulos hasta que la curva pase por los puntos.", "The points are your measurement. The curve is the model. The program moves the height of the triangles until the curve passes through the points.", "点是你的测量。曲线是模型。程序调整三角的高度，直到曲线穿过这些点。", "Os pontos são a sua medida. A curva é o modelo. O programa mexe na altura dos triângulos até a curva passar pelos pontos."),
                    false,
                )}
                <p>"Los Aₖ, y a veces ε∞, se mueven con Levenberg–Marquardt. En cada paso se compara la magnitud que pediste, no ε₂, salvo que hayas medido ε₂. El número que se baja es"</p>
                <p class="formula">"χ² = Σ ((dato − modelo) / error)²"</p>
                {open_proof(
                    "proof-lm",
                    tx(lang, "Ver cómo da el paso", "See how a step is taken", "查看步子怎么走", "Ver como o passo é dado"),
                    tx(lang, "El paso de Levenberg–Marquardt", "The Levenberg–Marquardt step", "莱文贝格–马夸特的一步", "O passo de Levenberg–Marquardt"),
                    proof_lm(lang),
                )}
                <p>"Sin tercera columna el error vale 1 y todos los puntos pesan igual. Un error grande deja que ese punto se aparte. El número que ves en el resultado es solo este χ² de los datos. La suavidad y el castigo de ε₂ negativa sí empujan el ajuste, pero no se suman a ese número."</p>
                <p>"Si hay un coeficiente por punto, χ²/dof no aplica: un χ² casi cero quiere decir que la curva pasa por tus datos, no que el material haya quedado identificado. Con menos anclas que puntos, χ²/dof parte el χ² por los puntos que sobran."</p>
                <p>"La suavidad, 0.08 por defecto, frena la segunda diferencia entre anclas interiores: Aₖ₋₁ − 2 Aₖ + Aₖ₊₁. No empuja los extremos hacia cero. 0 deja solo los datos."</p>
                <p>"Si ε₂ baja de 0, el valor 100 suma un castigo proporcional a ε₂². En un medio pasivo ε₂ es mayor o igual que cero. 0 permite ε₂ negativa."</p>
                <p>"El paso mezcla el descenso por gradiente y el salto al mínimo de la curvatura. Si el paso mejora, λ se divide entre 10. Si no, se multiplica por 10. Para cuando el objetivo ya no baja un 1 % y ningún punto queda a más del 2 % del rango, o al llegar al máximo de pasos. 20 es el valor del formulario; 240 es el techo."</p>
                <p>"Una raya de R de pocos cm⁻¹, un reststrahlen, no se cierra subiendo los pasos. La punta sube y se estanca cerca de 0.6 cuando el dato está en 0.97; el valle de al lado se rellena y el χ² puede seguir bajando. La banda ancha, ya ajustada, deja ε₁ positivo en esa frecuencia por Kramers–Kronig. Si la raya es más estrecha que el 8 % de su frecuencia y destaca, el programa prueba un oscilador de Lorentz como fondo. En una raya así de fuerte ese fondo no alcanza para clavarla al dato. Suavidad en 0 y el castigo de ε₂ negativa en 100. El tramo alto, ya lejos de la raya, sí puede quedar sobre los datos."</p>
                <p>"La barra ±1σ sale de la inversa de la curvatura de los datos solos, con δχ² = 1. Aparece cuando pegaste el error de cada punto. La suavidad no entra en esa barra."</p>
            </section>

            <section id="constantes">
                <h2>{tx(lang, "Cada constante", "Each constant", "每个常数", "Cada constante")}</h2>
                {figure(
                    "/images/learn-constants.jpg",
                    tx(lang, "Un rayo llega a un material: una parte rebota, otra entra y se apaga.", "A ray reaches a material: part of it bounces, part enters and fades.", "一束光到达材料：一部分弹回，一部分进入后渐渐熄灭。", "Um raio chega a um material: uma parte rebate, outra entra e se apaga."),
                    tx(lang, "El rayo que rebota es la reflectividad R. El que entra y cambia de ritmo es el índice n. El que se apaga dentro es la extinción k, y de ahí sale la absorción α.", "The ray that bounces is the reflectivity R. The one that enters and changes pace is the index n. The one that fades inside is the extinction k, and the absorption α comes from that.", "弹回的是反射率 R。进入并改变节奏的是折射率 n。在内部熄灭的是消光 k，吸收 α 由此得到。", "O raio que rebate é a refletividade R. O que entra e muda de ritmo é o índice n. O que se apaga por dentro é a extinção k, e daí sai a absorção α."),
                    false,
                )}
                <p>"Cuando el par (ε₁, ε₂) ya está, el resto es álgebra. Nada de esto es un segundo ajuste."</p>
                <h3>"ε₂ y ε₁"</h3>
                <p>"ε₂ es lo que ajustan los triángulos. ε₁ es ε∞ más la Kramers–Kronig de ese ε₂, incluida la cola que baja a cero fuera de la ventana. Cumplen, con n y k,"</p>
                <p class="formula">"ε₁ = n² − k² ,   ε₂ = 2 n k"</p>
                <p>"k lleva el signo de ε₂. En un medio pasivo los dos son positivos."</p>
                <h3>"n y k"</h3>
                <p class="formula">"n = √( (|ε| + ε₁) / 2 ) ,   k = √( (|ε| − ε₁) / 2 )"</p>
                {open_proof(
                    "proof-nk",
                    tx(lang, "Ver de dónde salen", "See where they come from", "查看来源", "Ver de onde saem"),
                    tx(lang, "De ε a n, k y R", "From ε to n, k, and R", "从 ε 到 n、k 和 R", "De ε a n, k e R"),
                    proof_nk(lang),
                )}
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

            <section id="lectura">
                <h2>{tx(lang, "Cómo leer el resultado", "How to read the result", "如何读结果", "Como ler o resultado")}</h2>
                <ol>
                    <li>"Mira el χ² y la diferencia, que es (ajuste − datos) × 100. Si el ajuste pasa por los datos, la magnitud que mediste quedó reproducida. Eso no alcanza para fijar lo que la medida no contiene."</li>
                    <li>"Si la curva del ajuste es casi una recta y los datos tienen picos, revisa la unidad antes de tocar la suavidad. Un cm⁻¹ leído como nm invierte el eje y el modelo se queda en el R de un ε∞ mal tomado."</li>
                    <li>"Lee el aviso de ε∞. Si dice 1 por defecto y mediste α, k o T, fíate de α y de k. ε₁, n, R, la pérdida, la altura de ε₂ y σ₂ esperan a que escribas ε∞."</li>
                    <li>"El panel de datos es tu columna. Los demás paneles son el par (ε₁, ε₂) y lo que se deduce de él."</li>
                    <li>"Pasa el mouse por una gráfica para el punto más cercano. Cada gráfica se baja en SVG o PNG."</li>
                    <li>"Descargar: .dat y CSV traen ω, ε₁, ε₂, n, k, R, la pérdida y, con unidad, σ₁, σ₂ y α. Con espesor añaden T y A. Datos y ajuste compara tu columna con el modelo. Picos es la tabla. JSON es el informe para procesarlo. Al final, «Descargar todos los resultados» guarda un HTML con el resumen resaltado, los picos y el espectro en tablas."</li>
                </ol>
                <p>"La casilla de fuera del rango dibuja ε₁ y ε₂ más allá de tus datos, con la cola lineal de ε₂. Sirve para ver la extrapolación. No añade información experimental."</p>
            </section>

            <section id="luminiscencia">
                <h2>{tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência")}</h2>
                <p>"Fotoluminiscencia, electroluminiscencia o catodoluminiscencia es emisión. No hay una Kramers–Kronig que convierta esa intensidad en ε. El laboratorio no finge una."</p>
                <ul>
                    <li>"Picos: la misma regla del 5 % de prominencia, hasta 12, con ancho a media altura y área."</li>
                    <li>"Centroide Σ x I / Σ I del espectro entero, e integral por trapecios."</li>
                    <li>"Temperatura de la cola alta, solo con ω en energía. En el tramo que baja del 50 % al 5 % del máximo se ajusta ln(I/E²). Si esa recta tiene R² de al menos 0.97 y la temperatura sale por debajo de 5000 K, se muestra. Una gaussiana o una segunda banda no pasan el corte."</li>
                    <li>"Forma de la absorción por van Roosbroeck–Shockley: α proporcional a I · e^(E/kT) / E², solo donde la intensidad pasa del 2 % del máximo, y normalizada a 1. La temperatura es la del campo T de la muestra, 300 K si la dejas vacía. Es una forma relativa, no el α en cm⁻¹."</li>
                </ul>
            </section>

            <section id="limites">
                <h2>{tx(lang, "Qué no hace", "What it does not do", "它不做什么", "O que ele não faz")}</h2>
                <ul>
                    <li>"No modela una película sobre un sustrato, ni elipsometría en ángulo, ni luz polarizada."</li>
                    <li>"No recupera lo que está fuera de tu ventana. La cola lineal y ε∞ son una hipótesis, no una medida."</li>
                    <li>"No distingue solo con α el índice de un material transparente. Hace falta ε∞, o medir n, ε₁ o R."</li>
                    <li>"No trata la luminiscencia como una constante óptica."</li>
                    <li>"El cálculo corre en este aparato, también instalada y sin red. No hay cuenta."</li>
                </ul>
                <p><NavLink href="/lab" class="btn btn-primary">{tx(lang, "Probarlo en el laboratorio", "Try it in the laboratory", "在实验室里试", "Experimentar no laboratório")}</NavLink></p>
            </section>
                </article>
            </div>
        </main>
    }
}

fn guide_nav(lang: lightmatter::i18n::Lang) -> View {
    let label = tx(lang, "Guía", "Guide", "指南", "Guia");
    let open = tx(lang, "Abrir el laboratorio", "Open the laboratory", "打开实验室", "Abrir o laboratório");
    let links = guide_links(lang);
    view! {
        <aside class="learn-side" aria-label={label}>
            <p class="learn-side-kicker">{label}</p>
            {links}
            <NavLink href="/lab" class="btn btn-ghost btn-small">{open}</NavLink>
        </aside>
    }
}

fn guide_links(lang: lightmatter::i18n::Lang) -> View {
    view! {
        <nav>
            <p>{tx(lang, "Empezar", "Start", "开始", "Começar")}</p>
            <a href="#dato">{tx(lang, "El dato", "The data", "数据", "O dado")}</a>
            <a href="#medidas">{tx(lang, "Qué puedes medir", "What you can measure", "可以测量什么", "O que você pode medir")}</a>
            <p>{tx(lang, "El análisis", "The analysis", "分析", "A análise")}</p>
            <a href="#metodo">{tx(lang, "El método", "The method", "方法", "O método")}</a>
            <a href="#ajuste">{tx(lang, "El ajuste", "The fit", "拟合", "O ajuste")}</a>
            <a href="#constantes">{tx(lang, "Cada constante", "Each constant", "每个常数", "Cada constante")}</a>
            <p>{tx(lang, "El resultado", "The result", "结果", "O resultado")}</p>
            <a href="#lectura">{tx(lang, "Cómo leerlo", "How to read it", "如何阅读", "Como ler")}</a>
            <a href="#luminiscencia">{tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência")}</a>
            <a href="#limites">{tx(lang, "Qué no hace", "What it does not do", "它不做什么", "O que ele não faz")}</a>
        </nav>
    }
}

fn axis_warning(lang: lightmatter::i18n::Lang) -> View {
    let title = tx(lang, "Si el eje queda al revés", "If the axis comes out reversed", "如果坐标轴反过来了", "Se o eixo sair invertido");
    let lead = tx(
        lang,
        "Un archivo en cm⁻¹, de 80 a 2000 por ejemplo, no es una longitud de onda. Si eliges nm, cada número se cambia por 1239.84 / ω. El número chico de cm⁻¹ se vuelve el número grande de eV, y el espectro se da la vuelta.",
        "A file in cm⁻¹, from 80 to 2000 for example, is not a wavelength. If you choose nm, each number is replaced by 1239.84 / ω. The small cm⁻¹ number becomes the large eV number, and the spectrum flips.",
        "以 cm⁻¹ 记录的文件，例如从 80 到 2000，不是波长。如果选择 nm，每个数都会换成 1239.84 / ω。小的波数变成大的 eV，光谱就此左右对调。",
        "Um arquivo em cm⁻¹, de 80 a 2000 por exemplo, não é um comprimento de onda. Se você escolhe nm, cada número vira 1239.84 / ω. O número pequeno em cm⁻¹ vira o número grande em eV, e o espectro se inverte.",
    );
    let file = tx(lang, "En el archivo", "In the file", "文件里", "No arquivo");
    let read = tx(lang, "Al leerlo como nm", "Read as nm", "按 nm 读取", "Lido como nm");
    let left = tx(lang, "80 cm⁻¹", "80 cm⁻¹", "80 cm⁻¹", "80 cm⁻¹");
    let right = tx(lang, "1984 cm⁻¹", "1984 cm⁻¹", "1984 cm⁻¹", "1984 cm⁻¹");
    let left_out = tx(lang, "15.5 eV, a la derecha", "15.5 eV, on the right", "15.5 eV，到了右边", "15.5 eV, à direita");
    let right_out = tx(lang, "0.625 eV, a la izquierda", "0.625 eV, on the left", "0.625 eV，到了左边", "0.625 eV, à esquerda");
    let after = tx(
        lang,
        "Un pico que estaba a la izquierda pasa a la derecha. ε∞ se estima del extremo que el programa toma por alta frecuencia, así que lee el R del lado equivocado y el ajuste puede quedarse en una recta cerca de ese R.",
        "A peak that was on the left moves to the right. ε∞ is estimated from the end the program takes as high frequency, so it reads R from the wrong side and the fit can settle on a straight line near that R.",
        "原来在左边的峰会跑到右边。ε∞ 取自程序当成高频端的那一头，于是读到的是错误一侧的 R，拟合可能停在靠近那个 R 的一条直线上。",
        "Um pico que estava à esquerda passa para a direita. ε∞ é estimado da ponta que o programa toma por alta frequência, então ele lê o R do lado errado e o ajuste pode ficar numa reta perto desse R.",
    );
    let fix = tx(
        lang,
        "Vuelve a cargar el archivo y elige cm⁻¹. Si el cuadro ya muestra eV, déjalo: cambiar la unidad convierte esos números otra vez.",
        "Load the file again and choose cm⁻¹. If the box already shows eV, leave it: changing the unit converts those numbers again.",
        "重新载入文件并选择 cm⁻¹。如果框里已经是 eV，就保持这样：再改单位会把这些数又转换一次。",
        "Carregue o arquivo de novo e escolha cm⁻¹. Se a caixa já mostra eV, deixe assim: mudar a unidade converte esses números outra vez.",
    );
    view! {
        <aside class="learn-warn" role="note">
            <p class="learn-warn-title">{title}</p>
            <p>{lead}</p>
            <table class="learn-flip">
                <thead>
                    <tr><th>{file}</th><th>{read}</th></tr>
                </thead>
                <tbody>
                    <tr><td>{left}</td><td>{left_out}</td></tr>
                    <tr><td>{right}</td><td>{right_out}</td></tr>
                </tbody>
            </table>
            <p>{after}</p>
            <p><strong>{fix}</strong></p>
        </aside>
    }
}

fn figure(src: &'static str, alt: &'static str, caption: &'static str, eager: bool) -> View {
    let loading = if eager { "eager" } else { "lazy" };
    view! {
        <figure class="learn-figure">
            <img src={src} alt={alt} width="1280" height="720" loading={loading} decoding="async" />
            <figcaption>{caption}</figcaption>
        </figure>
    }
}

fn open_proof(id: &'static str, label: &'static str, title: &'static str, body: View) -> View {
    view! {
        <div class="proof-row">
            <Modal id={id} class="proof-sheet">
                <button slot="trigger" type="button" class="proof-open">{label}</button>
                <div class="proof-body">
                    <p class="eyebrow">{title}</p>
                    {body}
                </div>
            </Modal>
        </div>
    }
}

fn proof_kk(lang: lightmatter::i18n::Lang) -> View {
    let a = tx(
        lang,
        "La respuesta lineal es una convolución: lo que sale en el tiempo es el campo pasado, pesado por una función G. En frecuencia esa convolución es un producto, X(ω) = G(ω) f(ω). G es un número complejo.",
        "A linear response is a convolution: what comes out in time is the past field, weighted by a function G. In frequency that convolution is a product, X(ω) = G(ω) f(ω). G is a complex number.",
        "线性响应是一次卷积：时间上的输出是过去的场，用函数 G 加权。在频率上，卷积变成乘积 X(ω) = G(ω) f(ω)。G 是一个复数。",
        "A resposta linear é uma convolução: o que sai no tempo é o campo passado, pesado por uma função G. Em frequência essa convolução é um produto, X(ω) = G(ω) f(ω). G é um número complexo.",
    );
    let b = tx(
        lang,
        "Si el material es causal, G(t) = 0 antes de que llegue el campo. Una función que se anula en el pasado no puede tener la parte real y la parte imaginaria por separado: una es la transformada de Hilbert de la otra.",
        "If the material is causal, G(t) = 0 before the field arrives. A function that vanishes in the past cannot have its real and imaginary parts chosen apart: one is the Hilbert transform of the other.",
        "如果材料是因果的，场到达之前 G(t) = 0。在过去为零的函数不能把实部和虚部分开选：一个是另一个的希尔伯特变换。",
        "Se o material é causal, G(t) = 0 antes de o campo chegar. Uma função que se anula no passado não pode ter a parte real e a imaginária escolhidas à parte: uma é a transformada de Hilbert da outra.",
    );
    let c = tx(
        lang,
        "El campo es un número real en el tiempo, así que ε₁(ω) es par y ε₂(ω) es impar. Al doblar la integral de −∞ a ∞ sobre las frecuencias positivas aparece el factor 2 y el núcleo ω′ / (ω′² − ω²). Lo que se anula a frecuencia infinita es ε₁ − ε∞: ε∞ junta la absorción que quedó por encima de tu ventana. P es el valor principal, el hueco alrededor de ω′ = ω.",
        "The field is a real number in time, so ε₁(ω) is even and ε₂(ω) is odd. Folding the integral from −∞ to ∞ onto positive frequencies produces the factor 2 and the kernel ω′ / (ω′² − ω²). What vanishes at infinite frequency is ε₁ − ε∞: ε∞ gathers the absorption that sits above your window. P is the principal value, the gap around ω′ = ω.",
        "场在时间上是实数，所以 ε₁(ω) 是偶函数，ε₂(ω) 是奇函数。把从 −∞ 到 ∞ 的积分折到正频率上，就出现因子 2 和核 ω′ / (ω′² − ω²)。在无穷频率处消失的是 ε₁ − ε∞：ε∞ 收进了窗口之上的吸收。P 是主值，也就是 ω′ = ω 周围的缺口。",
        "O campo é um número real no tempo, então ε₁(ω) é par e ε₂(ω) é ímpar. Dobrar a integral de −∞ a ∞ sobre as frequências positivas produz o fator 2 e o núcleo ω′ / (ω′² − ω²). O que se anula em frequência infinita é ε₁ − ε∞: ε∞ junta a absorção que ficou acima da sua janela. P é o valor principal, o vão em torno de ω′ = ω.",
    );
    view! {
        <p>{a}</p>
        <p>{b}</p>
        <p>{c}</p>
        <p class="formula">"ε₁(ω) = ε∞ + (2/π) P ∫₀^∞ ω′ ε₂(ω′) / (ω′² − ω²) dω′"</p>
    }
}

fn proof_g(lang: lightmatter::i18n::Lang) -> View {
    let a = tx(
        lang,
        "El triángulo k vale 1 en el ancla ωₖ. Baja en línea recta hasta 0 en ωₖ₋₁ y en ωₖ₊₁, y vale 0 fuera de ese tramo.",
        "Triangle k equals 1 at the anchor ωₖ. It falls in a straight line to 0 at ωₖ₋₁ and at ωₖ₊₁, and it is 0 outside that span.",
        "第 k 个三角在锚点 ωₖ 上等于 1。它沿直线降到 ωₖ₋₁ 和 ωₖ₊₁ 处的 0，这段之外为零。",
        "O triângulo k vale 1 na âncora ωₖ. Desce em linha reta até 0 em ωₖ₋₁ e em ωₖ₊₁, e vale 0 fora desse trecho.",
    );
    let b = tx(
        lang,
        "Esa tienda es la diferencia de dos rampas. La integral de Kramers–Kronig de una rampa, contra el núcleo ω′ / (ω′² − ω²), se hace por partes y deja logaritmos. Juntos son",
        "That tent is the difference of two ramps. The Kramers–Kronig integral of a ramp, against the kernel ω′ / (ω′² − ω²), is done by parts and leaves logarithms. Together they are",
        "这个帐篷是两条斜坡之差。斜坡对核 ω′ / (ω′² − ω²) 的克拉默斯–克罗尼格积分可以用分部积分，留下对数。合在一起是",
        "Essa tenda é a diferença de duas rampas. A integral de Kramers–Kronig de uma rampa, contra o núcleo ω′ / (ω′² − ω²), se faz por partes e deixa logaritmos. Juntos são",
    );
    let c = tx(
        lang,
        "El término del centro se resta porque en el ancla la pendiente cambia de signo: el triángulo sube y después baja. Con un signo más no sería ese triángulo, y ε₁ no le correspondería.",
        "The middle term is subtracted because at the anchor the slope changes sign: the triangle rises and then falls. With a plus sign it would not be that triangle, and ε₁ would not belong to it.",
        "中间一项要减去，因为在锚点处斜率变号：三角先升后降。若改成加号，就不再是这个三角，ε₁ 也不会与它对应。",
        "O termo do centro se subtrai porque na âncora a inclinação muda de sinal: o triângulo sobe e depois desce. Com um sinal de mais não seria esse triângulo, e ε₁ não lhe corresponderia.",
    );
    view! {
        <p>{a}</p>
        <p class="formula">"(ω − ωₖ₋₁) / (ωₖ − ωₖ₋₁)    entre ωₖ₋₁ y ωₖ"</p>
        <p class="formula">"(ωₖ₊₁ − ω) / (ωₖ₊₁ − ωₖ)    entre ωₖ y ωₖ₊₁"</p>
        <p>{b}</p>
        <p class="formula">"g(x, y) = (x+y) ln|x+y| − (x−y) ln|x−y|"</p>
        <p class="formula">"ε₁,ₖ(ω) = (1/π) [ g(ω,ωₖ₋₁)/(ωₖ−ωₖ₋₁) − (ωₖ₊₁−ωₖ₋₁) g(ω,ωₖ) / ((ωₖ−ωₖ₋₁)(ωₖ₊₁−ωₖ)) + g(ω,ωₖ₊₁)/(ωₖ₊₁−ωₖ) ]"</p>
        <p>{c}</p>
    }
}

fn proof_lm(lang: lightmatter::i18n::Lang) -> View {
    let a = tx(
        lang,
        "χ² suma las distancias al dato, cada una dividida por el error de ese punto. Sin tercera columna el error vale 1. Bajar χ² es el ajuste. La suavidad y el castigo de ε₂ negativa empujan el paso, y no se suman a este número.",
        "χ² adds the distances to the data, each divided by that point’s error. With no third column the error is 1. Lowering χ² is the fit. Smoothness and the penalty on negative ε₂ push the step, and they are not added into this number.",
        "χ² 把到数据的距离加起来，每一项除以该点的误差。没有第三列时误差为 1。降低 χ² 就是拟合。平滑和对负 ε₂ 的惩罚推动这一步，它们不加进这个数。",
        "χ² soma as distâncias ao dado, cada uma dividida pelo erro desse ponto. Sem terceira coluna o erro vale 1. Baixar χ² é o ajuste. A suavidade e o castigo de ε₂ negativa empurram o passo, e não se somam a este número.",
    );
    let b = tx(
        lang,
        "βₖ es la pendiente de bajada respecto al parámetro pₖ. αₖₗ es la curvatura, el producto de las derivadas del modelo. Cerca del mínimo, la segunda derivada del modelo estorba, y no se usa.",
        "βₖ is the downhill slope with respect to parameter pₖ. αₖₗ is the curvature, the product of the model derivatives. Near the minimum, the model’s second derivative gets in the way, and it is not used.",
        "βₖ 是相对参数 pₖ 的下坡斜率。αₖₗ 是曲率，即模型导数的乘积。靠近极小值时，模型的二阶导数会碍事，所以不用。",
        "βₖ é a inclinação de descida em relação ao parâmetro pₖ. αₖₗ é a curvatura, o produto das derivadas do modelo. Perto do mínimo, a segunda derivada do modelo atrapalha, e não se usa.",
    );
    let c = tx(
        lang,
        "El paso δp resuelve Σₗ αₖₗ (1 + δₖₗ λ) δpₗ = βₖ. Con λ pequeño se usa la curvatura y el salto es el del mínimo de esa parábola. Con λ grande la matriz queda casi diagonal y cada parámetro da un paso corto a lo largo de su pendiente.",
        "The step δp solves Σₗ αₖₗ (1 + δₖₗ λ) δpₗ = βₖ. With λ small, the curvature is used and the jump is that of the parabola’s minimum. With λ large the matrix is nearly diagonal and each parameter takes a short step along its slope.",
        "步长 δp 解的是 Σₗ αₖₗ (1 + δₖₗ λ) δpₗ = βₖ。λ 小时用曲率，跳跃就是那条抛物线的极小。λ 大时矩阵几乎是对角的，每个参数沿自己的斜率走一小步。",
        "O passo δp resolve Σₗ αₖₗ (1 + δₖₗ λ) δpₗ = βₖ. Com λ pequeno usa-se a curvatura e o salto é o do mínimo dessa parábola. Com λ grande a matriz fica quase diagonal e cada parâmetro dá um passo curto ao longo da sua inclinação.",
    );
    let d = tx(
        lang,
        "Si χ² baja, λ se divide entre 10. Si no baja, el paso se deshace y λ se multiplica por 10. Se para cuando χ² ya no baja un 1 % y ningún punto queda a más del 2 % del rango de los datos, o al llegar al tope de pasos. La barra ±1σ es la raíz de la diagonal de α⁻¹, con δχ² = 1, usando solo la curvatura de los datos.",
        "If χ² falls, λ is divided by 10. If it does not, the step is undone and λ is multiplied by 10. It stops when χ² no longer falls by 1% and no point is farther than 2% of the data span, or when the step cap is reached. The ±1σ bar is the square root of the diagonal of α⁻¹, with δχ² = 1, using only the curvature of the data.",
        "若 χ² 下降，λ 除以 10。若没有下降，这一步撤销，λ 乘以 10。当 χ² 不再下降 1%，且没有点离开数据范围的 2% 以上时停止，或者到达步数上限。±1σ 是 α⁻¹ 对角元的平方根，δχ² = 1，只用数据的曲率。",
        "Se χ² desce, λ divide-se por 10. Se não desce, o passo desfaz-se e λ multiplica-se por 10. Para quando χ² já não desce 1% e nenhum ponto fica a mais de 2% do intervalo dos dados, ou ao chegar ao teto de passos. A barra ±1σ é a raiz da diagonal de α⁻¹, com δχ² = 1, usando só a curvatura dos dados.",
    );
    view! {
        <p>{a}</p>
        <p class="formula">"βₖ = Σᵢ (yᵢ − fᵢ) / σᵢ² · ∂f/∂pₖ"</p>
        <p class="formula">"αₖₗ = Σᵢ (1/σᵢ²) (∂f/∂pₖ) (∂f/∂pₗ)"</p>
        <p>{b}</p>
        <p>{c}</p>
        <p>{d}</p>
    }
}

fn proof_nk(lang: lightmatter::i18n::Lang) -> View {
    let a = tx(
        lang,
        "El índice complejo se define para que su cuadrado sea la función dieléctrica. Con el signo de k igual al de ε₂,",
        "The complex index is defined so that its square is the dielectric function. With the sign of k the same as the sign of ε₂,",
        "复折射率的定义是它的平方等于介电函数。k 的符号与 ε₂ 相同，",
        "O índice complexo se define para que o seu quadrado seja a função dielétrica. Com o sinal de k igual ao de ε₂,",
    );
    let b = tx(
        lang,
        "De ahí |ε| = n² + k². Sumar y restar las dos ecuaciones despeja n y k.",
        "From that, |ε| = n² + k². Adding and subtracting the two equations solves for n and k.",
        "由此 |ε| = n² + k²。把两个方程相加、相减，就解出 n 和 k。",
        "Daqui |ε| = n² + k². Somar e subtrair as duas equações isola n e k.",
    );
    let c = tx(
        lang,
        "La reflectividad de un semiespacio, con la luz llegando en perpendicular desde el vacío, es el módulo al cuadrado de (1 − √ε) / (1 + √ε). En n y k queda la fórmula de la página.",
        "The reflectivity of a half-space, with light arriving perpendicularly from vacuum, is the squared modulus of (1 − √ε) / (1 + √ε). In n and k it becomes the formula on the page.",
        "半无限介质的反射率，光从真空垂直入射，是 (1 − √ε) / (1 + √ε) 的模的平方。用 n 和 k 写出来就是页面上的公式。",
        "A refletividade de um semiespaço, com a luz chegando na perpendicular a partir do vácuo, é o módulo ao quadrado de (1 − √ε) / (1 + √ε). Em n e k fica a fórmula da página.",
    );
    view! {
        <p>{a}</p>
        <p class="formula">"(n + i k)² = n² − k² + 2 i n k"</p>
        <p class="formula">"ε₁ = n² − k² ,    ε₂ = 2 n k"</p>
        <p>{b}</p>
        <p class="formula">"n = √( (|ε| + ε₁) / 2 ) ,    k = signo(ε₂) √( (|ε| − ε₁) / 2 )"</p>
        <p>{c}</p>
        <p class="formula">"R = ((n−1)² + k²) / ((n+1)² + k²)"</p>
    }
}

fn proof_drude(lang: lightmatter::i18n::Lang) -> View {
    let a = tx(
        lang,
        "En un oscilador de Lorentz el electrón está atado por un resorte de frecuencia ω₀. Un electrón libre no tiene ese resorte: ω₀ = 0. Con el convenio e^{−iωt} del laboratorio,",
        "In a Lorentz oscillator the electron is held by a spring of frequency ω₀. A free electron has no such spring: ω₀ = 0. In the laboratory’s e^{−iωt} convention,",
        "洛伦兹振子里，电子被频率为 ω₀ 的弹簧拉住。自由电子没有这根弹簧：ω₀ = 0。按实验室的 e^{−iωt} 约定，",
        "Num oscilador de Lorentz o elétron está preso por uma mola de frequência ω₀. Um elétron livre não tem essa mola: ω₀ = 0. Na convenção e^{−iωt} do laboratório,",
    );
    let b = tx(
        lang,
        "El denominador ω² + i γ ω, multiplicado por su conjugado, da ω⁴ + γ² ω². La parte real resta ωₚ² ω² sobre ese denominador. La parte imaginaria queda positiva y es la absorción de los electrones libres.",
        "The denominator ω² + i γ ω, multiplied by its conjugate, gives ω⁴ + γ² ω². The real part subtracts ωₚ² ω² over that denominator. The imaginary part stays positive and is the absorption of the free electrons.",
        "分母 ω² + i γ ω 乘上共轭，得到 ω⁴ + γ² ω²。实部是在这个分母上减去 ωₚ² ω²。虚部保持为正，就是自由电子的吸收。",
        "O denominador ω² + i γ ω, multiplicado pelo conjugado, dá ω⁴ + γ² ω². A parte real subtrai ωₚ² ω² sobre esse denominador. A parte imaginária fica positiva e é a absorção dos elétrons livres.",
    );
    view! {
        <p>{a}</p>
        <p class="formula">"ε(ω) = ε∞ − ωₚ² / (ω² + i γ ω)"</p>
        <p>{b}</p>
        <p class="formula">"ε₁ = ε∞ − ωₚ² ω² / (ω⁴ + γ² ω²)"</p>
        <p class="formula">"ε₂ = ωₚ² γ ω / (ω⁴ + γ² ω²)"</p>
    }
}
