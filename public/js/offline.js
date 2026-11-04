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

const drop = document.querySelector("[data-lm-drop]");
if (drop) {
  const input = drop.querySelector("input[type='file']");
  const name = drop.querySelector("[data-lm-file]");
  const idle = name ? name.textContent : "";
  const show = () => {
    const file = input && input.files && input.files[0];
    drop.classList.toggle("has-file", Boolean(file));
    if (!name) return;
    name.textContent = file ? `${file.name} · ${(file.size / 1024).toFixed(1)} kB · este archivo manda` : idle;
  };
  if (input) input.addEventListener("change", show);
  ["dragenter", "dragover"].forEach((type) => drop.addEventListener(type, () => drop.classList.add("is-over")));
  ["dragleave", "drop"].forEach((type) => drop.addEventListener(type, () => drop.classList.remove("is-over")));
  if (form) form.addEventListener("reset", () => setTimeout(show));
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
  svgBtn.title = "Vectorial, editable en Inkscape o Illustrator";
  svgBtn.addEventListener("click", () => save(new Blob([serialize()], { type: "image/svg+xml;charset=utf-8" }), `lightmatter-${slug}.svg`));
  const pngBtn = document.createElement("button");
  pngBtn.type = "button";
  pngBtn.textContent = "PNG";
  pngBtn.title = "Imagen a doble resolución";
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
  label.textContent = "Bajar gráfica:";
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
    if (navigator.onLine) return;
    event.preventDefault();
    if (!live) return;
    if (!adjust) {
      live.innerHTML = "<p class=\"form-error\" role=\"alert\">Abre la app una vez con red para guardar el motor Rust en el aparato.</p>";
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
    };
    live.innerHTML = "<p class=\"muted\">Calculando en este aparato…</p>";
    await new Promise((r) => setTimeout(r, 30));
    let result;
    try {
      result = JSON.parse(adjust(text, JSON.stringify(opts)));
    } catch (err) {
      live.textContent = "El motor Rust no pudo leer ese espectro.";
      return;
    }
    if (!result.ok) {
      live.innerHTML = "<p class=\"form-error\" role=\"alert\"></p>";
      live.querySelector("p").textContent = result.error || "No se pudo ajustar.";
      return;
    }
    live.innerHTML = result.html;
    const ext = { dat: "dat", csv: "csv", ajuste: "ajuste.csv", picos: "picos.csv", json: "json" };
    const mime = { json: "application/json;charset=utf-8", dat: "text/plain;charset=utf-8" };
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
