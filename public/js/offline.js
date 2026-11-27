"use strict";

const SERVER_MAX_POINTS = 50_000;
const LOCAL_SLOW_WORK = 200_000_000;

let fitWorker = null;
let fitRequestId = 0;
const fitRequests = new Map();

const MEASURE_LABEL = { eps2: "ε₂", eps1: "ε₁", n: "n", k: "k", r: "R", alpha: "α", t: "T", lum: "I" };
const UNIT_LABEL = { cm: "cm⁻¹", ev: "eV", nm: "nm", otra: "ω" };
const CHART_TOOL_TEXT = {
  es: { axis: "Eje", expand: "Ampliar gráfica", close: "Cerrar gráfica", zoomIn: "Acercar", zoomOut: "Alejar", reset: "Restablecer" },
  en: { axis: "Axis", expand: "Expand chart", close: "Close chart", zoomIn: "Zoom in", zoomOut: "Zoom out", reset: "Reset" },
  zh: { axis: "坐标轴", expand: "放大图表", close: "关闭图表", zoomIn: "放大", zoomOut: "缩小", reset: "重置" },
  pt: { axis: "Eixo", expand: "Ampliar gráfico", close: "Fechar gráfico", zoomIn: "Aproximar", zoomOut: "Afastar", reset: "Restaurar" },
};

function chartToolText(key) {
  return (CHART_TOOL_TEXT[pageLang()] || CHART_TOOL_TEXT.es)[key];
}

const JOB_TEXT = {
  es: { title: "Análisis del servidor", queued: "En cola", running: "Análisis en curso", completed: "Análisis terminado", failed: "No se pudo completar", cancelled: "Cancelado", waiting: "Consultando estado…", view: "Abrir completo", cancel: "Cancelar", cancelling: "Cancelando…", step: "paso {step}/{max}", statusError: "No se pudo consultar el estado; seguiré intentando.", network: "No llegó la respuesta. El trabajo quedó guardado; consulta su estado antes de enviarlo otra vez.", conditions: "Condiciones", measure: "Medida", unit: "Unidad de ω", points: "Puntos", steps: "Pasos", smooth: "Suavidad", neg: "ε₂ < 0", eps: "ε∞", drude: "Drude", dataFit: "Datos y modelo", triangles: "ε₂ malla", anchors: "anclas", open: "Ver detalle", close: "Ocultar", live: "En vivo", auto: "auto", on: "sí", off: "no", data: "datos", model: "modelo", chisq: "χ²", resid: "rms residuo", result: "Resultado", loadingResult: "Cargando resultado…" },
  en: { title: "Server analyses", queued: "Queued", running: "Analysis in progress", completed: "Analysis complete", failed: "Could not complete", cancelled: "Cancelled", waiting: "Checking status…", view: "Open full", cancel: "Cancel", cancelling: "Cancelling…", step: "step {step}/{max}", statusError: "Could not check status; will keep trying.", network: "The response did not arrive. The job was saved; check its status before submitting again.", conditions: "Conditions", measure: "Measure", unit: "Unit of ω", points: "Points", steps: "Steps", smooth: "Smoothness", neg: "ε₂ < 0", eps: "ε∞", drude: "Drude", dataFit: "Data and model", triangles: "ε₂ mesh", anchors: "anchors", open: "Show detail", close: "Hide", live: "Live", auto: "auto", on: "yes", off: "no", data: "data", model: "model", chisq: "χ²", resid: "rms residual", result: "Result", loadingResult: "Loading result…" },
  zh: { title: "服务器分析", queued: "排队中", running: "正在分析", completed: "分析完成", failed: "未能完成", cancelled: "已取消", waiting: "正在查询状态…", view: "打开完整结果", cancel: "取消", cancelling: "正在取消…", step: "第 {step}/{max} 步", statusError: "无法查询状态，将继续重试。", network: "未收到响应。任务已保存；再次提交前请先查看状态。", conditions: "条件", measure: "测量量", unit: "ω 单位", points: "点数", steps: "步数", smooth: "平滑", neg: "ε₂ < 0", eps: "ε∞", drude: "Drude", dataFit: "数据与模型", triangles: "ε₂ 网格", anchors: "锚点", open: "展开", close: "收起", live: "实时", auto: "auto", on: "是", off: "否", data: "数据", model: "模型", chisq: "χ²", resid: "残差 rms", result: "结果", loadingResult: "正在加载结果…" },
  pt: { title: "Análises no servidor", queued: "Na fila", running: "Análise em andamento", completed: "Análise concluída", failed: "Não foi possível concluir", cancelled: "Cancelado", waiting: "Consultando estado…", view: "Abrir completo", cancel: "Cancelar", cancelling: "Cancelando…", step: "passo {step}/{max}", statusError: "Não foi possível consultar o estado; vou continuar tentando.", network: "A resposta não chegou. O trabalho foi salvo; confira o estado antes de enviar de novo.", conditions: "Condições", measure: "Medida", unit: "Unidade de ω", points: "Pontos", steps: "Passos", smooth: "Suavidade", neg: "ε₂ < 0", eps: "ε∞", drude: "Drude", dataFit: "Dados e modelo", triangles: "ε₂ malha", anchors: "âncoras", open: "Ver detalhe", close: "Ocultar", live: "Ao vivo", auto: "auto", on: "sim", off: "não", data: "dados", model: "modelo", chisq: "χ²", resid: "rms resíduo", result: "Resultado", loadingResult: "Carregando resultado…" },
};

function jobText(key) {
  return (JOB_TEXT[pageLang()] || JOB_TEXT.es)[key];
}

const RECENT_SERVER_JOBS_KEY = "lm-server-jobs";
const MAX_RECENT_SERVER_JOBS = 8;
let jobPollingTimer = 0;

function pageLang() {
  const tag = (document.documentElement.lang || "es").toLowerCase();
  if (tag.startsWith("zh")) return "zh";
  if (tag.startsWith("pt")) return "pt";
  if (tag.startsWith("en")) return "en";
  return "es";
}

function uiText(key) {
  const pack = {
    es: { merged: "{count} archivos unidos en un solo espectro.", mergeFail: "No se pudieron unir: ", chart: "Gráfica", empty: "La curva de lo que cargues aparece aquí, sin esperar al ajuste.", needRow: "Hace falta otra fila numérica.", tooBig: "El archivo supera 4 MB.", unread: "No pude leer ese archivo.", working: "Calculando…", local: "Calculando en este aparato…", localSlow: "Este análisis puede tardar bastante. Se ejecuta en segundo plano y la página seguirá respondiendo.", serverWorking: "Calculando en el servidor…", serverFailed: "No se pudo completar el análisis en el servidor. Conservé tus datos; puedes volver a intentarlo.", needNet: "Abre la app una vez con red para guardar el motor Rust en el aparato.", engine: "El motor Rust no pudo leer ese espectro.", fail: "No se pudo ajustar.", serverLimit: "El servidor admite hasta 50 000 puntos numéricos; este espectro tiene {count}. Reduce los datos y vuelve a intentarlo.", serverWork: "Esta medida y el número de pasos exceden el trabajo permitido en el servidor. Reduce los puntos o los pasos.", save: "Bajar gráfica:", svg: "Vectorial, editable en Inkscape o Illustrator", png: "Imagen a doble resolución" },
    en: { merged: "{count} files merged into one spectrum.", mergeFail: "They could not be merged: ", chart: "Plot", empty: "The curve of what you load appears here, before the fit.", needRow: "Another numeric row is needed.", tooBig: "The file is over 4 MB.", unread: "That file could not be read.", working: "Fitting…", local: "Fitting on this device…", localSlow: "This fit may take a while. It runs in the background, and the page will remain responsive.", serverWorking: "Fitting on the server…", serverFailed: "The server could not complete the fit. Your data are still here; you can try again.", needNet: "Open the app once online so the Rust engine is saved on this device.", engine: "The Rust engine could not read that spectrum.", fail: "The fit did not run.", serverLimit: "The server accepts up to 50,000 numeric points; this spectrum has {count}. Reduce the data and try again.", serverWork: "This measurement and iteration count exceed the server work limit. Reduce the number of points or iterations.", save: "Save plot:", svg: "Vector file, editable in Inkscape or Illustrator", png: "Image at double resolution" },
    zh: { merged: "已将 {count} 个文件合并为一条光谱。", mergeFail: "无法合并：", chart: "图", empty: "载入的曲线会出现在这里，不必等待拟合。", needRow: "还需要一行数字。", tooBig: "文件超过 4 MB。", unread: "无法读取这个文件。", working: "正在拟合…", local: "正在本机拟合…", localSlow: "此分析可能需要一些时间。它会在后台运行，页面仍可正常响应。", serverWorking: "正在服务器上拟合…", serverFailed: "服务器无法完成拟合。数据仍保留在此处，可以重试。", needNet: "请先联网打开一次，以便把 Rust 引擎存到这台设备。", engine: "Rust 引擎无法读取这份光谱。", fail: "未能完成拟合。", serverLimit: "服务器最多接受 50 000 个数值点；当前光谱有 {count} 个。请减少数据后重试。", serverWork: "此测量和步数设置超出服务器工作量限制。请减少点数或步数。", save: "保存图：", svg: "矢量图，可在 Inkscape 或 Illustrator 中编辑", png: "双倍分辨率图像" },
    pt: { merged: "{count} arquivos unidos em um só espectro.", mergeFail: "Não foi possível uni-los: ", chart: "Gráfico", empty: "A curva do que você carregar aparece aqui, sem esperar o ajuste.", needRow: "Falta outra linha numérica.", tooBig: "O arquivo passa de 4 MB.", unread: "Não consegui ler esse arquivo.", working: "Ajustando…", local: "Ajustando neste aparelho…", localSlow: "Esta análise pode demorar. Ela roda em segundo plano, e a página continua respondendo.", serverWorking: "Ajustando no servidor…", serverFailed: "O servidor não conseguiu concluir o ajuste. Seus dados continuam aqui; você pode tentar novamente.", needNet: "Abra o app uma vez com rede para guardar o motor Rust neste aparelho.", engine: "O motor Rust não conseguiu ler esse espectro.", fail: "Não foi possível ajustar.", serverLimit: "O servidor aceita até 50 000 pontos numéricos; este espectro tem {count}. Reduza os dados e tente novamente.", serverWork: "Esta medida e o número de iterações excedem o limite de trabalho do servidor. Reduza os pontos ou as iterações.", save: "Baixar gráfica:", svg: "Vetorial, editável no Inkscape ou no Illustrator", png: "Imagem em dobro da resolução" },
  };
  return (pack[pageLang()] || pack.es)[key];
}

function parseColumns(text) {
  const xs = [];
  const ys = [];
  for (const line of String(text).split(/\r?\n/)) {
    const t = line.trim();
    if (!t || "#*!%".includes(t[0])) continue;
    const parts = t.split(/[\s,;]+/).filter(Boolean);
    if (parts.length < 2) continue;
    const a = Number(parts[0]);
    const b = Number(parts[1]);
    if (!Number.isFinite(a) || !Number.isFinite(b)) continue;
    xs.push(a);
    ys.push(b);
  }
  return { xs, ys };
}

function axisName(select, table, fallback) {
  const key = select ? select.value : "";
  return table[key] || fallback;
}

function niceStep(span, count) {
  const raw = span / Math.max(1, count);
  if (!(raw > 0)) return 1;
  const p = 10 ** Math.floor(Math.log10(raw));
  const n = raw / p;
  const f = n < 1.5 ? 1 : n < 3 ? 2 : n < 7 ? 5 : 10;
  return f * p;
}

function thinSeries(xs, ys, cap) {
  const n = xs.length;
  if (n <= cap) return [xs, ys];
  const keep = new Uint8Array(n);
  keep[0] = 1;
  keep[n - 1] = 1;
  for (let i = 1; i < n - 1; i++) {
    const left = ys[i] - ys[i - 1];
    const right = ys[i + 1] - ys[i];
    if (left * right < 0) keep[i] = 1;
  }
  const step = Math.ceil(n / cap);
  for (let i = 0; i < n; i += step) keep[i] = 1;
  const ox = [];
  const oy = [];
  for (let i = 0; i < n; i++) {
    if (keep[i]) {
      ox.push(xs[i]);
      oy.push(ys[i]);
    }
  }
  if (ox.length <= cap * 2) return [ox, oy];
  const stride = Math.ceil(ox.length / cap);
  const tx = [];
  const ty = [];
  for (let i = 0; i < ox.length; i += stride) {
    tx.push(ox[i]);
    ty.push(oy[i]);
  }
  tx.push(ox[ox.length - 1]);
  ty.push(oy[oy.length - 1]);
  return [tx, ty];
}

function drawPreview() {
  const form = document.querySelector("form[action='/lab/run']");
  const figure = document.querySelector("[data-lm-preview]");
  const slot = document.querySelector("[data-lm-preview-slot]");
  const meta = document.querySelector("[data-lm-preview-meta]");
  const box = form ? form.querySelector("textarea[name='spectrum']") : null;
  if (!figure || !slot || !box) return;
  figure.querySelectorAll(".chart-tip, .chart-tools").forEach((node) => node.remove());
  delete figure.dataset.lmReady;
  const { xs, ys } = parseColumns(box.value);
  const yName = axisName(form && form.querySelector("select[name='measure']"), MEASURE_LABEL, "medida");
  const xName = axisName(form && form.querySelector("select[name='unit']"), UNIT_LABEL, "ω");
  const caption = figure.querySelector("figcaption");
  if (caption) caption.textContent = uiText("chart");
  if (xs.length < 2) {
    slot.innerHTML = `<p class="preview-empty">${uiText("empty")}</p>`;
    if (meta) meta.textContent = xs.length === 1 ? uiText("needRow") : "";
    return;
  }
  const order = xs.map((_, i) => i).sort((a, b) => xs[a] - xs[b] || a - b);
  const sx = order.map((i) => xs[i]);
  const sy = order.map((i) => ys[i]);
  let xmin = sx[0];
  let xmax = sx[sx.length - 1];
  let ymin = sy[0];
  let ymax = sy[0];
  for (const v of sy) {
    if (v < ymin) ymin = v;
    if (v > ymax) ymax = v;
  }
  const yRawLo = ymin;
  const yRawHi = ymax;
  if (xmax - xmin < 1e-12) xmax = xmin + 1;
  const nonneg = ymin >= 0;
  if (ymax - ymin < 1e-12) {
    const half = Math.max(Math.abs(ymax) * 0.1, 0.5);
    ymin -= half;
    ymax += half;
  }
  const pad = (ymax - ymin) * 0.08;
  ymin -= pad;
  ymax += pad;
  if (nonneg) ymin = Math.max(0, ymin);
  const log = xmin > 0 && xmax / xmin > 20;
  const width = 720;
  const height = 340;
  const left = 74;
  const right = 18;
  const top = 36;
  const bottom = 48;
  const pw = width - left - right;
  const ph = height - top - bottom;
  const xOf = (x) => {
    if (log) return left + ((Math.log(x) - Math.log(xmin)) / (Math.log(xmax) - Math.log(xmin))) * pw;
    return left + ((x - xmin) / (xmax - xmin)) * pw;
  };
  const yOf = (y) => top + (1 - (y - ymin) / (ymax - ymin)) * ph;
  const [dx, dy] = thinSeries(sx, sy, 1500);
  let d = "";
  for (let i = 0; i < dx.length; i++) {
    const px = xOf(dx[i]);
    const py = yOf(dy[i]);
    if (!Number.isFinite(px) || !Number.isFinite(py)) continue;
    d += `${d ? "L" : "M"}${px.toFixed(2)},${py.toFixed(2)}`;
  }
  const parts = [
    `<rect width="720" height="340" fill="var(--chart-bg)"/>`,
    `<line x1="${left}" y1="${top}" x2="${left}" y2="${top + ph}" stroke="var(--chart-grid)"/>`,
    `<line x1="${left}" y1="${top + ph}" x2="${left + pw}" y2="${top + ph}" stroke="var(--chart-grid)"/>`,
  ];
  const xTicks = log
    ? (() => {
        const out = [];
        for (let e = Math.ceil(Math.log10(xmin)); e <= Math.floor(Math.log10(xmax)); e++) out.push(10 ** e);
        return out.length >= 2 ? out : [xmin, xmax];
      })()
    : (() => {
        const step = niceStep(xmax - xmin, 5);
        const out = [];
        for (let v = Math.ceil(xmin / step) * step; v <= xmax + step * 1e-6; v += step) out.push(v);
        return out;
      })();
  const yStep = niceStep(ymax - ymin, 4);
  for (let v = Math.ceil(ymin / yStep) * yStep; v <= ymax + yStep * 1e-6; v += yStep) {
    const py = yOf(v);
    if (py < top - 1 || py > top + ph + 1) continue;
    parts.push(`<line x1="${left}" y1="${py.toFixed(1)}" x2="${left + pw}" y2="${py.toFixed(1)}" stroke="var(--chart-grid)"/>`);
    parts.push(`<text x="${left - 8}" y="${(py + 4).toFixed(1)}" fill="var(--chart-ink)" font-size="11" text-anchor="end" font-family="ui-monospace,monospace">${fmt(v)}</text>`);
  }
  for (const v of xTicks) {
    if (!(v > 0) && log) continue;
    const px = xOf(v);
    if (px < left - 1 || px > left + pw + 1) continue;
    parts.push(`<text x="${px.toFixed(1)}" y="${top + ph + 18}" fill="var(--chart-ink)" font-size="11" text-anchor="middle" font-family="ui-monospace,monospace">${fmt(v)}</text>`);
  }
  parts.push(`<text x="${left + pw / 2}" y="${height - 8}" fill="var(--chart-ink)" font-size="12" text-anchor="middle">${xName}</text>`);
  parts.push(`<text x="16" y="${top + ph / 2}" fill="var(--chart-ink)" font-size="12" text-anchor="middle" transform="rotate(-90 16 ${top + ph / 2})">${yName}</text>`);
  parts.push(`<path data-series="datos" fill="none" stroke="#9aa7b2" stroke-width="1.6" d="${d}"/>`);
  slot.innerHTML =
    `<svg viewBox="0 0 ${width} ${height}" role="img" aria-label="Datos cargados" data-xmin="${xmin}" data-xmax="${xmax}" data-log="${log ? 1 : 0}" data-ymin="${ymin}" data-ymax="${ymax}" data-left="${left}" data-top="${top}" data-pw="${pw}" data-ph="${ph}" data-xlabel="${xName}">${parts.join("")}</svg>`;
  if (meta) {
    meta.textContent = `${xs.length} puntos · ${xName} ${fmt(sx[0])} … ${fmt(sx[sx.length - 1])} · ${yName} ${fmt(yRawLo)} … ${fmt(yRawHi)}${log ? " · eje log" : ""}`;
  }
  enhanceChart(figure);
}

function initLabControls() {
  const form = document.querySelector("form[action='/lab/run']");
  const drop = document.querySelector("[data-lm-drop]");
  if (!form || !drop) return;
  const input = drop.querySelector("input[type='file']");
  const name = drop.querySelector("[data-lm-file]");
  const box = form.querySelector("textarea[name='spectrum']");
  const idle = name ? name.textContent : "";
  const meta = document.querySelector("[data-lm-preview-meta]");
  const showIdle = () => {
    drop.classList.remove("has-file");
    if (name) name.textContent = idle;
  };
  if (input) {
    drop.addEventListener("click", (event) => {
      if (event.target !== input) input.click();
    });
    input.addEventListener("change", async () => {
      const picked = Array.from(input.files || []);
      if (!picked.length) return;
      const total = picked.reduce((sum, file) => sum + file.size, 0);
      if (total > 4_000_000) {
        if (meta) meta.textContent = uiText("tooBig");
        input.value = "";
        return;
      }
      const parts = [];
      try {
        for (const file of picked) parts.push([file.name, await file.text()]);
      } catch {
        if (meta) meta.textContent = uiText("unread");
        input.value = "";
        return;
      }
      input.value = "";
      if (parts.length === 1) {
        const file = picked[0];
        if (box) box.value = parts[0][1];
        drop.classList.add("has-file");
        if (name) name.textContent = `${file.name} · ${(file.size / 1024).toFixed(1)} kB · ya está en el cuadro`;
        drawPreview();
        return;
      }
      const merged = await mergeInWorker(parts);
      if (!merged.ok) {
        if (meta) meta.textContent = `${uiText("mergeFail")}${merged.error || ""}`;
        return;
      }
      if (box) box.value = merged.text;
      drop.classList.add("has-file");
      if (name) {
        name.textContent = `${uiText("merged").replace("{count}", String(parts.length))} ${parts.map(([n]) => n).join(" + ")}`;
      }
      drawPreview();
    });
  }
  ["dragenter", "dragover"].forEach((type) => drop.addEventListener(type, () => drop.classList.add("is-over")));
  ["dragleave", "drop"].forEach((type) => drop.addEventListener(type, () => drop.classList.remove("is-over")));
  if (box) {
    let timer = 0;
    box.addEventListener("input", () => {
      clearTimeout(timer);
      timer = setTimeout(drawPreview, 120);
      showIdle();
    });
  }
  form.addEventListener("reset", () => setTimeout(() => { showIdle(); drawPreview(); }));
  const measureSelect = form.querySelector("select[name='measure']");
  const unitSelect = form.querySelector("select[name='unit']");
  if (measureSelect) measureSelect.addEventListener("change", drawPreview);
  if (unitSelect) unitSelect.addEventListener("change", drawPreview);
  drawPreview();
  const measure = form.querySelector("select[name='measure']");
  if (measure) {
    const fields = form.querySelectorAll("[data-lm-for], [data-lm-unless]");
    const epsInf = form.querySelector("input[name='eps_inf']");
    const sync = () => {
      if (epsInf) epsInf.placeholder = ["eps1", "n", "r"].includes(measure.value) ? "auto" : "1";
      fields.forEach((el) => {
        const unless = el.getAttribute("data-lm-unless");
        if (unless) {
          el.hidden = unless.split(/\s+/).includes(measure.value);
          return;
        }
        el.hidden = el.getAttribute("data-lm-for") !== measure.value;
      });
    };
    measure.addEventListener("change", sync);
    sync();
  }
}

// ---- Gráficas: valor exacto bajo el mouse y descarga en SVG o PNG ----

function fmt(v) {
  if (!Number.isFinite(v)) return "—";
  const a = Math.abs(v);
  if (v === 0) return "0";
  if (a >= 1e5 || a < 1e-3) return v.toExponential(3);
  if (a >= 100) return v.toFixed(2);
  if (a >= 1) return v.toFixed(4);
  return v.toPrecision(4);
}

function parsePath(d) {
  const pts = [];
  const re = /[ML]\s*(-?[\d.]+(?:e-?\d+)?),(-?[\d.]+(?:e-?\d+)?)/g;
  let m;
  while ((m = re.exec(d))) pts.push([Number(m[1]), Number(m[2])]);
  return pts;
}

function enhanceChart(figure) {
  const svg = figure.querySelector("svg[data-xmin]");
  if (!svg || figure.dataset.lmReady) return;
  figure.dataset.lmReady = "1";
  const ds = svg.dataset;
  const xmin = Number(ds.xmin), xmax = Number(ds.xmax), log = ds.log === "1";
  const ymin = Number(ds.ymin), ymax = Number(ds.ymax);
  const left = Number(ds.left), top = Number(ds.top), pw = Number(ds.pw), ph = Number(ds.ph);
  let displayXMin = xmin, displayXMax = xmax, displayLog = log;
  const toX = (px) => (displayLog ? Math.exp(Math.log(displayXMin) + ((px - left) / pw) * (Math.log(displayXMax) - Math.log(displayXMin))) : displayXMin + ((px - left) / pw) * (displayXMax - displayXMin));
  const toY = (py) => ymin + (1 - (py - top) / ph) * (ymax - ymin);
  const series = Array.from(svg.querySelectorAll("path[data-series]")).map((p) => ({
    name: p.dataset.series,
    color: p.getAttribute("stroke"),
    pts: parsePath(p.getAttribute("d") || ""),
  })).filter((s) => s.pts.length);
  if (!series.length) return;
  const originalSeries = series.map((s) => s.pts.map((point) => [...point]));
  const originalPaths = Array.from(svg.querySelectorAll("path[d]"), (path) => ({ node: path, d: path.getAttribute("d") || "" }));
  const baseUnit = ds.xunit;
  const baseValueAtPixel = (px) => (log ? Math.exp(Math.log(xmin) + ((px - left) / pw) * (Math.log(xmax) - Math.log(xmin))) : xmin + ((px - left) / pw) * (xmax - xmin));
  const xInUnit = (value, unit) => {
    const energy = baseUnit === "cm" ? value * 1.23984193e-4 : value;
    if (unit === "cm") return energy / 1.23984193e-4;
    if (unit === "nm") return energy > 0 ? 1239.84193 / energy : NaN;
    return energy;
  };
  const mapDisplayX = (value) => displayLog
    ? left + ((Math.log(value) - Math.log(displayXMin)) / (Math.log(displayXMax) - Math.log(displayXMin))) * pw
    : left + ((value - displayXMin) / (displayXMax - displayXMin)) * pw;
  const setAxisUnit = (unit) => {
    if (unit === baseUnit) {
      displayXMin = xmin;
      displayXMax = xmax;
      displayLog = log;
      originalPaths.forEach(({ node, d }) => node.setAttribute("d", d));
      series.forEach((s, i) => { s.pts = originalSeries[i].map((point) => [...point]); });
    } else {
      const endpoints = [xInUnit(xmin, unit), xInUnit(xmax, unit)];
      displayXMin = Math.min(...endpoints);
      displayXMax = Math.max(...endpoints);
      displayLog = log && displayXMin > 0;
      const convertPixel = (px) => mapDisplayX(xInUnit(baseValueAtPixel(px), unit));
      const pointPattern = /([ML])\s*(-?[\d.]+(?:e[+-]?\d+)?),(-?[\d.]+(?:e[+-]?\d+)?)/gi;
      originalPaths.forEach(({ node, d }) => node.setAttribute("d", d.replace(pointPattern, (_match, command, px, py) => `${command}${convertPixel(Number(px)).toFixed(2)},${py}`)));
      series.forEach((s, i) => {
        s.pts = originalSeries[i]
          .map(([px, py]) => [convertPixel(px), py])
          .sort((a, b) => a[0] - b[0]);
      });
    }
    const tickNodes = svg.querySelectorAll(".chart-x-tick");
    tickNodes.forEach((node, i) => {
      const count = Math.max(1, tickNodes.length - 1);
      const fraction = i / count;
      const value = displayLog
        ? Math.exp(Math.log(displayXMin) + fraction * (Math.log(displayXMax) - Math.log(displayXMin)))
        : displayXMin + fraction * (displayXMax - displayXMin);
      node.setAttribute("x", mapDisplayX(value).toFixed(1));
      node.textContent = fmt(value);
    });
    const axisLabel = unit === "cm" ? "ω (cm⁻¹)" : unit === "nm" ? "λ (nm)" : "E (eV)";
    ds.xlabel = axisLabel;
    svg.querySelectorAll(".chart-x-label").forEach((node) => { node.textContent = axisLabel + (displayLog ? "  (log)" : ""); });
  };

  const ns = "http://www.w3.org/2000/svg";
  const cursor = document.createElementNS(ns, "line");
  cursor.setAttribute("stroke", "var(--chart-label)");
  cursor.setAttribute("stroke-dasharray", "3 3");
  cursor.setAttribute("y1", top);
  cursor.setAttribute("y2", top + ph);
  cursor.style.display = "none";
  svg.appendChild(cursor);
  const dots = series.map((s) => {
    const c = document.createElementNS(ns, "circle");
    c.setAttribute("r", "3.5");
    c.setAttribute("fill", s.color);
    c.setAttribute("stroke", "var(--chart-bg)");
    c.setAttribute("stroke-width", "1.5");
    c.style.display = "none";
    svg.appendChild(c);
    return c;
  });
  const tip = document.createElement("div");
  tip.className = "chart-tip";
  tip.hidden = true;
  figure.appendChild(tip);

  const nearest = (pts, px) => {
    let lo = 0, hi = pts.length - 1;
    while (hi - lo > 1) {
      const mid = (lo + hi) >> 1;
      if (pts[mid][0] <= px) lo = mid; else hi = mid;
    }
    return Math.abs(pts[lo][0] - px) <= Math.abs(pts[hi][0] - px) ? pts[lo] : pts[hi];
  };
  const hide = () => {
    cursor.style.display = "none";
    dots.forEach((d) => (d.style.display = "none"));
    tip.hidden = true;
  };
  const move = (clientX, clientY) => {
    const pointer = svg.createSVGPoint();
    pointer.x = clientX;
    pointer.y = clientY;
    const px = pointer.matrixTransform(svg.getScreenCTM().inverse()).x;
    if (px < left - 4 || px > left + pw + 4) return hide();
    let best = null;
    const rows = series.map((s, i) => {
      const p = nearest(s.pts, px);
      if (!best || Math.abs(p[0] - px) < Math.abs(best[0] - px)) best = p;
      dots[i].setAttribute("cx", p[0]);
      dots[i].setAttribute("cy", p[1]);
      dots[i].style.display = "";
      return `<span class="tip-row"><i style="background:${s.color}"></i>${s.name} <b>${fmt(toY(p[1]))}</b></span>`;
    });
    cursor.setAttribute("x1", best[0]);
    cursor.setAttribute("x2", best[0]);
    cursor.style.display = "";
    tip.innerHTML = `<span class="tip-x">${ds.xlabel || "ω"} = <b>${fmt(toX(best[0]))}</b></span>${rows.join("")}`;
    tip.hidden = false;
    const frect = figure.getBoundingClientRect();
    const x = clientX - frect.left;
    const flip = x > frect.width * 0.6;
    tip.style.left = flip ? "" : `${x + 14}px`;
    tip.style.right = flip ? `${frect.width - x + 14}px` : "";
    tip.style.top = `${clientY - frect.top - 10}px`;
  };
  svg.addEventListener("mousemove", (e) => move(e.clientX, e.clientY));
  svg.addEventListener("mouseleave", hide);
  svg.addEventListener("touchstart", (e) => { const t = e.touches[0]; if (t) move(t.clientX, t.clientY); }, { passive: true });
  svg.addEventListener("touchmove", (e) => { const t = e.touches[0]; if (t) move(t.clientX, t.clientY); }, { passive: true });
  svg.addEventListener("touchend", hide);

  const tools = document.createElement("div");
  tools.className = "chart-tools";
  const title = (figure.querySelector("figcaption") || {}).textContent || "grafica";
  const slug = title
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .split(/[—·(]/)[0]
    .replace(/[^a-z0-9ε₁₂σα]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 40) || "grafica";
  const serialize = () => {
    hide();
    const clone = svg.cloneNode(true);
    clone.setAttribute("xmlns", ns);
    clone.setAttribute("width", "1440");
    clone.setAttribute("height", "680");
    clone.querySelectorAll("circle, line[stroke-dasharray]").forEach((n) => n.remove());
    return new XMLSerializer().serializeToString(clone);
  };
  const save = (blob, name) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 2000);
  };
  const svgBtn = document.createElement("button");
  svgBtn.type = "button";
  svgBtn.textContent = "SVG";
  svgBtn.title = uiText("svg");
  svgBtn.addEventListener("click", () => save(new Blob([serialize()], { type: "image/svg+xml;charset=utf-8" }), `lightmatter-${slug}.svg`));
  const pngBtn = document.createElement("button");
  pngBtn.type = "button";
  pngBtn.textContent = "PNG";
  pngBtn.title = uiText("png");
  pngBtn.addEventListener("click", () => {
    const img = new Image();
    const url = URL.createObjectURL(new Blob([serialize()], { type: "image/svg+xml;charset=utf-8" }));
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = 1440;
      canvas.height = 680;
      const ctx = canvas.getContext("2d");
      const ink = getComputedStyle(document.documentElement).getPropertyValue("--chart-bg").trim() || "#07141b";
      ctx.fillStyle = ink;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
      URL.revokeObjectURL(url);
      canvas.toBlob((blob) => blob && save(blob, `lightmatter-${slug}.png`), "image/png");
    };
    img.src = url;
  });
  let axisSelect = null;
  if ((baseUnit === "cm" || baseUnit === "ev") && !title.toLowerCase().includes("luminiscencia")) {
    axisSelect = document.createElement("select");
    axisSelect.setAttribute("aria-label", chartToolText("axis"));
    for (const [value, label] of [["cm", "cm⁻¹"], ["ev", "eV"], ["nm", "nm"]]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      axisSelect.append(option);
    }
    axisSelect.value = baseUnit;
    axisSelect.addEventListener("change", () => setAxisUnit(axisSelect.value));
  }
  let zoom = 1;
  const initialView = (svg.getAttribute("viewBox") || "0 0 720 340").split(/\s+/).map(Number);
  const setZoom = (value) => {
    zoom = Math.max(1, Math.min(8, value));
    const [x0, y0, width, height] = initialView;
    const nextWidth = width / zoom;
    const nextHeight = height / zoom;
    svg.setAttribute("viewBox", `${x0 + (width - nextWidth) / 2} ${y0 + (height - nextHeight) / 2} ${nextWidth} ${nextHeight}`);
  };
  const toolButton = (text, title, action) => {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = text;
    button.title = title;
    button.setAttribute("aria-label", title);
    button.addEventListener("click", action);
    return button;
  };
  const zoomOutBtn = toolButton("−", chartToolText("zoomOut"), () => setZoom(zoom / 1.25));
  const resetBtn = toolButton("1:1", chartToolText("reset"), () => {
    if (axisSelect) {
      axisSelect.value = baseUnit;
      setAxisUnit(baseUnit);
    }
    setZoom(1);
  });
  const zoomInBtn = toolButton("+", chartToolText("zoomIn"), () => setZoom(zoom * 1.25));
  const expandBtn = toolButton("⛶", chartToolText("expand"), () => {
    const expanded = figure.classList.toggle("is-expanded");
    document.body.classList.toggle("chart-expanded", expanded);
    expandBtn.textContent = expanded ? "×" : "⛶";
    expandBtn.title = chartToolText(expanded ? "close" : "expand");
    expandBtn.setAttribute("aria-label", expandBtn.title);
    expandBtn.setAttribute("aria-expanded", String(expanded));
  });
  const label = document.createElement("span");
  label.textContent = uiText("save");
  if (axisSelect) tools.append(axisSelect);
  tools.append(expandBtn, zoomOutBtn, resetBtn, zoomInBtn, label, svgBtn, pngBtn);
  figure.appendChild(tools);
}

function enhanceCharts(root) {
  (root || document).querySelectorAll("figure.chart").forEach(enhanceChart);
}
// ---- Sin red: el motor Rust corre en el aparato ----

function showLiveMessage(live, message, kind = "muted") {
  if (!live) return;
  live.hidden = false;
  live.classList.remove("is-idle");
  live.replaceChildren();
  const paragraph = document.createElement("p");
  paragraph.className = kind;
  paragraph.setAttribute("role", kind === "muted" ? "status" : "alert");
  paragraph.textContent = message;
  live.append(paragraph);
}

function readRecentJobs() {
  try {
    const jobs = JSON.parse(localStorage.getItem(RECENT_SERVER_JOBS_KEY) || "[]");
    return Array.isArray(jobs) ? jobs.filter((job) => job && typeof job.id === "string" && job.id.startsWith("job_")) : [];
  } catch {
    return [];
  }
}

function rememberServerJob(id, meta) {
  if (!/^job_[a-f0-9_]{8,60}$/i.test(id)) return;
  const jobs = readRecentJobs().filter((job) => job.id !== id);
  const prev = readRecentJobs().find((job) => job.id === id) || {};
  jobs.push({
    ...prev,
    id,
    createdAt: Date.now(),
    ...(meta && typeof meta === "object" ? meta : {}),
  });
  try {
    localStorage.setItem(RECENT_SERVER_JOBS_KEY, JSON.stringify(jobs.slice(-MAX_RECENT_SERVER_JOBS)));
  } catch {}
}

function jobMetaFromForm(form, text) {
  const { xs, ys } = parseColumns(text || "");
  const order = xs.map((_, i) => i).sort((a, b) => xs[a] - xs[b] || a - b);
  const sx = order.map((i) => xs[i]);
  const sy = order.map((i) => ys[i]);
  const [px, py] = thinSeries(sx, sy, 120);
  return {
    measure: String(form.querySelector("select[name='measure']")?.value || "eps2"),
    unit: String(form.querySelector("select[name='unit']")?.value || "otra"),
    iter: Number(form.querySelector("input[name='iter']")?.value) || 2000,
    smooth: Number(form.querySelector("input[name='smooth']")?.value ?? 0),
    neg: Number(form.querySelector("input[name='neg']")?.value ?? 100),
    eps_inf: String(form.querySelector("input[name='eps_inf']")?.value || ""),
    drude: form.querySelector("input[name='drude']")?.checked === true,
    points: xs.length,
    preview: { x: px, y: py },
  };
}

function optionLabel(options, key, fallback) {
  if (!options) return fallback || "—";
  const value = options[key];
  if (value == null || value === "") return fallback || jobText("auto");
  return String(value);
}

function fmtSci(v) {
  if (!Number.isFinite(v)) return "—";
  const a = Math.abs(v);
  if (a !== 0 && (a < 1e-3 || a >= 1e4)) return v.toExponential(3);
  return v.toPrecision(4);
}

function drawJobSpark(host, curve, measureKey, unitKey) {
  if (!host) return;
  const x = curve?.x || [];
  const data = curve?.data || [];
  const model = curve?.model || [];
  const meshX = curve?.mesh_x || [];
  const meshY = curve?.mesh_y || [];
  if (x.length < 2 || data.length < 2) {
    host.innerHTML = "";
    return;
  }
  const yName = MEASURE_LABEL[measureKey] || measureKey || "y";
  const xName = UNIT_LABEL[unitKey] || unitKey || "ω";
  let xmin = x[0];
  let xmax = x[x.length - 1];
  let ymin = data[0];
  let ymax = data[0];
  for (const v of data) {
    if (v < ymin) ymin = v;
    if (v > ymax) ymax = v;
  }
  for (const v of model) {
    if (!Number.isFinite(v)) continue;
    if (v < ymin) ymin = v;
    if (v > ymax) ymax = v;
  }
  if (!(xmax > xmin)) xmax = xmin + 1;
  if (!(ymax > ymin)) {
    const half = Math.max(Math.abs(ymax) * 0.1, 0.5);
    ymin -= half;
    ymax += half;
  }
  const pad = (ymax - ymin) * 0.08;
  ymin -= pad;
  ymax += pad;
  if (data.every((v) => v >= 0) && model.every((v) => !Number.isFinite(v) || v >= 0)) ymin = Math.max(0, ymin);
  const log = xmin > 0 && xmax / xmin > 20;
  const width = 640;
  const height = 228;
  const left = 52;
  const right = 14;
  const top = 28;
  const bottom = 34;
  const pw = width - left - right;
  const ph = height - top - bottom;
  const xOf = (v) => {
    if (log) return left + ((Math.log(v) - Math.log(xmin)) / (Math.log(xmax) - Math.log(xmin))) * pw;
    return left + ((v - xmin) / (xmax - xmin)) * pw;
  };
  const yOf = (v) => top + (1 - (v - ymin) / (ymax - ymin)) * ph;
  const pathOf = (xs, ys) => {
    let d = "";
    for (let i = 0; i < xs.length; i++) {
      const px = xOf(xs[i]);
      const py = yOf(ys[i]);
      if (!Number.isFinite(px) || !Number.isFinite(py)) continue;
      d += `${d ? "L" : "M"}${px.toFixed(1)},${py.toFixed(1)}`;
    }
    return d;
  };
  const metrics = [];
  if (Number.isFinite(curve?.chisq)) metrics.push(`${jobText("chisq")} ${fmtSci(curve.chisq)}`);
  if (Number.isFinite(curve?.resid_rms)) metrics.push(`${jobText("resid")} ${fmtSci(curve.resid_rms)}`);
  const hasMesh = meshX.length >= 3 && meshY.length === meshX.length;
  const parts = [
    `<rect width="${width}" height="${height}" rx="12" fill="var(--chart-bg)"/>`,
    `<text x="${left}" y="16" fill="var(--muted)" font-size="11">${jobText("dataFit")} · ${yName}</text>`,
    `<text x="${width - right}" y="16" fill="var(--muted)" font-size="11" text-anchor="end">${metrics.join(" · ")}</text>`,
    `<circle cx="${left}" cy="${height - 14}" r="3" fill="#3ee0c5"/><text x="${left + 8}" y="${height - 10}" fill="var(--muted)" font-size="10">${jobText("data")}</text>`,
    `<circle cx="${left + 70}" cy="${height - 14}" r="3" fill="#f2c14e"/><text x="${left + 78}" y="${height - 10}" fill="var(--muted)" font-size="10">${jobText("model")}</text>`,
  ];
  if (hasMesh) {
    parts.push(
      `<path d="M${left + 140},${height - 10} L${left + 148},${height - 18} L${left + 156},${height - 10} Z" fill="rgba(122,162,255,0.45)" stroke="rgba(122,162,255,0.95)" stroke-width="0.8"/>` +
      `<text x="${left + 162}" y="${height - 10}" fill="var(--muted)" font-size="10">${jobText("triangles")}</text>`
    );
  }

  // Triángulos Aₖ sobre el mismo eje ω que R: altura relativa de ε₂, debajo de datos/modelo.
  if (hasMesh) {
    const sorted = meshY.filter((v) => Number.isFinite(v) && v >= 0).slice().sort((a, b) => a - b);
    const at = (p) => sorted[Math.min(sorted.length - 1, Math.max(0, Math.floor(sorted.length * p)))] || 0;
    const p50 = at(0.5);
    const p90 = at(0.9);
    const p99 = at(0.99);
    const mMax = Math.max(p90 * 1.35, p50 * 4, p99 * 0.55, 1e-12);
    const baseY = top + ph;
    const meshYOf = (v) => {
      const t = Math.min(Math.max(v, 0), mMax) / mMax;
      return baseY - t * ph * 0.92;
    };
    const y0 = baseY;
    let tris = "";
    let rug = "";
    // Hasta ~100 triángulos visibles: en reststrahlen se ven subir y bajar en vivo.
    const triStep = Math.max(1, Math.floor(meshX.length / 100));
    for (let i = 1; i < meshX.length - 1; i += triStep) {
      const h = meshY[i];
      if (!(h > mMax * 0.015)) continue;
      const x0 = xOf(meshX[i - 1]);
      const x1 = xOf(meshX[i]);
      const x2 = xOf(meshX[i + 1]);
      const y1 = meshYOf(h);
      if (![x0, x1, x2, y1].every(Number.isFinite)) continue;
      if (x1 < left - 2 || x1 > left + pw + 2) continue;
      tris += `<path d="M${x0.toFixed(1)},${y0.toFixed(1)} L${x1.toFixed(1)},${y1.toFixed(1)} L${x2.toFixed(1)},${y0.toFixed(1)} Z" fill="rgba(122,162,255,0.18)" stroke="rgba(122,162,255,0.55)" stroke-width="0.7"/>`;
    }
    // Contorno de ε₂ (envolvente de la malla) alineado en ω con R.
    let envelope = "";
    let area = `M${xOf(meshX[0]).toFixed(1)},${y0.toFixed(1)}`;
    for (let i = 0; i < meshX.length; i++) {
      const px = xOf(meshX[i]);
      const py = meshYOf(meshY[i]);
      if (!Number.isFinite(px) || !Number.isFinite(py)) continue;
      envelope += `${envelope ? "L" : "M"}${px.toFixed(1)},${py.toFixed(1)}`;
      area += ` L${px.toFixed(1)},${py.toFixed(1)}`;
      if (i % Math.max(1, Math.floor(meshX.length / 160)) === 0) {
        rug += `M${px.toFixed(1)},${(baseY + 1).toFixed(1)} L${px.toFixed(1)},${(baseY + 6).toFixed(1)} `;
      }
    }
    area += ` L${xOf(meshX[meshX.length - 1]).toFixed(1)},${y0.toFixed(1)} Z`;
    if (tris) parts.push(tris);
    if (area) parts.push(`<path d="${area}" fill="rgba(122,162,255,0.08)"/>`);
    if (envelope) parts.push(`<path d="${envelope}" fill="none" stroke="rgba(122,162,255,0.9)" stroke-width="1.35"/>`);
    if (rug) parts.push(`<path d="${rug}" fill="none" stroke="rgba(122,162,255,0.7)" stroke-width="1"/>`);
  }

  const dataPath = pathOf(x, data);
  if (dataPath) parts.push(`<path fill="none" stroke="#3ee0c5" stroke-width="1.8" d="${dataPath}"/>`);
  if (model.length === x.length) {
    const modelPath = pathOf(x, model);
    if (modelPath) parts.push(`<path fill="none" stroke="#f2c14e" stroke-width="1.7" stroke-dasharray="4 3" d="${modelPath}"/>`);
  }
  parts.push(`<text x="${left + pw / 2}" y="${height - 8}" fill="var(--muted)" font-size="11" text-anchor="middle">${xName}${log ? " · log" : ""}</text>`);

  host.innerHTML =
    `<svg class="server-job-spark" viewBox="0 0 ${width} ${height}" role="img" aria-label="${jobText("dataFit")}">${parts.join("")}</svg>`;
}

async function embedJobResult(card, reportId, measureKey, unitKey) {
  if (!card?.embed || card.embed.dataset.loaded === reportId) return;
  card.embed.hidden = false;
  card.embed.dataset.loaded = reportId;
  card.embed.innerHTML = `<p class="server-job-embed-note">${jobText("loadingResult")}</p>`;
  try {
    const response = await fetch(`/lab/download/${encodeURIComponent(reportId)}/json`, {
      credentials: "same-origin",
      cache: "no-store",
    });
    if (!response.ok) throw new Error("report unavailable");
    const report = await response.json();
    const chisq = report.chisq;
    const dof = report.chisq_dof;
    const eps = report.eps_inf;
    const points = report.points_used ?? report.data_x?.length;
    const summary = document.createElement("div");
    summary.className = "server-job-embed-summary";
    summary.innerHTML = [
      `<strong>${jobText("result")}</strong>`,
      Number.isFinite(chisq) ? `${jobText("chisq")} ${fmtSci(chisq)}` : "",
      Number.isFinite(dof) ? `χ²/dof ${fmtSci(dof)}` : "",
      Number.isFinite(eps) ? `ε∞ ${fmtSci(eps)}` : "",
      points ? `${jobText("points")} ${points}` : "",
    ].filter(Boolean).join(" · ");
    // Un solo par de gráficas: actualizar las del job; no duplicar en el embed.
    const x = report.data_x || [];
    const data = report.data_y || [];
    const model = report.model_q?.length === x.length ? report.model_q : (report.model_eps2 || []);
    const meshX = report.omega || [];
    const meshY = report.eps2 || [];
    if (card.charts) {
      drawJobSpark(card.charts, {
        x,
        data,
        model,
        mesh_x: meshX,
        mesh_y: meshY,
        chisq,
        resid_rms: Number.isFinite(chisq) && x.length ? Math.sqrt(chisq / Math.max(1, x.length)) : null,
      }, measureKey, unitKey);
    }
    card.embed.replaceChildren(summary);
    if (!card.body.hidden) card.article.classList.add("is-open");
  } catch {
    card.embed.innerHTML = `<p class="server-job-embed-note">${jobText("statusError")}</p>`;
  }
}

function forgetServerJob(id) {
  const jobs = readRecentJobs().filter((job) => job.id !== id);
  try {
    localStorage.setItem(RECENT_SERVER_JOBS_KEY, JSON.stringify(jobs));
  } catch {}
}

function newServerJobId() {
  const bytes = new Uint8Array(16);
  if (globalThis.crypto && typeof globalThis.crypto.getRandomValues === "function") {
    globalThis.crypto.getRandomValues(bytes);
  } else {
    bytes.forEach((_, index) => { bytes[index] = Math.floor(Math.random() * 256); });
  }
  return `job_${Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("")}`;
}

function fillJobConditions(dl, options, points, localMeta) {
  const opts = options || {};
  const measure = opts.measure || localMeta?.measure || "eps2";
  const unit = opts.unit || localMeta?.unit || "otra";
  const rows = [
    [jobText("measure"), MEASURE_LABEL[measure] || measure],
    [jobText("unit"), UNIT_LABEL[unit] || unit],
    [jobText("points"), String(points ?? localMeta?.points ?? "—")],
    [jobText("steps"), String(opts.iter_max ?? localMeta?.iter ?? "—")],
    [jobText("smooth"), String(opts.smoothness ?? localMeta?.smooth ?? "—")],
    [jobText("neg"), String(opts.negativity ?? localMeta?.neg ?? "—")],
    [jobText("eps"), optionLabel(opts, "eps_inf", localMeta?.eps_inf || jobText("auto"))],
    [jobText("drude"), (opts.drude_on ?? localMeta?.drude) ? jobText("on") : jobText("off")],
  ];
  dl.replaceChildren();
  for (const [k, v] of rows) {
    const dt = document.createElement("dt");
    dt.textContent = k;
    const dd = document.createElement("dd");
    dd.textContent = v;
    dl.append(dt, dd);
  }
}

function initServerJobs() {
  if (jobPollingTimer) clearTimeout(jobPollingTimer);
  jobPollingTimer = 0;
  const slot = document.getElementById("lm-jobs");
  if (!slot) return;
  const currentId = new URLSearchParams(window.location.search).get("j");
  if (currentId) rememberServerJob(currentId);
  const jobs = readRecentJobs();
  slot.replaceChildren();
  slot.hidden = jobs.length === 0;
  if (!jobs.length) return;

  const heading = document.createElement("h2");
  heading.textContent = jobText("title");
  const list = document.createElement("div");
  list.className = "server-job-list";
  const cards = new Map();
  for (const job of jobs.slice().reverse()) {
    const article = document.createElement("article");
    article.className = "server-job";
    if (job.id === currentId) article.classList.add("is-current");

    const head = document.createElement("button");
    head.type = "button";
    head.className = "server-job-head";
    head.setAttribute("aria-expanded", "false");

    const label = document.createElement("span");
    label.className = "server-job-id";
    label.textContent = job.id.slice(-8);

    const measureChip = document.createElement("span");
    measureChip.className = "server-job-chip";
    measureChip.textContent = MEASURE_LABEL[job.measure] || job.measure || "…";

    const state = document.createElement("span");
    state.className = "server-job-state";
    state.textContent = jobText("waiting");

    const toggle = document.createElement("span");
    toggle.className = "server-job-toggle";
    toggle.textContent = "▸";

    head.append(label, measureChip, state, toggle);

    const actions = document.createElement("div");
    actions.className = "server-job-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "btn btn-ghost btn-small";
    cancel.textContent = jobText("cancel");
    cancel.hidden = true;
    cancel.addEventListener("click", async (event) => {
      event.stopPropagation();
      cancel.disabled = true;
      cancel.textContent = jobText("cancelling");
      try {
        const response = await fetch(`/lab/job/${encodeURIComponent(job.id)}/cancel`, {
          method: "POST",
          credentials: "same-origin",
          cache: "no-store",
        });
        if (!response.ok && response.status !== 404) throw new Error("cancel failed");
        const status = response.ok ? await response.json() : null;
        card.state.textContent = (status && status.error) || jobText("cancelled");
        cancel.hidden = true;
        card.link.disabled = true;
        delete card.link.dataset.href;
        article.classList.remove("is-running");
        card.bar.style.width = "0%";
      } catch {
        cancel.disabled = false;
        cancel.textContent = jobText("cancel");
        card.state.textContent = jobText("statusError");
      }
    });
    const link = document.createElement("button");
    link.type = "button";
    link.className = "btn btn-ghost btn-small";
    link.textContent = jobText("view");
    link.disabled = true;
    link.addEventListener("click", (event) => {
      event.stopPropagation();
      if (link.dataset.href) window.location.assign(link.dataset.href);
    });
    actions.append(cancel, link);

    const body = document.createElement("div");
    body.className = "server-job-body";
    body.hidden = true;

    const progress = document.createElement("div");
    progress.className = "server-job-progress";
    progress.innerHTML = '<span class="server-job-progress-bar"></span>';
    const bar = progress.querySelector(".server-job-progress-bar");

    const condTitle = document.createElement("p");
    condTitle.className = "server-job-section";
    condTitle.textContent = jobText("conditions");
    const dl = document.createElement("dl");
    dl.className = "server-job-facts";
    fillJobConditions(dl, null, job.points, job);

    const charts = document.createElement("div");
    charts.className = "server-job-charts";
    const localCurve = job.preview ? { x: job.preview.x, data: job.preview.y, model: [], mesh_x: [], mesh_y: [] } : null;
    drawJobSpark(charts, localCurve, job.measure, job.unit);
    const embed = document.createElement("div");
    embed.className = "server-job-embed";
    embed.hidden = true;

    body.append(progress, condTitle, dl, charts, embed);
    article.append(head, actions, body);
    list.append(article);

    head.addEventListener("click", () => {
      const open = !article.classList.contains("is-open");
      article.classList.toggle("is-open", open);
      body.hidden = !open;
      head.setAttribute("aria-expanded", open ? "true" : "false");
      toggle.textContent = open ? "▾" : "▸";
    });

    const card = { article, state, link, cancel, bar, dl, charts, embed, measureChip, toggle, body, head };
    cards.set(job.id, card);
  }
  slot.append(heading, list);

  const refresh = async () => {
    let pending = false;
    for (const job of jobs) {
      const card = cards.get(job.id);
      if (!card) continue;
      try {
        const response = await fetch(`/lab/job/${encodeURIComponent(job.id)}`, {
          credentials: "same-origin",
          cache: "no-store",
        });
        if (response.status === 404) {
          forgetServerJob(job.id);
          card.state.textContent = jobText("statusError");
          card.cancel.hidden = true;
          continue;
        }
        if (!response.ok) throw new Error("job status unavailable");
        const status = await response.json();
        const opts = status.options || {};
        card.measureChip.textContent = MEASURE_LABEL[opts.measure] || opts.measure || card.measureChip.textContent;
        fillJobConditions(card.dl, opts, status.points ?? job.points, job);
        const curve = status.curve || (job.preview ? { x: job.preview.x, data: job.preview.y, model: [], mesh_x: [], mesh_y: [] } : null);
        if (status.state !== "completed") {
          drawJobSpark(card.charts, curve, opts.measure || job.measure, opts.unit || job.unit);
        }

        if (status.state === "queued" || status.state === "running") {
          pending = true;
          card.article.classList.add("is-running");
          let label = jobText(status.state);
          const step = Number(status.progress_step) || 0;
          const max = Number(status.progress_max) || 0;
          if (status.state === "running" && max > 0) {
            label += " · " + jobText("step").replace("{step}", String(step)).replace("{max}", String(max));
            card.bar.style.width = `${Math.max(2, Math.min(100, (100 * step) / max)).toFixed(1)}%`;
          } else {
            card.bar.style.width = status.state === "queued" ? "4%" : "8%";
          }
          if (status.progress_note) label += " · " + status.progress_note;
          if (Number.isFinite(curve?.chisq)) label += ` · ${jobText("chisq")} ${fmtSci(curve.chisq)}`;
          if (status.state === "running") label = `${jobText("live")} · ${label}`;
          card.state.textContent = label;
          card.cancel.hidden = false;
          card.cancel.disabled = false;
          card.cancel.textContent = jobText("cancel");
          card.link.disabled = true;
          delete card.link.dataset.href;
        } else if (status.state === "completed" && status.report_id) {
          card.article.classList.remove("is-running");
          card.bar.style.width = "100%";
          card.state.textContent = jobText("completed");
          card.cancel.hidden = true;
          card.link.dataset.href = `/lab?r=${encodeURIComponent(status.report_id)}`;
          card.link.disabled = false;
          void embedJobResult(card, status.report_id, opts.measure || job.measure, opts.unit || job.unit);
        } else if (status.state === "cancelled") {
          card.article.classList.remove("is-running");
          card.bar.style.width = "0%";
          card.state.textContent = status.error || jobText("cancelled");
          card.cancel.hidden = true;
          card.link.disabled = true;
          delete card.link.dataset.href;
        } else {
          card.article.classList.remove("is-running");
          card.bar.style.width = "0%";
          card.state.textContent = status.error || jobText("failed");
          card.cancel.hidden = true;
          card.link.disabled = true;
          delete card.link.dataset.href;
        }
      } catch {
        pending = true;
        card.state.textContent = jobText("statusError");
        card.cancel.hidden = true;
      }
    }
    if (pending) jobPollingTimer = window.setTimeout(refresh, 2000);
  };
  void refresh();
}

function callFitWorker(payload) {
  if (!fitWorker) {
    fitWorker = new Worker("/js/fit-worker.js?v=3", { type: "module" });
    fitWorker.addEventListener("message", (event) => {
      const request = fitRequests.get(event.data.id);
      if (!request) return;
      fitRequests.delete(event.data.id);
      if (event.data.error) request.reject(new Error(event.data.error));
      else request.resolve(event.data.result);
    });
    fitWorker.addEventListener("error", () => {
      const error = new Error("Web Worker could not run the local fit.");
      fitWorker.terminate();
      fitWorker = null;
      for (const request of fitRequests.values()) request.reject(error);
      fitRequests.clear();
    });
  }

  const id = ++fitRequestId;
  return new Promise((resolve, reject) => {
    fitRequests.set(id, { resolve, reject });
    try {
      fitWorker.postMessage({ id, ...payload });
    } catch (error) {
      fitRequests.delete(id);
      reject(error);
    }
  });
}

function runFitInWorker(text, options) {
  return callFitWorker({ text, options });
}

function mergeInWorker(parts) {
  return callFitWorker({ op: "merge", parts }).then((raw) => JSON.parse(raw));
}

function initLabSubmission() {
  const form = document.querySelector("form[action='/lab/run']");
  const live = document.getElementById("lm-live");
  if (!form) return;
  form.addEventListener("submit", async (event) => {
    if (event.submitter?.name === "execution" && event.submitter.value === "server") {
      event.preventDefault();
      const box = form.querySelector("textarea[name='spectrum']");
      const points = parseColumns(box ? box.value : "").xs.length;
      if (points > SERVER_MAX_POINTS) {
        if (live) {
          live.hidden = false;
          live.classList.remove("is-idle");
          live.innerHTML = "";
          const error = document.createElement("p");
          error.className = "form-error";
          error.setAttribute("role", "alert");
          error.textContent = uiText("serverLimit").replace("{count}", String(points));
          live.append(error);
          live.scrollIntoView({ behavior: "smooth", block: "start" });
        }
        return;
      }
      const submitter = event.submitter;
      const buttons = [...form.querySelectorAll("button[type='submit']")];
      const labels = buttons.map((button) => button.textContent);
      buttons.forEach((button) => { button.disabled = true; });
      if (submitter) submitter.textContent = uiText("serverWorking");
      form.setAttribute("aria-busy", "true");
      showLiveMessage(live, uiText("serverWorking"));
      live?.scrollIntoView({ behavior: "smooth", block: "start" });
      const data = new FormData(form);
      if (submitter?.name) data.append(submitter.name, submitter.value);
      const jobId = newServerJobId();
      const spectrumText = String(data.get("spectrum") || "");
      rememberServerJob(jobId, jobMetaFromForm(form, spectrumText));
      data.append("job_id", jobId);
      try {
        const response = await fetch(submitter?.formAction || form.action, {
          method: "POST",
          body: data,
          credentials: "same-origin",
        });
        if (!response.ok) throw new Error(`Server returned ${response.status}`);
        const destination = new URL(response.url, window.location.href);
        if (destination.origin !== window.location.origin) throw new Error("Unexpected redirect origin");
        if (destination.searchParams.get("j") !== jobId) forgetServerJob(jobId);
        window.location.assign(destination.href);
      } catch {
        form.removeAttribute("aria-busy");
        buttons.forEach((button, index) => {
          button.disabled = false;
          button.textContent = labels[index];
        });
        showLiveMessage(live, jobText("network"), "form-error");
      }
      return;
    }
    event.preventDefault();
    const btn = event.submitter || form.querySelector("button[type='submit']");
    const label = btn ? btn.textContent : "";
    if (btn) {
      btn.disabled = true;
      btn.textContent = uiText("working");
    }
    const restore = () => {
      if (!btn) return;
      btn.disabled = false;
      btn.textContent = label;
    };
    if (!live) {
      restore();
      return;
    }
    const data = new FormData(form);
    let text = String(data.get("spectrum") || "");
    const file = data.get("file");
    if (file instanceof File && file.size > 0) {
      text = await file.text();
    }
    const opts = {
      iter: Number(data.get("iter") || 20),
      log_mesh: data.get("log_mesh") === "1",
      extended: data.get("extended") === "1",
      drude: data.get("drude") === "1",
      eps_inf: String(data.get("eps_inf") || ""),
      omega_p: String(data.get("omega_p") || ""),
      gamma: String(data.get("gamma") || ""),
      smooth: Number(data.get("smooth") ?? 0),
      neg: Number(data.get("neg") ?? 100),
      measure: String(data.get("measure") || "eps2"),
      unit: String(data.get("unit") || "otra"),
      thickness: String(data.get("thickness") || ""),
      temperature: String(data.get("temperature") || ""),
      mesh: String(data.get("mesh") || "auto"),
      solver: String(data.get("solver") || "auto"),
      edges: String(data.get("edges") || "auto"),
      lang: pageLang(),
    };
    const pointCount = text ? text.split(/\r?\n/).length : 0;
    const estimatedWork = (pointCount + 1) ** 3 * opts.iter;
    const isDense = opts.measure !== "eps2" && opts.measure !== "lum";
    const status = isDense && estimatedWork > LOCAL_SLOW_WORK ? uiText("localSlow") : uiText("local");
    if (btn) btn.textContent = uiText("local");
    showLiveMessage(live, status);
    await new Promise((resolve) => requestAnimationFrame(resolve));
    let result;
    try {
      result = JSON.parse(await runFitInWorker(text, opts));
    } catch {
      live.textContent = uiText("engine");
      restore();
      return;
    }
    if (!result.ok) {
      live.innerHTML = "<p class=\"form-error\" role=\"alert\"></p>";
      live.querySelector("p").textContent = result.error || uiText("fail");
      restore();
      return;
    }
    live.innerHTML = result.html;
    const ext = { dat: "dat", csv: "csv", ajuste: "ajuste.csv", picos: "picos.csv", json: "json", informe: "informe.html" };
    const mime = { json: "application/json;charset=utf-8", dat: "text/plain;charset=utf-8", informe: "text/html;charset=utf-8" };
    live.querySelectorAll("[data-lm-file]").forEach((link) => {
      const key = link.getAttribute("data-lm-file");
      const body = result.files && result.files[key];
      if (!body) {
        link.remove();
        return;
      }
      link.href = URL.createObjectURL(new Blob([body], { type: mime[key] || "text/csv;charset=utf-8" }));
      link.download = `lightmatter.${ext[key] || key}`;
    });
    enhanceCharts(live);
    restore();
    live.scrollIntoView({ behavior: "smooth", block: "start" });
  });
}

function initLabPage() {
  enhanceCharts(document);
  initLabControls();
  initLabSubmission();
  initServerJobs();
}

document.addEventListener("resuma:navigate", initLabPage);
initLabPage();

