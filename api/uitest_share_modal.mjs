// Prüft den Teilen-Dialog aus Nutzersicht: Der Kurz-Link muss die Vorgabe
// sein und ohne Zusatzklick erscheinen, der volle Link muss erreichbar
// bleiben.
//
// Kein cargo-Test, weil dafür ein laufender dx-Server und ein WebDriver nötig
// sind (siehe `make e2e`). Aufruf:
//   dx serve --port 8080 &
//   chromedriver --port=4444 &
//   node api/uitest_share_modal.mjs [app-url]
//
// Lokal liefert die Rückfallebene (tinyurl/da.gd), weil `localhost` kein
// erlaubtes Ziel des eigenen Endpunkts ist. Der Test prüft deshalb das
// Verhalten, nicht den konkreten Anbieter.
// Der Teilen-Knopf erscheint nur, wenn Daten eingegeben sind (query_string).
// Deshalb direkt mit einem befüllten Rezept starten.
const BASE =
  process.argv[2] ??
  "http://localhost:8080/open-farming-hackdays-label-creator/lebensmittelrecht?v=2&product_title=Brownies&producer_name=Anna";
const WD = "http://localhost:4444";

let failures = 0;
const check = (name, ok, detail = "") => {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${ok ? "" : `  -> ${detail}`}`);
  if (!ok) failures++;
};

const post = async (path, body) => {
  const r = await fetch(`${WD}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body ?? {}),
  });
  return await r.json();
};
const get = async (path) => await (await fetch(`${WD}${path}`)).json();

const session = await post("/session", {
  capabilities: {
    alwaysMatch: {
      browserName: "chrome",
      "goog:chromeOptions": {
        // Ohne feste Fenstergrösse: die App rendert je nach Breite ein
        // anderes Layout, und in der breiten Variante zeigte der Klick auf
        // den ersten Treffer ins Leere.
        args: ["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
      },
    },
  },
});
const sid = session.value.sessionId;
const S = `/session/${sid}`;
const script = async (code, args = []) =>
  (await post(`${S}/execute/sync`, { script: code, args })).value;

try {
  // Der Teilen-Knopf ist auf den akzeptierten Haftungsausschluss gated
  // (layout.rs liest ihn aus localStorage). Wie in tests/common/mod.rs:
  // Flag vor dem Laden setzen, sonst bleibt der Knopf wirkungslos.
  // Erst die App-Wurzel laden (nicht nur den Origin): dort läuft das WASM an,
  // und localStorage gilt anschliessend für dieselbe Herkunft.
  await post(`${S}/url`, { url: BASE.split("lebensmittelrecht")[0] });
  await script("localStorage.setItem('disclaimer_accepted','true'); return null;");
  await post(`${S}/url`, { url: BASE });
  // Auf das WASM warten: die App rendert erst nach dem Laden. Gewartet wird
  // gezielt auf den Teilen-Knopf, der zusätzlich eingegebene Daten voraussetzt.
  for (let i = 0; i < 60; i++) {
    const ready = await script(
      "return [...document.querySelectorAll('button')].some(b=>/link kopieren|copier le lien|copia link/i.test(b.textContent||''))"
    );
    if (ready) break;
    await new Promise((r) => setTimeout(r, 500));
  }

  // Teilen-Knopf finden und öffnen.
  const opened = await script(`
    const b=[...document.querySelectorAll('button')].find(x=>/link kopieren|copier le lien|copia link/i.test(x.textContent||'') && !x.disabled);
    if(!b) return 'kein Teilen-Knopf';
    b.click(); return 'ok';`);
  check("Teilen-Dialog geöffnet", opened === "ok", opened);
  await new Promise((r) => setTimeout(r, 1500));

  // Der Kurz-Link wird beim Öffnen automatisch geholt.
  let state = null;
  for (let i = 0; i < 40; i++) {
    const snapshot = await script(`
      const d=[...document.querySelectorAll('dialog[open]')].find(x=>/Link teilen|Partager le lien|Condividi link/i.test(x.textContent||'')); if(!d) return null;
      const radios=[...d.querySelectorAll('input[type=radio]')];
      const labels=radios.map(r=>r.closest('label')?.textContent.trim());
      const input=d.querySelector('input[type=text]');
      return {
        radios: radios.map(r=>r.checked),
        labels,
        value: input ? input.value : null,
        text: (d.textContent||''),
        buttons: [...d.querySelectorAll('button')].map(b=>b.textContent.trim()),
      };`);
    if (snapshot) state = snapshot;
    if (state && state.value) break;
    await new Promise((r) => setTimeout(r, 500));
  }

  check("Dialog sichtbar", !!state, "kein offener Dialog");
  if (state) {
    check("zwei Auswahlmöglichkeiten", state.radios.length === 2, JSON.stringify(state.labels));
    check("Kurz-Link ist vorausgewählt", state.radios[1] === true, JSON.stringify(state.radios));
    // Lokal ist `localhost` kein erlaubtes Ziel für den eigenen Endpunkt
    // (ALLOWED_TARGET_HOSTS), deshalb liefert hier die Rückfallebene. Geprüft
    // wird darum nur, dass überhaupt ohne Zusatzklick ein kurzer Link kommt.
    check(
      "Kurz-Link erscheint ohne Zusatzklick",
      typeof state.value === "string" && state.value.length < 60 && state.value.startsWith("https://"),
      state.value
    );
    check(
      "kein 'Link kürzen'-Knopf mehr",
      !state.buttons.some((b) => /kürzen|raccourcir|accorcia/i.test(b)),
      JSON.stringify(state.buttons)
    );
    // Der Hinweis muss zum tatsächlichen Anbieter passen: declarino.ch nur
    // dann versprechen, wenn der Link auch dort liegt. Genau diese Lüge war
    // der Anlass für den Umbau.
    const viaOwn = state.value.includes("declarino.ch/s/");
    check(
      "Hinweis passt zum tatsächlichen Anbieter",
      viaOwn
        ? /declarino\.ch/.test(state.text) && !/Drittanbieter/i.test(state.text)
        : /Drittanbieter/i.test(state.text),
      state.text.slice(0, 170)
    );
  }

  // Umschalten auf den vollen Link.
  const full = await script(`
    const d=[...document.querySelectorAll('dialog[open]')].find(x=>/Link teilen|Partager|Condividi/i.test(x.textContent||''));
    [...d.querySelectorAll('input[type=radio]')][0].click(); return true;`);
  await new Promise((r) => setTimeout(r, 600));
  const fullState = await script(`
    const d=[...document.querySelectorAll('dialog[open]')].find(x=>/Link teilen|Partager|Condividi/i.test(x.textContent||''));
    const i=d.querySelector('input[type=text]'); return i ? i.value : null;`);
  check(
    "voller Link weiterhin wählbar",
    typeof fullState === "string" && fullState.length > 200 && !fullState.includes("/s/"),
    `${String(fullState).slice(0, 80)} (${String(fullState).length} Zeichen)`
  );

  // Zurück auf Kurz-Link: der bereits geholte Link muss erhalten bleiben.
  await script(`
    const d=[...document.querySelectorAll('dialog[open]')].find(x=>/Link teilen|Partager|Condividi/i.test(x.textContent||''));
    [...d.querySelectorAll('input[type=radio]')][1].click(); return true;`);
  await new Promise((r) => setTimeout(r, 600));
  const backShort = await script(`
    const d=[...document.querySelectorAll('dialog[open]')].find(x=>/Link teilen|Partager|Condividi/i.test(x.textContent||''));
    const i=d.querySelector('input[type=text]'); return i ? i.value : null;`);
  check(
    "Zurückschalten zeigt denselben Kurz-Link",
    backShort === state.value,
    `${backShort} vs ${state.value}`
  );
} finally {
  await fetch(`${WD}${S}`, { method: "DELETE" });
}

console.log(failures === 0 ? "\nOberfläche verhält sich wie gewünscht." : `\n${failures} Prüfung(en) fehlgeschlagen.`);
process.exit(failures === 0 ? 0 : 1);
