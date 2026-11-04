"use strict";

const form = document.querySelector("form[action='/lab/run']");
const live = document.getElementById("lm-live");

let adjust = null;
import("/images/lightmatter.js")
  .then((mod) => mod.default())
  .then(() => import("/images/lightmatter.js"))
  .then((mod) => {
    adjust = mod.adjust;
  })
  .catch(() => {
    adjust = null;
  });

const MEASURE_LABEL = { eps2: "ε₂", eps1: "ε₁", n: "n", k: "k", r: "R", alpha: "α", t: "T", lum: "I" };
const UNIT_LABEL = { cm: "cm⁻¹", ev: "eV", nm: "nm", otra: "ω" };

function pageLang() {
  const tag = (document.documentElement.lang || "es").toLowerCase();
  if (tag.startsWith("zh")) return "zh";
  if (tag.startsWith("pt")) return "pt";
  if (tag.startsWith("en")) return "en";
  return "es";
}

function uiText(key) {
  const pack = {
    es: { chart: "Gráfica", empty: "La curva de lo que cargues aparece aquí, sin esperar al ajuste.", needRow: "Hace falta otra fila numérica.", tooBig: "El archivo supera 4 MB.", unread: "No pude leer ese archivo.", working: "Calculando…", local: "Calculando en este aparato…", needNet: "Abre la app una vez con red para guardar el motor Rust en el aparato.", engine: "El motor Rust no pudo leer ese espectro.", fail: "No se pudo ajustar.", save: "Bajar gráfica:", svg: "Vectorial, editable en Inkscape o Illustrator", png: "Imagen a doble resolución" },
    en: { chart: "Plot", empty: "The curve of what you load appears here, before the fit.", needRow: "Another numeric row is needed.", tooBig: "The file is over 4 MB.", unread: "That file could not be read.", working: "Fitting…", local: "Fitting on this device…", needNet: "Open the app once online so the Rust engine is saved on this device.", engine: "The Rust engine could not read that spectrum.", fail: "The fit did not run.", save: "Save plot:", svg: "Vector file, editable in Inkscape or Illustrator", png: "Image at double resolution" },
    zh: { chart: "图", empty: "载入的曲线会出现在这里，不必等待拟合。", needRow: "还需要一行数字。", tooBig: "文件超过 4 MB。", unread: "无法读取这个文件。", working: "正在拟合…", local: "正在本机拟合…", needNet: "请先联网打开一次，以便把 Rust 引擎存到这台设备。", engine: "Rust 引擎无法读取这份光谱。", fail: "未能完成拟合。", save: "保存图：", svg: "矢量图，可在 Inkscape 或 Illustrator 中编辑", png: "双倍分辨率图像" },
    pt: { chart: "Gráfico", empty: "A curva do que você carregar aparece aqui, sem esperar o ajuste.", needRow: "Falta outra linha numérica.", tooBig: "O arquivo passa de 4 MB.", unread: "Não consegui ler esse arquivo.", working: "Ajustando…", local: "Ajustando neste aparelho…", needNet: "Abra o app uma vez com rede para guardar o motor Rust neste aparelho.", engine: "O motor Rust não conseguiu ler esse espectro.", fail: "Não foi possível ajustar.", save: "Baixar gráfico:", svg: "Vetorial, editável no Inkscape ou no Illustrator", png: "Imagem em dobro da resolução" },
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
    `<rect width="720" height="340" fill="#07141b"/>`,
    `<line x1="${left}" y1="${top}" x2="${left}" y2="${top + ph}" stroke="rgba(143,184,176,0.35)"/>`,
    `<line x1="${left}" y1="${top + ph}" x2="${left + pw}" y2="${top + ph}" stroke="rgba(143,184,176,0.35)"/>`,
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
    parts.push(`<line x1="${left}" y1="${py.toFixed(1)}" x2="${left + pw}" y2="${py.toFixed(1)}" stroke="rgba(143,184,176,0.12)"/>`);
    parts.push(`<text x="${left - 8}" y="${(py + 4).toFixed(1)}" fill="#8fb8b0" font-size="11" text-anchor="end" font-family="ui-monospace,monospace">${fmt(v)}</text>`);
  }
  for (const v of xTicks) {
    if (!(v > 0) && log) continue;
    const px = xOf(v);
    if (px < left - 1 || px > left + pw + 1) continue;
    parts.push(`<text x="${px.toFixed(1)}" y="${top + ph + 18}" fill="#8fb8b0" font-size="11" text-anchor="middle" font-family="ui-monospace,monospace">${fmt(v)}</text>`);
  }
  parts.push(`<text x="${left + pw / 2}" y="${height - 8}" fill="#8fb8b0" font-size="12" text-anchor="middle">${xName}</text>`);
  parts.push(`<text x="16" y="${top + ph / 2}" fill="#8fb8b0" font-size="12" text-anchor="middle" transform="rotate(-90 16 ${top + ph / 2})">${yName}</text>`);
  parts.push(`<path data-series="datos" fill="none" stroke="#9aa7b2" stroke-width="1.6" d="${d}"/>`);
  slot.innerHTML =
    `<svg viewBox="0 0 ${width} ${height}" role="img" aria-label="Datos cargados" data-xmin="${xmin}" data-xmax="${xmax}" data-log="${log ? 1 : 0}" data-ymin="${ymin}" data-ymax="${ymax}" data-left="${left}" data-top="${top}" data-pw="${pw}" data-ph="${ph}" data-xlabel="${xName}">${parts.join("")}</svg>`;
  if (meta) {
    meta.textContent = `${xs.length} puntos · ${xName} ${fmt(sx[0])} … ${fmt(sx[sx.length - 1])} · ${yName} ${fmt(yRawLo)} … ${fmt(yRawHi)}${log ? " · eje log" : ""}`;
  }
  enhanceChart(figure);
}

const drop = document.querySelector("[data-lm-drop]");
if (drop && form) {
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
    input.addEventListener("change", async () => {
      const file = input.files && input.files[0];
      if (!file) return;
      if (file.size > 4_000_000) {
        if (meta) meta.textContent = uiText("tooBig");
        input.value = "";
        return;
      }
      let text = "";
      try {
        text = await file.text();
      } catch {
        if (meta) meta.textContent = uiText("unread");
        return;
      }
      if (box) box.value = text;
      drop.classList.add("has-file");
      if (name) name.textContent = `${file.name} · ${(file.size / 1024).toFixed(1)} kB · ya está en el cuadro`;
      input.value = "";
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
}

const measure = form ? form.querySelector("select[name='measure']") : null;
if (measure) {
  const fields = form.querySelectorAll("[data-lm-for], [data-lm-unless]");
  const sync = () => {
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
  const toX = (px) => (log ? Math.exp(Math.log(xmin) + ((px - left) / pw) * (Math.log(xmax) - Math.log(xmin))) : xmin + ((px - left) / pw) * (xmax - xmin));
  const toY = (py) => ymin + (1 - (py - top) / ph) * (ymax - ymin);
  const series = Array.from(svg.querySelectorAll("path[data-series]")).map((p) => ({
    name: p.dataset.series,
    color: p.getAttribute("stroke"),
    pts: parsePath(p.getAttribute("d") || ""),
  })).filter((s) => s.pts.length);
  if (!series.length) return;

  const ns = "http://www.w3.org/2000/svg";
  const cursor = document.createElementNS(ns, "line");
  cursor.setAttribute("stroke", "rgba(231,246,242,0.45)");
  cursor.setAttribute("stroke-dasharray", "3 3");
  cursor.setAttribute("y1", top);
  cursor.setAttribute("y2", top + ph);
  cursor.style.display = "none";
  svg.appendChild(cursor);
  const dots = series.map((s) => {
    const c = document.createElementNS(ns, "circle");
    c.setAttribute("r", "3.5");
    c.setAttribute("fill", s.color);
    c.setAttribute("stroke", "#07141b");
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
    const rect = svg.getBoundingClientRect();
    const px = ((clientX - rect.left) / rect.width) * 720;
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
      ctx.fillStyle = "#07141b";
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
      URL.revokeObjectURL(url);
      canvas.toBlob((blob) => blob && save(blob, `lightmatter-${slug}.png`), "image/png");
    };
    img.src = url;
  });
  const label = document.createElement("span");
  label.textContent = uiText("save");
  tools.append(label, svgBtn, pngBtn);
  figure.appendChild(tools);
}

function enhanceCharts(root) {
  (root || document).querySelectorAll("figure.chart").forEach(enhanceChart);
}
enhanceCharts(document);

// ---- Sin red: el motor Rust corre en el aparato ----

if (form) {
  form.addEventListener("submit", async (event) => {
    if (navigator.onLine) {
      const btn = form.querySelector("button[type='submit']");
      if (btn) {
        btn.disabled = true;
        btn.textContent = uiText("working");
      }
      return;
    }
    if (live) live.hidden = false;
    event.preventDefault();
    if (!live) return;
    if (!adjust) {
      live.innerHTML = `<p class="form-error" role="alert">${uiText("needNet")}</p>`;
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
      smooth: Number(data.get("smooth") ?? 0.08),
      neg: Number(data.get("neg") ?? 100),
      measure: String(data.get("measure") || "eps2"),
      unit: String(data.get("unit") || "otra"),
      thickness: String(data.get("thickness") || ""),
      temperature: String(data.get("temperature") || ""),
      lang: pageLang(),
    };
    live.innerHTML = `<p class="muted">${uiText("local")}</p>`;
    await new Promise((r) => setTimeout(r, 30));
    let result;
    try {
      result = JSON.parse(adjust(text, JSON.stringify(opts)));
    } catch (err) {
      live.textContent = uiText("engine");
      return;
    }
    if (!result.ok) {
      live.innerHTML = "<p class=\"form-error\" role=\"alert\"></p>";
      live.querySelector("p").textContent = result.error || uiText("fail");
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
    live.scrollIntoView({ behavior: "smooth", block: "start" });
  });
}
