const installBtn = document.querySelector(".install-app");
const installHint = document.querySelector("[data-install-hint]");
const installHelp = {
  es: "Abre el menú del navegador y elige «Instalar aplicación» o «Añadir a pantalla de inicio».",
  en: "Open your browser menu and choose “Install app” or “Add to Home Screen”.",
  zh: "打开浏览器菜单，选择“安装应用”或“添加到主屏幕”。",
  pt: "Abra o menu do navegador e escolha “Instalar aplicativo” ou “Adicionar à tela inicial”.",
};
const installLang = (document.documentElement.lang || "es").slice(0, 2).toLowerCase();
const manualInstallHelp = installHelp[installLang] || installHelp.es;
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
      if (!promptEvent) {
        if (installHint) {
          installHint.textContent = manualInstallHelp;
          installHint.hidden = false;
        }
        return;
      }
      try {
        await promptEvent.prompt();
        const choice = await promptEvent.userChoice;
        if (choice.outcome === "accepted") installBtn.hidden = true;
        else if (installHint) {
          installHint.textContent = manualInstallHelp;
          installHint.hidden = false;
        }
      } catch {
        /* el navegador cerró el cuadro */
      }
      promptEvent = null;
    });
    window.addEventListener("appinstalled", () => {
      installBtn.hidden = true;
    });
  }
}
