document.addEventListener("change", (event) => {
  const el = event.target;
  if (!(el instanceof HTMLSelectElement) || !el.closest(".lang-switch")) return;
  if (el.value) window.location.assign(el.value);
});

const installBtn = document.querySelector(".install-app");
if (installBtn) {
  const standalone =
    window.matchMedia("(display-mode: standalone)").matches ||
    window.navigator.standalone === true;
  if (standalone) {
    installBtn.remove();
  } else {
    let promptEvent = null;
    window.addEventListener("beforeinstallprompt", (event) => {
      event.preventDefault();
      promptEvent = event;
      installBtn.hidden = false;
    });
    installBtn.addEventListener("click", async () => {
      if (!promptEvent) return;
      promptEvent.prompt();
      try {
        await promptEvent.userChoice;
      } catch {
        /* el navegador cerró el cuadro */
      }
      promptEvent = null;
      installBtn.hidden = true;
    });
    window.addEventListener("appinstalled", () => {
      installBtn.hidden = true;
    });
  }
}
