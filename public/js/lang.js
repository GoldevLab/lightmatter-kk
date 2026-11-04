document.addEventListener("change", (event) => {
  const el = event.target;
  if (!(el instanceof HTMLSelectElement) || !el.closest(".lang-switch")) return;
  if (el.value) window.location.assign(el.value);
});
