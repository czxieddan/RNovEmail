use std::fmt::Write;

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{
    AdminData, AdminSection, Lang, LoginScopeView, PageContext, PortalData, Text, Theme, text,
};

const ADMIN_CSS: &str = r#"
:root {
  color-scheme: light;
  --bg: #f8fafc;
  --panel: #ffffff;
  --ink: #1e293b;
  --muted: #475569;
  --line: #e2e8f0;
  --blue: #2563eb;
  --blue-soft: #eff6ff;
  --orange: #f97316;
  --danger: #b91c1c;
  --ok: #047857;
  --shadow: 0 18px 45px rgba(15, 23, 42, 0.08);
}
[data-theme="dark"] {
  color-scheme: dark;
  --bg: #0b1020;
  --panel: #111827;
  --ink: #f8fafc;
  --muted: #cbd5e1;
  --line: #334155;
  --blue: #60a5fa;
  --blue-soft: #172554;
  --orange: #fb923c;
  --danger: #fca5a5;
  --ok: #86efac;
  --shadow: none;
}
* { box-sizing: border-box; }
[hidden] { display: none !important; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--ink);
  font: 14px/1.5 Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
a { color: inherit; text-decoration: none; }
.skip {
  position: absolute;
  left: 16px;
  top: -48px;
  background: var(--ink);
  color: var(--bg);
  padding: 8px 12px;
  border-radius: 6px;
  z-index: 10;
}
.skip:focus { top: 16px; }
.shell {
  display: grid;
  grid-template-columns: 264px minmax(0, 1fr);
  min-height: 100vh;
}
.side {
  background: var(--panel);
  border-right: 1px solid var(--line);
  padding: 28px 20px;
}
.brand {
  align-items: center;
  display: flex;
  font-weight: 800;
  gap: 10px;
  letter-spacing: 0;
}
.mark {
  border-radius: 6px;
  flex: 0 0 auto;
  height: 24px;
  width: 24px;
}
span.mark {
  background: var(--blue);
  color: #fff;
  display: inline-grid;
  place-items: center;
}
.logo-mark {
  display: block;
  overflow: visible;
}
.nav {
  display: grid;
  gap: 8px;
  margin-top: 28px;
}
.nav a {
  border-radius: 8px;
  color: var(--muted);
  padding: 10px 12px;
}
.nav a:hover,
.nav a:focus {
  background: var(--blue-soft);
  color: var(--blue);
  outline: none;
}
.main {
  min-width: 0;
  padding: 32px;
}
.topbar {
  align-items: end;
  display: flex;
  gap: 18px;
  justify-content: space-between;
  margin-bottom: 24px;
}
.actions {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}
.settings {
  position: relative;
}
.settings summary {
  list-style: none;
}
.settings summary::-webkit-details-marker {
  display: none;
}
.settings-menu {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: var(--shadow);
  display: grid;
  gap: 8px;
  min-width: 180px;
  padding: 10px;
  position: absolute;
  right: 0;
  top: calc(100% + 8px);
  z-index: 20;
}
.settings-menu .button,
.settings-menu button {
  justify-content: flex-start;
  width: 100%;
}
.eyebrow {
  color: var(--blue);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0;
  margin: 0 0 4px;
  text-transform: uppercase;
}
h1 {
  font-size: 28px;
  line-height: 1.15;
  margin: 0;
}
h2 {
  font-size: 17px;
  line-height: 1.25;
  margin: 0;
}
p {
  color: var(--muted);
  margin: 8px 0 0;
}
label {
  color: var(--ink);
  display: grid;
  font-weight: 650;
  gap: 6px;
}
input,
select,
textarea {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--ink);
  font: inherit;
  min-height: 40px;
  padding: 9px 11px;
  width: 100%;
}
input:focus,
select:focus,
textarea:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.16);
  outline: none;
}
textarea {
  min-height: 112px;
  resize: vertical;
}
button,
.button {
  align-items: center;
  background: var(--blue);
  border: 0;
  border-radius: 8px;
  color: #fff;
  cursor: pointer;
  display: inline-flex;
  font: inherit;
  font-weight: 700;
  justify-content: center;
  min-height: 40px;
  padding: 9px 14px;
  transition: background 160ms ease, box-shadow 160ms ease;
}
button:hover,
.button:hover,
button:focus,
.button:focus {
  box-shadow: 0 10px 24px rgba(37, 99, 235, 0.2);
  outline: none;
}
.button.secondary,
button.secondary {
  background: transparent;
  border: 1px solid var(--line);
  color: var(--ink);
}
.grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}
.panel {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: var(--shadow);
  padding: 18px;
}
.panel.accent { border-top: 3px solid var(--orange); }
.mail-toolbar {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}
.mail-toolbar p {
  margin: 0;
}
.compose-panel {
  display: none;
}
.compose-panel:target {
  display: block;
}
.compose-heading {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}
.message-subject {
  display: grid;
  gap: 4px;
}
.message-snippet {
  margin: 0;
  max-width: 54rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mail-app {
  min-height: 100vh;
}
.mail-topbar {
  align-items: center;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
  display: grid;
  gap: 18px;
  grid-template-columns: auto minmax(240px, 560px) auto auto;
  min-height: 56px;
  padding: 0 16px;
  position: sticky;
  top: 0;
  z-index: 30;
}
.mail-brand {
  align-items: center;
  display: flex;
  font-size: 14px;
  font-weight: 800;
  gap: 8px;
  min-width: 150px;
}
.mail-search {
  justify-self: start;
  max-width: 560px;
  width: 100%;
}
.mail-search input {
  background: var(--bg);
  border-radius: 999px;
  min-height: 38px;
  padding: 8px 16px;
}
.mail-nav-group {
  align-items: center;
  display: flex;
  gap: 24px;
  justify-content: flex-start;
  justify-self: start;
  white-space: nowrap;
}
.mail-nav-link,
.mail-compose-link {
  color: var(--muted);
  display: inline-flex;
  font-weight: 750;
  min-height: 32px;
  place-items: center;
}
.mail-nav-link:hover,
.mail-nav-link:focus,
.mail-compose-link:hover,
.mail-compose-link:focus {
  color: var(--blue);
  outline: none;
}
.mail-compose-link {
  color: var(--blue);
}
.mail-user {
  align-items: center;
  display: flex;
  gap: 10px;
  justify-self: end;
}
.mail-address {
  color: var(--muted);
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.avatar-menu {
  position: relative;
}
.avatar-menu summary {
  list-style: none;
}
.avatar-menu summary::-webkit-details-marker {
  display: none;
}
.avatar-button {
  border-radius: 999px;
  height: 42px;
  padding: 0;
  width: 42px;
}
.mail-menu {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: var(--shadow);
  display: grid;
  gap: 8px;
  min-width: 190px;
  padding: 10px;
  position: absolute;
  right: 0;
  top: calc(100% + 10px);
}
.mail-menu .button,
.mail-menu button {
  justify-content: flex-start;
  width: 100%;
}
.mail-main {
  display: grid;
  gap: 0;
  margin: 0;
  max-width: none;
  min-height: calc(100vh - 56px);
  padding: 0;
}
.mail-workspace {
  min-height: calc(100vh - 56px);
  width: 100%;
}
.mail-pane {
  display: none;
}
.mail-pane[data-active="true"] {
  display: block;
}
.mail-list-pane,
.compose-workspace {
  min-height: calc(100vh - 56px);
}
.mail-list {
  display: grid;
  gap: 1px;
  margin-top: 0;
}
.mail-row {
  align-items: stretch;
  background: transparent;
  border: 0;
  border-bottom: 1px solid var(--line);
  display: grid;
  min-height: 48px;
  position: relative;
}
.mail-row-actions {
  align-items: center;
  display: flex;
  gap: 4px;
  left: 14px;
  padding: 0;
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  z-index: 2;
}
.mail-action-icon {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 999px;
  color: var(--ink);
  display: inline-flex;
  justify-content: center;
  min-height: 32px;
  min-width: 32px;
  padding: 0;
}
.mail-action-icon:hover,
.mail-action-icon:focus,
.mail-action-icon[aria-pressed="true"] {
  background: var(--blue-soft);
  color: var(--blue);
  box-shadow: none;
}
.mail-action-icon svg {
  height: 17px;
  width: 17px;
}
.mail-row-link {
  align-items: center;
  color: inherit;
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(140px, 220px) minmax(0, 1fr) auto;
  min-width: 0;
  min-height: 48px;
  padding: 9px 20px 9px 92px;
  transition: background 160ms ease;
  width: 100%;
}
.mail-row-link:hover,
.mail-row-link:focus {
  background: var(--blue-soft);
  outline: none;
}
.mail-participant {
  color: var(--ink);
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mail-content {
  min-width: 0;
}
.mail-line {
  align-items: baseline;
  display: flex;
  gap: 8px;
  min-width: 0;
  white-space: nowrap;
}
.mail-subject {
  color: var(--ink);
  flex: 0 1 auto;
  font-weight: 750;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mail-preview {
  color: var(--muted);
  flex: 1 1 auto;
  margin: 0;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mail-date {
  color: var(--muted);
  font-size: 12px;
  white-space: nowrap;
}
.message-details {
  color: var(--muted);
  margin-top: 16px;
}
.message-details summary {
  cursor: pointer;
  font-weight: 700;
}
.message-details dl {
  display: grid;
  gap: 6px 12px;
  grid-template-columns: max-content minmax(0, 1fr);
  margin: 8px 0 0;
}
.message-details dt {
  color: var(--ink);
  font-weight: 700;
}
.message-details dd {
  margin: 0;
  overflow-wrap: anywhere;
}
.compose-actions {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.compose-workspace {
  padding: 28px clamp(18px, 4vw, 64px);
}
.form-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-top: 16px;
}
.span-2 { grid-column: 1 / -1; }
.stack {
  display: grid;
  gap: 16px;
}
.status {
  border-radius: 8px;
  color: var(--muted);
  min-height: 22px;
}
.status[data-state="ok"] { color: var(--ok); }
.status[data-state="error"] { color: var(--danger); }
.table-wrap { overflow-x: auto; }
.table {
  border-collapse: collapse;
  margin-top: 16px;
  min-width: 760px;
  width: 100%;
}
.table th,
.table td {
  border-bottom: 1px solid var(--line);
  padding: 10px 8px;
  text-align: left;
  vertical-align: top;
}
.table th {
  color: var(--muted);
  font-size: 12px;
  text-transform: uppercase;
}
.row-form {
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(120px, 1fr) minmax(120px, 1fr) auto;
}
.row-actions {
  display: grid;
  gap: 8px;
}
.record-meta {
  color: var(--muted);
  display: grid;
  gap: 6px;
}
.record-meta p { margin: 0; overflow-wrap: anywhere; }
.record-meta strong { color: var(--ink); margin-right: 6px; }
.endpoint {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  overflow-wrap: anywhere;
}
.message-body {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 8px;
  margin: 10px 0 0;
  max-height: 360px;
  overflow: auto;
  padding: 12px;
  white-space: pre-wrap;
}
.message-details .message-body {
  max-height: 220px;
}
.message-html {
  background: #fff;
  border: 1px solid var(--line);
  border-radius: 8px;
  margin-top: 10px;
  min-height: 360px;
  width: 100%;
}
.message-view {
  color: var(--ink);
  min-height: calc(100vh - 56px);
  padding: 10px clamp(18px, 4vw, 64px) 56px;
}
.message-toolbar {
  align-items: center;
  display: flex;
  gap: 12px;
  min-height: 40px;
}
.message-toolbar form {
  display: inline-flex;
  margin: 0;
}
.message-toolbar-spacer {
  border-left: 1px solid var(--line);
  height: 22px;
}
.message-subject-line {
  font-size: 22px;
  font-weight: 500;
  line-height: 1.35;
  margin: 18px 0 18px 54px;
}
.message-sender-row {
  align-items: start;
  display: grid;
  gap: 12px;
  grid-template-columns: 42px minmax(0, 1fr) auto;
  margin-top: 8px;
}
.sender-avatar {
  align-items: center;
  background: var(--blue);
  border-radius: 999px;
  color: #fff;
  display: inline-flex;
  font-weight: 800;
  height: 40px;
  justify-content: center;
  width: 40px;
}
.sender-primary {
  color: var(--ink);
  font-weight: 800;
}
.sender-secondary {
  color: var(--muted);
  font-size: 13px;
  margin-top: 2px;
}
.message-time {
  color: var(--muted);
  font-size: 13px;
  white-space: nowrap;
}
.message-body-reader {
  background: transparent;
  border: 0;
  color: var(--ink);
  font: inherit;
  line-height: 1.65;
  margin: 14px 0 0 54px;
  max-width: 1120px;
  overflow: visible;
  padding: 0;
  white-space: pre-wrap;
}
.message-html-reader {
  background: #fff;
  border: 0;
  margin: 14px 0 0 54px;
  min-height: 420px;
  width: calc(100% - 54px);
}
.message-view .message-details {
  border-top: 1px solid var(--line);
  margin: 28px 0 0 54px;
  max-width: 1120px;
  padding-top: 12px;
}
@media (max-width: 900px) {
  .shell { grid-template-columns: 1fr; }
  .side { border-bottom: 1px solid var(--line); border-right: 0; }
  .main { padding: 20px; }
  .topbar { align-items: stretch; flex-direction: column; }
  .settings-menu { left: 0; right: auto; }
  .mail-topbar { grid-template-columns: 1fr; padding: 14px 18px; position: static; }
  .mail-nav-group { justify-content: flex-start; overflow-x: auto; }
  .mail-search { justify-self: stretch; }
  .mail-compose-link { justify-self: start; }
  .mail-user { justify-content: space-between; }
  .mail-row-link { grid-template-columns: minmax(0, 1fr); }
  .message-subject-line,
  .message-body-reader,
  .message-html-reader,
  .message-view .message-details { margin-left: 0; width: 100%; }
  .message-sender-row { grid-template-columns: 36px minmax(0, 1fr); }
  .message-time { grid-column: 2; }
  .actions,
  .mail-toolbar,
  .form-grid,
  .grid,
  .row-form { grid-template-columns: 1fr; justify-content: stretch; }
  .span-2 { grid-column: auto; }
}
"#;

const ADMIN_JS: &str = r##"
setupMailPanes();
setupDraftActions();
setupMailSearch();
setupDetailToggles();
setupReplyActions();

document.querySelectorAll("[data-draft-key]").forEach((form) => {
  restoreDraft(form);
  form.addEventListener("input", () => saveDraft(form));
});

document.querySelectorAll("[data-api-form]").forEach((form) => {
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const status = form.querySelector("[data-form-status]");
    setStatus(status, "Saving", "");
    if (form.dataset.draftKey) saveDraft(form);

    try {
      const method = form.dataset.method || "POST";
      const request = {
        method,
        credentials: "same-origin"
      };
      if (method !== "DELETE") {
        request.headers = { "content-type": "application/json" };
        request.body = JSON.stringify(formPayload(form));
      }
      const response = await fetch(form.dataset.endpoint, {
        ...request
      });
      const text = await response.text();
      if (response.status === 401) {
        saveDraft(form);
        redirectToLogin(form);
        return;
      }
      setStatus(status, response.ok ? "Saved" : text || response.statusText, response.ok ? "ok" : "error");
      if (response.ok) clearDraft(form);
      if (response.ok && form.dataset.redirect) {
        location.href = form.dataset.redirect;
        return;
      }
      if (response.ok && form.dataset.reload === "true") location.reload();
      if (response.ok && form.dataset.reset !== "false") form.reset();
    } catch (error) {
      setStatus(status, "Request failed", "error");
    }
  });
});

function setStatus(status, text, state) {
  if (!status) return;
  status.textContent = text;
  status.dataset.state = state;
}

function formPayload(form) {
  const payload = {};
  new FormData(form).forEach((value, key) => {
    const text = String(value).trim();
    if (!text) return;
    if (key === "domains" || key === "roles" || key === "to") payload[key] = splitList(text);
    else if (key === "enabled" || key === "starred" || key.endsWith("_enabled")) payload[key] = text === "true";
    else payload[key] = text;
  });
  return payload;
}

function splitList(value) {
  return value.split(",").map((item) => item.trim()).filter(Boolean);
}

function saveDraft(form) {
  const key = form.dataset.draftKey;
  if (!key) return;
  localStorage.setItem(key, JSON.stringify(formPayload(form)));
}

function restoreDraft(form) {
  const key = form.dataset.draftKey;
  if (!key) return;
  const raw = localStorage.getItem(key);
  if (!raw) return;
  const draft = safeJson(raw);
  if (!draft) return;
  Object.entries(draft).forEach(([name, value]) => setFieldValue(form, name, value));
  if (!location.hash && Object.keys(draft).length > 0) location.hash = draftTarget(form);
}

function clearDraft(form) {
  const key = form.dataset.draftKey;
  if (key) localStorage.removeItem(key);
}

function safeJson(raw) {
  try {
    return JSON.parse(raw);
  } catch (error) {
    return null;
  }
}

function setFieldValue(form, name, value) {
  const field = form.elements.namedItem(name);
  if (!field) return;
  field.value = Array.isArray(value) ? value.join(", ") : value;
}

function draftTarget(form) {
  const panel = form.closest(".compose-panel");
  return panel ? panel.id : form.id;
}

function redirectToLogin(form) {
  const scope = form.dataset.loginScope || "user";
  const next = encodeURIComponent(location.pathname);
  const lang = document.documentElement.lang || "zh";
  const theme = document.documentElement.dataset.theme || "light";
  location.href = `/login?scope=${scope}&next=${next}&lang=${lang}&theme=${theme}`;
}

function setupMailPanes() {
  const panes = Array.from(document.querySelectorAll("[data-mail-pane]"));
  if (panes.length === 0) return;
  const activate = () => {
    const selected = (location.hash || "#inbox").slice(1);
    panes.forEach((pane) => {
      pane.dataset.active = pane.id === selected ? "true" : "false";
    });
  };
  window.addEventListener("hashchange", activate);
  activate();
}

function setupDraftActions() {
  document.querySelectorAll("[data-draft-action]").forEach((button) => {
    button.addEventListener("click", () => handleDraftAction(button));
  });
  document.querySelectorAll("[data-close-compose]").forEach((button) => {
    button.addEventListener("click", () => closeCompose(button));
  });
}

function setupMailSearch() {
  const search = document.querySelector("[data-mail-search]");
  if (!search) return;
  search.addEventListener("input", () => {
    const query = search.value;
    document.querySelectorAll("[data-search-text]").forEach((row) => {
      const value = row.dataset.searchText || "";
      row.hidden = !mailSearchMatches(query, value);
    });
  });
}

function mailSearchMatches(query, value) {
  const terms = searchTerms(query);
  if (terms.length === 0) return true;
  const haystack = normalizeSearchText(value);
  return terms.every((term) => haystack.includes(term) || fuzzyMailMatch(term, haystack));
}

function searchTerms(value) {
  return normalizeSearchText(value).split(" ").filter(Boolean);
}

function normalizeSearchText(value) {
  const text = String(value || "");
  const normalized = text.normalize ? text.normalize("NFKC") : text;
  return normalized.toLocaleLowerCase().replace(/[\u3000\s]+/g, " ").trim();
}

function fuzzyMailMatch(term, haystack) {
  let offset = 0;
  for (const char of term) {
    const found = haystack.indexOf(char, offset);
    if (found === -1) return false;
    offset = found + char.length;
  }
  return true;
}

function setupDetailToggles() {
  document.querySelectorAll("[data-toggle-details]").forEach((button) => {
    button.addEventListener("click", () => openMessageDetails(button));
  });
}

function openMessageDetails(button) {
  const details = document.getElementById(button.dataset.toggleDetails);
  if (!details) return;
  details.open = true;
  details.scrollIntoView({ block: "start", behavior: "smooth" });
}

function setupReplyActions() {
  document.querySelectorAll("[data-reply-action]").forEach((button) => {
    button.addEventListener("click", () => startReply(button));
  });
}

function startReply(button) {
  localStorage.setItem("portal-compose", JSON.stringify(replyDraft(button)));
  location.href = button.dataset.replyHref || "/portal#compose";
}

function replyDraft(button) {
  return compactDraft({
    from: button.dataset.replyFrom || "",
    to: splitList(button.dataset.replyTo || ""),
    subject: button.dataset.replySubject || "",
    text: button.dataset.replyBody || ""
  });
}

function compactDraft(draft) {
  return Object.fromEntries(Object.entries(draft).filter(([, value]) => draftValuePresent(value)));
}

function draftValuePresent(value) {
  if (Array.isArray(value)) return value.length > 0;
  return String(value).trim().length > 0;
}

function handleDraftAction(button) {
  const form = draftForm(button);
  if (!form) return;
  if (button.dataset.draftAction === "save") saveDraft(form);
  if (button.dataset.draftAction === "open") openDraft(form);
  if (button.dataset.draftAction === "discard") discardDraft(form, button);
  const status = form.querySelector("[data-form-status]");
  if (button.dataset.draftAction === "save") setStatus(status, button.dataset.savedText || "Saved", "ok");
}

function openDraft(form) {
  restoreDraft(form);
  location.hash = draftTarget(form);
}

function discardDraft(form, button) {
  if (button.dataset.confirm && !confirm(button.dataset.confirm)) return;
  clearDraft(form);
  form.reset();
}

function closeCompose(button) {
  const form = draftForm(button);
  if (form && hasDraftContent(form)) {
    if (confirm(button.dataset.saveConfirm || "")) saveDraft(form);
    else clearDraft(form);
  }
  location.hash = "inbox";
}

function draftForm(button) {
  return document.getElementById(button.dataset.draftForm || "portal-compose");
}

function hasDraftContent(form) {
  return ["to", "subject", "text", "html"].some((name) => {
    const field = form.elements.namedItem(name);
    return field && String(field.value).trim().length > 0;
  });
}
"##;

pub fn login_page(ctx: &PageContext, scope: LoginScopeView, failed: bool) -> Markup {
    base_page(
        ctx,
        text(ctx.lang, Text::Login),
        html! {
            main id="content" class="main" {
                section class="panel accent" {
                    h1 { (text(ctx.lang, Text::Login)) }
                    form class="form-grid" method="post" action="/login" {
                        input type="hidden" name="scope" value=(scope_value(scope));
                        input type="hidden" name="next" value=(ctx.next);
                        input type="hidden" name="lang" value=(ctx.lang.code());
                        input type="hidden" name="theme" value=(ctx.theme.as_str());
                        @if scope == LoginScopeView::User {
                            label class="span-2" {
                                (text(ctx.lang, Text::Email))
                                input name="identity" type="email" autocomplete="email" required;
                            }
                        }
                        label class="span-2" {
                            (text(ctx.lang, Text::Password))
                            input name="secret" type="password" autocomplete="current-password" required;
                        }
                        div class="span-2" {
                            button type="submit" { (text(ctx.lang, Text::Login)) }
                        }
                        @if failed {
                            p class="status span-2" data-state="error" { (text(ctx.lang, Text::Login)) }
                        }
                    }
                    div class="actions" {
                        (login_language_links(ctx, scope))
                        (login_theme_link(ctx, scope))
                    }
                }
            }
        },
    )
}

pub fn portal_page(ctx: &PageContext, data: &PortalData) -> Markup {
    portal_layout(
        ctx,
        text(ctx.lang, Text::Portal),
        &data.email,
        html! {
            div class="mail-workspace" {
                (message_list(ctx, "inbox", Text::Inbox, &data.inbox, true, true))
                (message_list(ctx, "sent", Text::Sent, &data.sent, false, false))
                (compose_form(ctx, data))
            }
        },
    )
}

pub fn portal_message_page(ctx: &PageContext, data: &crate::PortalMessageData) -> Markup {
    portal_layout(
        ctx,
        &data.message.subject,
        &data.email,
        html! {
            article class="message-view" {
                div class="message-toolbar" {
                    a class="mail-action-icon" href=(localized_path(ctx, "/portal")) aria-label=(text(ctx.lang, Text::Back)) title=(text(ctx.lang, Text::Back)) {
                        (back_icon())
                    }
                    (delete_detail_form(ctx, &data.message))
                    (favorite_detail_form(ctx, &data.message))
                    (reply_detail_button(ctx, &data.message))
                    span class="message-toolbar-spacer" {}
                    button class="mail-action-icon" type="button" data-toggle-details="message-details" title=(text(ctx.lang, Text::Details)) aria-label=(text(ctx.lang, Text::Details)) {
                        (mail_icon())
                    }
                }
                h1 class="message-subject-line" { (&data.message.subject) }
                div class="message-sender-row" {
                    span class="sender-avatar" aria-hidden="true" { (avatar_label(&data.message.from)) }
                    div {
                        div class="sender-primary" { (&data.message.from) }
                        div class="sender-secondary" { (text(ctx.lang, Text::To)) " " (&data.message.to) }
                    }
                    time class="message-time" datetime=(&data.message.received_at) { (message_date(&data.message.received_at)) }
                }
                (primary_message_body(ctx, &data.message))
                (message_detail_summary(ctx, &data.message))
            }
        },
    )
}

fn compose_form(ctx: &PageContext, data: &PortalData) -> Markup {
    html! {
        section id="compose" class="compose-panel compose-workspace mail-pane" data-mail-pane="" data-active="false" {
            div class="compose-heading" {
                h2 { (text(ctx.lang, Text::Compose)) }
                button class="button secondary" type="button" data-close-compose="" data-save-confirm=(text(ctx.lang, Text::SaveDraftConfirm)) {
                    (text(ctx.lang, Text::Back))
                }
            }
            form id="portal-compose" class="form-grid" data-api-form="" data-reload="true" data-draft-key="portal-compose" data-login-scope="user" data-endpoint="/api/v1/portal/mail/send" {
                label {
                    (text(ctx.lang, Text::From))
                    select name="from" {
                        @for mailbox in &data.mailboxes {
                            option value=(&mailbox.email) { (&mailbox.email) }
                        }
                    }
                }
                (field(ctx, Text::To, "to", "text", ""))
                (field(ctx, Text::Subject, "subject", "text", ""))
                label class="span-2" {
                    (text(ctx.lang, Text::Body))
                    textarea name="text" required {}
                }
                div class="span-2 compose-actions" {
                    button type="submit" { (text(ctx.lang, Text::Send)) }
                    button class="secondary" type="button" data-draft-action="open" data-draft-form="portal-compose" {
                        (text(ctx.lang, Text::Drafts))
                    }
                    button class="secondary" type="button" data-draft-action="save" data-draft-form="portal-compose" data-saved-text=(text(ctx.lang, Text::SaveDraft)) {
                        (text(ctx.lang, Text::SaveDraft))
                    }
                    button class="secondary" type="button" data-draft-action="discard" data-draft-form="portal-compose" data-confirm=(text(ctx.lang, Text::DiscardDraftConfirm)) {
                        (text(ctx.lang, Text::DiscardDraft))
                    }
                }
                p class="status span-2" data-form-status="" {}
            }
        }
    }
}

fn message_list(
    ctx: &PageContext,
    id: &str,
    title: Text,
    messages: &[crate::MessageRow],
    inbound: bool,
    active: bool,
) -> Markup {
    html! {
        section id=(id) class="mail-pane mail-list-pane" data-mail-pane="" data-active=(if active { "true" } else { "false" }) aria-label=(text(ctx.lang, title)) {
            div class="mail-list" {
                @for message in messages {
                    (message_row(ctx, message, inbound))
                }
            }
        }
    }
}

fn message_row(ctx: &PageContext, message: &crate::MessageRow, inbound: bool) -> Markup {
    html! {
        article class="mail-row" data-search-text=(message_search_text(message)) {
            a class="mail-row-link" href=(localized_path(ctx, &message_detail_path(message, inbound))) {
                div class="mail-participant" {
                    @if inbound { (&message.from) } @else { (&message.to) }
                }
                div class="mail-content" {
                    div class="mail-line" {
                        span class="mail-subject" { (&message.subject) }
                        span class="mail-preview" { (&message.text) }
                    }
                }
                time class="mail-date" datetime=(&message.at) { (message_date(&message.at)) }
            }
            div class="mail-row-actions" {
                (favorite_form(ctx, message, inbound))
                (delete_message_form(ctx, message, inbound))
            }
        }
    }
}

fn favorite_form(ctx: &PageContext, message: &crate::MessageRow, inbound: bool) -> Markup {
    let next_state = (!message.starred).to_string();
    let label = if message.starred {
        text(ctx.lang, Text::Unfavorite)
    } else {
        text(ctx.lang, Text::Favorite)
    };
    html! {
        form data-api-form="" data-reset="false" data-reload="true" data-endpoint=(favorite_endpoint(message, inbound)) {
            input type="hidden" name="starred" value=(next_state);
            button class="mail-action-icon" type="submit" aria-label=(label) title=(label) aria-pressed=(message.starred.to_string()) {
                (star_icon(message.starred))
            }
        }
    }
}

fn delete_message_form(ctx: &PageContext, message: &crate::MessageRow, inbound: bool) -> Markup {
    html! {
        form data-api-form="" data-reset="false" data-reload="true" data-method="DELETE" data-endpoint=(message_endpoint(message, inbound)) {
            button class="mail-action-icon" type="submit" aria-label=(text(ctx.lang, Text::Delete)) title=(text(ctx.lang, Text::Delete)) {
                (trash_icon())
            }
        }
    }
}

fn favorite_detail_form(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    let next_state = (!message.starred).to_string();
    let label = if message.starred {
        text(ctx.lang, Text::Unfavorite)
    } else {
        text(ctx.lang, Text::Favorite)
    };
    html! {
        form data-api-form="" data-reset="false" data-reload="true" data-endpoint=(format!("{}/favorite", message_detail_endpoint(message))) {
            input type="hidden" name="starred" value=(next_state);
            button class="mail-action-icon" type="submit" aria-label=(label) title=(label) aria-pressed=(message.starred.to_string()) {
                (star_icon(message.starred))
            }
        }
    }
}

fn delete_detail_form(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    html! {
        form data-api-form="" data-reset="false" data-method="DELETE" data-endpoint=(message_detail_endpoint(message)) data-redirect=(localized_path(ctx, "/portal")) {
            button class="mail-action-icon" type="submit" aria-label=(text(ctx.lang, Text::Delete)) title=(text(ctx.lang, Text::Delete)) {
                (trash_icon())
            }
        }
    }
}

fn reply_detail_button(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    html! {
        button class="mail-action-icon" type="button" data-reply-action="" data-reply-from=(&message.mailbox) data-reply-to=(reply_recipient(message)) data-reply-subject=(reply_subject(&message.subject)) data-reply-body=(reply_body(message)) data-reply-href=(localized_anchor_path(ctx, "/portal", "compose")) aria-label=(text(ctx.lang, Text::Reply)) title=(text(ctx.lang, Text::Reply)) {
            (reply_icon())
        }
    }
}

fn message_detail_path(message: &crate::MessageRow, inbound: bool) -> String {
    format!(
        "/portal/{}/{}",
        if inbound { "inbound" } else { "outbound" },
        message.id
    )
}

fn message_search_text(message: &crate::MessageRow) -> String {
    [
        message.from.as_str(),
        message.to.as_str(),
        message.subject.as_str(),
        message.text.as_str(),
    ]
    .join(" ")
}

fn message_date(value: &str) -> &str {
    value
        .split_once('T')
        .map(|(date, _)| date)
        .filter(|date| !date.is_empty())
        .unwrap_or(value)
}

fn star_icon(filled: bool) -> Markup {
    html! {
        svg viewBox="0 0 24 24" fill=(if filled { "currentColor" } else { "none" }) stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
            polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" {}
        }
    }
}

fn trash_icon() -> Markup {
    html! {
        svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
            path d="M3 6h18" {}
            path d="M8 6V4h8v2" {}
            path d="M19 6l-1 14H6L5 6" {}
            path d="M10 11v6" {}
            path d="M14 11v6" {}
        }
    }
}

fn back_icon() -> Markup {
    html! {
        svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
            path d="M19 12H5" {}
            path d="m12 19-7-7 7-7" {}
        }
    }
}

fn mail_icon() -> Markup {
    html! {
        svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
            rect x="3" y="5" width="18" height="14" rx="2" {}
            path d="m3 7 9 6 9-6" {}
        }
    }
}

fn reply_icon() -> Markup {
    html! {
        svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" {
            path d="m9 17-5-5 5-5" {}
            path d="M20 18v-2a4 4 0 0 0-4-4H4" {}
        }
    }
}

fn primary_message_body(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    if !message.text.trim().is_empty() {
        return html! {
            pre class="message-body-reader" { (&message.text) }
        };
    }
    if !message.html.trim().is_empty() {
        return html! {
            iframe class="message-html-reader" sandbox="" referrerpolicy="no-referrer" srcdoc=(&message.html) {}
        };
    }
    message_body_status(ctx, message)
}

fn message_body_status(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    if !message.text.trim().is_empty() || !message.html.trim().is_empty() {
        return html! {};
    }
    html! {
        div class="message-body-reader" {
            p class="status" data-state=(body_status_state(message)) {
                (text(ctx.lang, Text::BodyUnavailable))
                @if !message.detail_error.is_empty() {
                    " "
                    (text(ctx.lang, Text::DetailFetchFailed))
                    " "
                    code class="endpoint" { (&message.detail_error) }
                } @else if message.detail_loaded {
                    " "
                    (text(ctx.lang, Text::ProviderDidNotReturnBody))
                }
            }
        }
    }
}

fn message_detail_summary(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    html! {
        details id="message-details" class="message-details" {
            summary { (text(ctx.lang, Text::Details)) }
            dl {
                (detail_pair(ctx, Text::Email, &message.mailbox))
                (detail_code_pair(ctx, Text::ProviderId, &message.provider_id))
                (detail_pair(ctx, Text::Status, message_status_text(ctx, &message.status)))
                (detail_pair(ctx, Text::From, &message.from))
                (detail_pair(ctx, Text::To, &message.to))
                (detail_pair(ctx, Text::Cc, &message.cc))
                (detail_pair(ctx, Text::Bcc, &message.bcc))
                (detail_pair(ctx, Text::ReplyTo, &message.reply_to))
                (detail_pair(ctx, Text::ReceivedAt, &message.received_at))
                (detail_pair(ctx, Text::DetailFetchFailed, &message.detail_error))
                @if !message.html.is_empty() && message.html != message.text {
                    dt { (text(ctx.lang, Text::Html)) }
                    dd { pre class="message-body" { (&message.html) } }
                }
                (detail_headers(ctx, &message.headers))
                (detail_attachments(ctx, &message.attachments))
                (raw_message_detail(ctx, message))
            }
        }
    }
}

fn detail_pair(ctx: &PageContext, label: Text, value: &str) -> Markup {
    match value.is_empty() {
        true => html! {},
        false => html! {
            dt { (text(ctx.lang, label)) }
            dd { (value) }
        },
    }
}

fn detail_code_pair(ctx: &PageContext, label: Text, value: &str) -> Markup {
    match value.is_empty() {
        true => html! {},
        false => html! {
            dt { (text(ctx.lang, label)) }
            dd { code class="endpoint" { (value) } }
        },
    }
}

fn detail_headers(ctx: &PageContext, headers: &[crate::MessageHeaderRow]) -> Markup {
    match headers.is_empty() {
        true => html! {},
        false => html! {
            dt { (text(ctx.lang, Text::Headers)) }
            dd {
                div class="table-wrap" {
                    table class="table" {
                        tbody {
                            @for header in headers {
                                tr {
                                    td { (&header.name) }
                                    td { (&header.value) }
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}

fn detail_attachments(ctx: &PageContext, attachments: &[crate::MessageAttachmentRow]) -> Markup {
    match attachments.is_empty() {
        true => html! {},
        false => html! {
            dt { (text(ctx.lang, Text::Attachments)) }
            dd {
                div class="table-wrap" {
                    table class="table" {
                        thead {
                            tr {
                                th { (text(ctx.lang, Text::File)) }
                                th { (text(ctx.lang, Text::ContentType)) }
                                th { (text(ctx.lang, Text::Status)) }
                                th { (text(ctx.lang, Text::ContentId)) }
                            }
                        }
                        tbody {
                            @for attachment in attachments {
                                tr {
                                    td { (&attachment.filename) }
                                    td { (&attachment.content_type) }
                                    td { (&attachment.content_disposition) }
                                    td { (&attachment.content_id) }
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}

fn raw_message_detail(ctx: &PageContext, message: &crate::MessageDetailRow) -> Markup {
    match message.raw_download_url.is_empty() {
        true => html! {},
        false => html! {
            dt { (text(ctx.lang, Text::RawMessage)) }
            dd {
                @if !message.raw_expires_at.is_empty() {
                    p { (text(ctx.lang, Text::ExpiresAt)) ": " (&message.raw_expires_at) }
                }
                a href=(&message.raw_download_url) target="_blank" rel="noopener noreferrer" {
                    (&message.raw_download_url)
                }
            }
        },
    }
}

fn body_status_state(message: &crate::MessageDetailRow) -> &'static str {
    match message.detail_error.is_empty() && message.detail_loaded {
        true => "",
        false => "error",
    }
}

fn message_status_text<'a>(ctx: &PageContext, status: &'a str) -> &'a str {
    match status {
        "Received" => text(ctx.lang, Text::Received),
        "Sent" => text(ctx.lang, Text::Sent),
        _ => status,
    }
}

fn favorite_endpoint(message: &crate::MessageRow, inbound: bool) -> String {
    format!("{}/favorite", message_endpoint(message, inbound))
}

fn message_endpoint(message: &crate::MessageRow, inbound: bool) -> String {
    format!(
        "/api/v1/portal/mail/{}/{}",
        direction_path(inbound),
        message.provider_id
    )
}

fn message_detail_endpoint(message: &crate::MessageDetailRow) -> String {
    format!(
        "/api/v1/portal/mail/{}/{}",
        message.direction, message.provider_id
    )
}

fn direction_path(inbound: bool) -> &'static str {
    match inbound {
        true => "inbound",
        false => "outbound",
    }
}

fn reply_recipient(message: &crate::MessageDetailRow) -> &str {
    if !message.reply_to.trim().is_empty() {
        return &message.reply_to;
    }
    if message.direction == "outbound" && !message.to.trim().is_empty() {
        return &message.to;
    }
    &message.from
}

fn reply_subject(subject: &str) -> String {
    match subject.trim_start().to_ascii_lowercase().starts_with("re:") {
        true => subject.to_string(),
        false => format!("Re: {subject}"),
    }
}

fn reply_body(message: &crate::MessageDetailRow) -> String {
    let body = message.text.trim();
    match body.is_empty() {
        true => String::new(),
        false => format!(
            "\n\n\nOn {}, {} wrote:\n{}",
            message.received_at, message.from, body
        ),
    }
}

fn avatar_label(email: &str) -> String {
    email
        .chars()
        .next()
        .map(|value| value.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "R".to_string())
}

pub fn admin_page(ctx: &PageContext, section: AdminSection, data: &AdminData) -> Markup {
    app_layout(
        ctx,
        title(ctx.lang, section),
        true,
        admin_content(ctx, section, data),
    )
}

fn admin_content(ctx: &PageContext, section: AdminSection, data: &AdminData) -> Markup {
    match section {
        AdminSection::Dashboard => dashboard(ctx, data),
        AdminSection::Users => users(ctx, data),
        AdminSection::Domains => domains(ctx, data),
        AdminSection::Providers => providers(ctx, data),
        AdminSection::Mailboxes => mailboxes(ctx, data),
        AdminSection::Audit => audit(ctx, data),
    }
}

fn dashboard(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        section class="grid" {
            (summary_card(ctx, Text::Users, "/admin/users", data.users.len()))
            (summary_card(ctx, Text::Domains, "/admin/domains", data.domains.len()))
            (summary_card(ctx, Text::Providers, "/admin/providers", data.providers.len()))
            (summary_card(ctx, Text::Mailboxes, "/admin/mailboxes", data.mailboxes.len()))
        }
    }
}

fn users(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        (create_user_form(ctx))
        section class="panel" {
            h2 { (text(ctx.lang, Text::Users)) }
            div class="table-wrap" {
                table class="table" {
                    thead { tr { th { (text(ctx.lang, Text::Email)) } th { (text(ctx.lang, Text::Status)) } th { (text(ctx.lang, Text::Save)) } } }
                    tbody {
                        @for user in &data.users {
                            tr {
                                td { (&user.email) }
                                td { (&user.status) }
                                td { (user_update_form(ctx, user)) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn create_user_form(ctx: &PageContext) -> Markup {
    html! {
        section class="panel accent" {
            h2 { (text(ctx.lang, Text::Add)) " " (text(ctx.lang, Text::Users)) }
            form class="form-grid" data-api-form="" data-reload="true" data-endpoint="/api/v1/admin/users" {
                (field(ctx, Text::Name, "display_name", "text", ""))
                (field(ctx, Text::Email, "email", "email", ""))
                (field(ctx, Text::Roles, "roles", "text", "MailUser"))
                (field(ctx, Text::Password, "login_secret", "password", ""))
                (submit_row(ctx, Text::Create))
            }
        }
    }
}

fn user_update_form(ctx: &PageContext, user: &crate::UserRow) -> Markup {
    html! {
        form class="row-form" data-api-form="" data-reset="false" data-method="PATCH" data-endpoint=(format!("/api/v1/admin/users/{}", user.email)) {
            input name="display_name" value=(&user.display_name);
            input name="roles" value=(&user.roles);
            select name="status" {
                option value="Active" selected[user.status == "Active"] { "Active" }
                option value="Disabled" selected[user.status == "Disabled"] { "Disabled" }
            }
            input name="login_secret" type="password" placeholder=(text(ctx.lang, Text::Password));
            button type="submit" { (text(ctx.lang, Text::Save)) }
            p class="status" data-form-status="" {}
        }
    }
}

fn domains(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        section class="panel accent" {
            h2 { (text(ctx.lang, Text::Add)) " " (text(ctx.lang, Text::Domains)) }
            form class="form-grid" data-api-form="" data-reload="true" data-endpoint="/api/v1/admin/domains" {
                (field(ctx, Text::Domains, "domain", "text", ""))
                (submit_row(ctx, Text::Create))
            }
        }
        section class="panel" {
            h2 { (text(ctx.lang, Text::Domains)) }
            div class="table-wrap" {
                table class="table" {
                    tbody {
                        @for domain in &data.domains {
                            tr { td { (domain_update_form(ctx, domain)) } }
                        }
                    }
                }
            }
        }
    }
}

fn domain_update_form(ctx: &PageContext, domain: &crate::DomainRow) -> Markup {
    html! {
        div class="row-actions" {
            form class="row-form" data-api-form="" data-reset="false" data-reload="true" data-method="PATCH" data-endpoint=(format!("/api/v1/admin/domains/{}", domain.domain)) {
                input name="domain" value=(&domain.domain);
                button type="submit" { (text(ctx.lang, Text::Save)) }
                p class="status" data-form-status="" {}
            }
            (delete_form(ctx, &format!("/api/v1/admin/domains/{}", domain.domain)))
        }
    }
}

fn providers(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        (create_provider_form(ctx))
        section class="panel" {
            h2 { (text(ctx.lang, Text::Providers)) }
            div class="table-wrap" {
                table class="table" {
                    tbody {
                        @for provider in &data.providers {
                            tr { td { (provider_update_form(ctx, provider)) } }
                        }
                    }
                }
            }
        }
    }
}

fn create_provider_form(ctx: &PageContext) -> Markup {
    html! {
        section class="panel accent" {
            h2 { (text(ctx.lang, Text::Add)) " " (text(ctx.lang, Text::Providers)) }
            form class="form-grid" data-api-form="" data-reload="true" data-endpoint="/api/v1/admin/provider-accounts" {
                (field(ctx, Text::Name, "name", "text", ""))
                label { (text(ctx.lang, Text::Providers)) select name="provider_type" { option value="resend" { "Resend" } } }
                (field(ctx, Text::Domains, "domains", "text", ""))
                (field(ctx, Text::ApiKey, "api_key", "password", ""))
                (field(ctx, Text::WebhookSecret, "webhook_secret", "password", ""))
                (submit_row(ctx, Text::Create))
            }
        }
    }
}

fn provider_update_form(ctx: &PageContext, provider: &crate::ProviderRow) -> Markup {
    html! {
        div class="row-actions" {
            div class="record-meta" {
                p {
                    strong { (text(ctx.lang, Text::ProviderId)) }
                    code class="endpoint" { (&provider.id) }
                }
                p {
                    strong { (text(ctx.lang, Text::WebhookEndpoint)) }
                    code class="endpoint" { (&provider.webhook_endpoint) }
                }
            }
            form class="row-form" data-api-form="" data-reset="false" data-reload="true" data-method="PATCH" data-endpoint=(format!("/api/v1/admin/provider-accounts/{}", provider.id)) {
                input name="name" value=(&provider.name);
                input name="domains" value=(&provider.domains);
                select name="enabled" {
                    option value="true" selected[provider.enabled] { (text(ctx.lang, Text::Enabled)) }
                    option value="false" selected[!provider.enabled] { (text(ctx.lang, Text::Disabled)) }
                }
                input name="api_key" type="password" placeholder=(secret_placeholder(ctx, provider.api_key_configured));
                input name="webhook_secret" type="password" placeholder=(text(ctx.lang, Text::WebhookSecret));
                span class="status" {
                    (text(ctx.lang, Text::ApiKey)) ": "
                    @if provider.api_key_configured {
                        (text(ctx.lang, Text::Configured))
                    } @else {
                        (text(ctx.lang, Text::Disabled))
                    }
                }
                button type="submit" { (text(ctx.lang, Text::Save)) }
                p class="status" data-form-status="" {}
            }
            (delete_form(ctx, &format!("/api/v1/admin/provider-accounts/{}", provider.id)))
        }
    }
}

fn secret_placeholder(ctx: &PageContext, configured: bool) -> &'static str {
    match configured {
        true => text(ctx.lang, Text::SecretConfiguredHint),
        false => text(ctx.lang, Text::ApiKey),
    }
}

fn mailboxes(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        section class="panel accent" {
            h2 { (text(ctx.lang, Text::Add)) " " (text(ctx.lang, Text::Mailboxes)) }
            form class="form-grid" data-api-form="" data-reload="true" data-endpoint="/api/v1/admin/mailboxes" {
                (field(ctx, Text::Email, "owner_email", "email", ""))
                (field(ctx, Text::Mailboxes, "mailbox_email", "email", ""))
                (submit_row(ctx, Text::Create))
            }
        }
        section class="panel" {
            h2 { (text(ctx.lang, Text::Mailboxes)) }
            div class="table-wrap" {
                table class="table" {
                    thead {
                        tr {
                            th { (text(ctx.lang, Text::Email)) }
                            th { (text(ctx.lang, Text::Users)) }
                            th { (text(ctx.lang, Text::Status)) }
                        }
                    }
                    tbody {
                        @for mailbox in &data.mailboxes {
                            tr {
                                td { (&mailbox.email) }
                                td { (&mailbox.owner) }
                                td { (mailbox_update_form(ctx, mailbox)) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn mailbox_update_form(ctx: &PageContext, mailbox: &crate::MailboxRow) -> Markup {
    html! {
        form class="row-form" data-api-form="" data-reset="false" data-method="PATCH" data-endpoint=(format!("/api/v1/admin/mailboxes/{}", mailbox.email)) {
            select name="status" {
                option value="Active" selected[mailbox.status == "Active"] { "Active" }
                option value="Disabled" selected[mailbox.status == "Disabled"] { "Disabled" }
            }
            (bool_select(ctx, "inbound_enabled", mailbox.inbound_enabled, Text::Inbound))
            (bool_select(ctx, "outbound_enabled", mailbox.outbound_enabled, Text::Outbound))
            button type="submit" { (text(ctx.lang, Text::Save)) }
            p class="status" data-form-status="" {}
        }
    }
}

fn audit(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        section class="panel accent" {
            h2 { (text(ctx.lang, Text::Audit)) }
            p { (text(ctx.lang, Text::AuditRetention)) }
            div class="table-wrap" {
                table class="table" {
                    thead { tr { th { "Time" } th { "Action" } th { "Target" } th { "Result" } } }
                    tbody {
                        @for event in &data.audit_events {
                            tr {
                                td { (&event.at) }
                                td { (&event.action) }
                                td { (&event.target) }
                                td { (&event.result) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn portal_layout(ctx: &PageContext, title: &str, email: &str, content: Markup) -> Markup {
    base_page(
        ctx,
        title,
        html! {
            a class="skip" href="#content" { "Skip" }
            div class="mail-app" {
                (mail_topbar(ctx, email))
                main id="content" class="mail-main" {
                    (content)
                }
            }
            script { (PreEscaped(ADMIN_JS)) }
        },
    )
}

fn mail_topbar(ctx: &PageContext, email: &str) -> Markup {
    html! {
        header class="mail-topbar" {
            a class="mail-brand" href=(localized_path(ctx, "/portal")) {
                (brand_mark())
                span { "RNovEmail" }
            }
            label class="mail-search" {
                input type="search" data-mail-search="" placeholder=(text(ctx.lang, Text::SearchMail)) autocomplete="off";
            }
            nav class="mail-nav-group" aria-label=(text(ctx.lang, Text::Portal)) {
                a class="mail-nav-link" href="#inbox" { (text(ctx.lang, Text::Inbox)) }
                a class="mail-nav-link" href="#sent" { (text(ctx.lang, Text::Sent)) }
                a class="mail-compose-link" href="#compose" { (text(ctx.lang, Text::Compose)) }
            }
            div class="mail-user" {
                (avatar_menu(ctx, email))
            }
        }
    }
}

fn brand_mark() -> Markup {
    html! {
        svg id="rnovemail-logo" class="mark logo-mark" viewBox="0 0 40 40" role="img" aria-label="RNovEmail" {
            defs {
                linearGradient id="rnovemail-logo-gradient" x1="6" y1="34" x2="34" y2="6" gradientUnits="userSpaceOnUse" {
                    stop offset="0%" stop-color="#14b8a6" {}
                    stop offset="52%" stop-color="#2563eb" {}
                    stop offset="100%" stop-color="#8b5cf6" {}
                }
            }
            rect x="3" y="3" width="34" height="34" rx="9" fill="url(#rnovemail-logo-gradient)" {}
            path d="M10.5 15.5h19v13h-19z" fill="#ffffff" opacity="0.18" {}
            path d="M10.5 15.5h19v13h-19z" fill="none" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" {}
            path d="M11.5 16.5 20 22.5l8.5-6" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {}
            path d="M23.5 18.5 28.5 12.5" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" {}
            circle cx="30" cy="11" r="3.2" fill="#ffffff" {}
        }
    }
}

fn avatar_menu(ctx: &PageContext, email: &str) -> Markup {
    html! {
        details class="avatar-menu" {
            summary class="button avatar-button" aria-label=(text(ctx.lang, Text::Settings)) {
                (avatar_label(email))
            }
            div class="mail-menu" {
                (language_links(ctx))
                (theme_link(ctx))
                form method="post" action="/logout" {
                    button class="secondary" type="submit" { (text(ctx.lang, Text::Logout)) }
                }
            }
        }
    }
}

fn app_layout(ctx: &PageContext, title: &str, admin: bool, content: Markup) -> Markup {
    base_page(
        ctx,
        title,
        html! {
            a class="skip" href="#content" { "Skip" }
            div class="shell" {
                aside class="side" {
                    div class="brand" {
                        (brand_mark())
                        span { "RNovEmail" }
                    }
                    @if admin { (admin_nav(ctx)) }
                }
                main id="content" class="main" {
                    div class="topbar" {
                        div {
                            p class="eyebrow" { (if admin { text(ctx.lang, Text::AdminConsole) } else { text(ctx.lang, Text::Portal) }) }
                            h1 { (title) }
                        }
                        div class="actions" {
                            (settings_menu(ctx))
                        }
                    }
                    div class="stack" { (content) }
                }
            }
            script { (PreEscaped(ADMIN_JS)) }
        },
    )
}

fn base_page(ctx: &PageContext, title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(ctx.lang.code()) data-theme=(ctx.theme.as_str()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                (favicon_link())
                title { "RNovEmail " (title) }
                style { (PreEscaped(ADMIN_CSS)) }
            }
            body { (body) }
        }
    }
}

fn favicon_link() -> Markup {
    html! {
        link id="rnovemail-favicon" rel="icon" type="image/svg+xml" href=(favicon_href());
    }
}

fn favicon_href() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 40 40'%3E%3Cdefs%3E%3ClinearGradient id='g' x1='6' y1='34' x2='34' y2='6' gradientUnits='userSpaceOnUse'%3E%3Cstop stop-color='%2314b8a6'/%3E%3Cstop offset='.52' stop-color='%232563eb'/%3E%3Cstop offset='1' stop-color='%238b5cf6'/%3E%3C/linearGradient%3E%3C/defs%3E%3Crect x='3' y='3' width='34' height='34' rx='9' fill='url(%23g)'/%3E%3Cpath d='M10.5 15.5h19v13h-19z' fill='none' stroke='white' stroke-width='2'/%3E%3Cpath d='M11.5 16.5 20 22.5l8.5-6' fill='none' stroke='white' stroke-width='2' stroke-linecap='round'/%3E%3Ccircle cx='30' cy='11' r='3.2' fill='white'/%3E%3C/svg%3E"
}

fn admin_nav(ctx: &PageContext) -> Markup {
    html! {
        nav class="nav" aria-label="Admin navigation" {
            a href=(localized_path(ctx, "/admin")) { (text(ctx.lang, Text::Operations)) }
            a href=(localized_path(ctx, "/admin/users")) { (text(ctx.lang, Text::Users)) }
            a href=(localized_path(ctx, "/admin/domains")) { (text(ctx.lang, Text::Domains)) }
            a href=(localized_path(ctx, "/admin/providers")) { (text(ctx.lang, Text::Providers)) }
            a href=(localized_path(ctx, "/admin/mailboxes")) { (text(ctx.lang, Text::Mailboxes)) }
            a href=(localized_path(ctx, "/admin/audit")) { (text(ctx.lang, Text::Audit)) }
        }
    }
}

fn language_links(ctx: &PageContext) -> Markup {
    html! {
        a class="button secondary" href=(switch_lang(ctx, Lang::Zh)) { (text(ctx.lang, Text::LanguageZh)) }
        a class="button secondary" href=(switch_lang(ctx, Lang::Ja)) { (text(ctx.lang, Text::LanguageJa)) }
    }
}

fn settings_menu(ctx: &PageContext) -> Markup {
    html! {
        details class="settings" {
            summary class="button secondary" { (text(ctx.lang, Text::Settings)) }
            div class="settings-menu" {
                (language_links(ctx))
                (theme_link(ctx))
                form method="post" action="/logout" {
                    button class="secondary" type="submit" { (text(ctx.lang, Text::Logout)) }
                }
            }
        }
    }
}

fn theme_link(ctx: &PageContext) -> Markup {
    html! {
        a class="button secondary" href=(switch_theme(ctx)) {
            (text(ctx.lang, Text::Theme)) ": " (theme_name(ctx.lang, ctx.theme.opposite()))
        }
    }
}

fn login_language_links(ctx: &PageContext, scope: LoginScopeView) -> Markup {
    html! {
        a class="button secondary" href=(login_href(ctx, scope, Lang::Zh, ctx.theme)) { (text(ctx.lang, Text::LanguageZh)) }
        a class="button secondary" href=(login_href(ctx, scope, Lang::Ja, ctx.theme)) { (text(ctx.lang, Text::LanguageJa)) }
    }
}

fn login_theme_link(ctx: &PageContext, scope: LoginScopeView) -> Markup {
    html! {
        a class="button secondary" href=(login_href(ctx, scope, ctx.lang, ctx.theme.opposite())) {
            (text(ctx.lang, Text::Theme)) ": " (theme_name(ctx.lang, ctx.theme.opposite()))
        }
    }
}

fn field(ctx: &PageContext, label: Text, name: &str, input_type: &str, value: &str) -> Markup {
    html! {
        label {
            (text(ctx.lang, label))
            input name=(name) type=(input_type) value=(value);
        }
    }
}

fn submit_row(ctx: &PageContext, label: Text) -> Markup {
    html! {
        div class="span-2" { button type="submit" { (text(ctx.lang, label)) } }
        p class="status span-2" data-form-status="" {}
    }
}

fn bool_select(ctx: &PageContext, name: &str, value: bool, label: Text) -> Markup {
    html! {
        label {
            (text(ctx.lang, label))
            select name=(name) {
                option value="true" selected[value] { (text(ctx.lang, Text::Enabled)) }
                option value="false" selected[!value] { (text(ctx.lang, Text::Disabled)) }
            }
        }
    }
}

fn delete_form(ctx: &PageContext, endpoint: &str) -> Markup {
    html! {
        form data-api-form="" data-reset="false" data-reload="true" data-method="DELETE" data-endpoint=(endpoint) {
            button class="secondary" type="submit" { (text(ctx.lang, Text::Delete)) }
            p class="status" data-form-status="" {}
        }
    }
}

fn summary_card(ctx: &PageContext, key: Text, href: &str, count: usize) -> Markup {
    html! {
        a class="panel" href=(localized_path(ctx, href)) {
            h2 { (text(ctx.lang, key)) }
            p { (count) }
        }
    }
}

fn title(lang: Lang, section: AdminSection) -> &'static str {
    match section {
        AdminSection::Audit => text(lang, Text::Audit),
        AdminSection::Dashboard => text(lang, Text::Operations),
        AdminSection::Domains => text(lang, Text::Domains),
        AdminSection::Mailboxes => text(lang, Text::Mailboxes),
        AdminSection::Providers => text(lang, Text::Providers),
        AdminSection::Users => text(lang, Text::Users),
    }
}

fn theme_name(lang: Lang, theme: Theme) -> &'static str {
    match theme {
        Theme::Light => text(lang, Text::Light),
        Theme::Dark => text(lang, Text::Dark),
    }
}

fn scope_value(scope: LoginScopeView) -> &'static str {
    match scope {
        LoginScopeView::Admin => "admin",
        LoginScopeView::User => "user",
    }
}

fn localized_path(ctx: &PageContext, path: &str) -> String {
    format!(
        "{}?lang={}&theme={}",
        path,
        ctx.lang.code(),
        ctx.theme.as_str()
    )
}

fn localized_anchor_path(ctx: &PageContext, path: &str, anchor: &str) -> String {
    format!(
        "{}?lang={}&theme={}#{}",
        path,
        ctx.lang.code(),
        ctx.theme.as_str(),
        anchor
    )
}

fn switch_lang(ctx: &PageContext, lang: Lang) -> String {
    format!("?lang={}&theme={}", lang.code(), ctx.theme.as_str())
}

fn switch_theme(ctx: &PageContext) -> String {
    format!(
        "?lang={}&theme={}",
        ctx.lang.code(),
        ctx.theme.opposite().as_str()
    )
}

fn login_href(ctx: &PageContext, scope: LoginScopeView, lang: Lang, theme: Theme) -> String {
    format!(
        "?scope={}&next={}&lang={}&theme={}",
        scope_value(scope),
        query_value(&ctx.next),
        lang.code(),
        theme.as_str()
    )
}

fn query_value(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.bytes() {
        push_query_byte(byte, &mut escaped);
    }
    escaped
}

fn push_query_byte(byte: u8, target: &mut String) {
    match byte {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
            target.push(byte as char);
        }
        _ => {
            let _ = write!(target, "%{byte:02X}");
        }
    }
}
