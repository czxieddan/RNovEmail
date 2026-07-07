use maud::{DOCTYPE, Markup, PreEscaped, html};

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
  --shadow: 0 18px 45px rgba(15, 23, 42, 0.08);
}
* { box-sizing: border-box; }
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
  color: #fff;
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
  border-right: 1px solid var(--line);
  background: #fff;
  padding: 28px 20px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 800;
  letter-spacing: 0;
}
.mark {
  display: inline-grid;
  width: 34px;
  height: 34px;
  place-items: center;
  border-radius: 8px;
  background: var(--blue);
  color: #fff;
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
.token {
  align-items: end;
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(220px, 320px) auto;
}
label {
  color: var(--ink);
  display: grid;
  font-weight: 650;
  gap: 6px;
}
input,
textarea,
select {
  background: #fff;
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--ink);
  font: inherit;
  min-height: 40px;
  padding: 9px 11px;
  width: 100%;
}
textarea { min-height: 88px; resize: vertical; }
input:focus,
textarea:focus,
select:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.14);
  outline: none;
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
  background: #1d4ed8;
  box-shadow: 0 10px 24px rgba(37, 99, 235, 0.2);
  outline: none;
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
.status[data-state="ok"] { color: #047857; }
.status[data-state="error"] { color: #b91c1c; }
.table {
  border-collapse: collapse;
  margin-top: 16px;
  width: 100%;
}
.table th,
.table td {
  border-bottom: 1px solid var(--line);
  padding: 10px 0;
  text-align: left;
}
.table th { color: var(--muted); font-size: 12px; text-transform: uppercase; }
@media (max-width: 900px) {
  .shell { grid-template-columns: 1fr; }
  .side { border-bottom: 1px solid var(--line); border-right: 0; }
  .main { padding: 20px; }
  .topbar { align-items: stretch; flex-direction: column; }
  .token,
  .form-grid,
  .grid { grid-template-columns: 1fr; }
  .span-2 { grid-column: auto; }
}
"#;

const ADMIN_JS: &str = r#"
const tokenInput = document.querySelector("[data-token]");
const saveToken = document.querySelector("[data-save-token]");
const tokenStatus = document.querySelector("[data-token-status]");
const storedToken = sessionStorage.getItem("rnovemail.adminToken") || "";

if (tokenInput) {
  tokenInput.value = storedToken;
}

if (saveToken) {
  saveToken.addEventListener("click", () => {
    sessionStorage.setItem("rnovemail.adminToken", tokenInput.value.trim());
    tokenStatus.textContent = "Token saved";
    tokenStatus.dataset.state = "ok";
  });
}

document.querySelectorAll("[data-api-form]").forEach((form) => {
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const status = form.querySelector("[data-form-status]");
    status.textContent = "Saving";
    status.dataset.state = "";

    try {
      const response = await fetch(form.dataset.endpoint, {
        method: "POST",
        headers: {
          "authorization": `Bearer ${tokenInput.value.trim()}`,
          "content-type": "application/json"
        },
        body: JSON.stringify(formPayload(form))
      });
      const text = await response.text();
      status.textContent = response.ok ? "Saved" : text || response.statusText;
      status.dataset.state = response.ok ? "ok" : "error";
      if (response.ok) form.reset();
    } catch (error) {
      status.textContent = "Request failed";
      status.dataset.state = "error";
    }
  });
});

function formPayload(form) {
  const payload = {};
  new FormData(form).forEach((value, key) => {
    const text = String(value).trim();
    if (!text) return;
    payload[key] = key === "domains" || key === "roles" ? splitList(text) : text;
  });
  return payload;
}

function splitList(value) {
  return value.split(",").map((item) => item.trim()).filter(Boolean);
}
"#;

pub fn login_page() -> Markup {
    admin_layout(
        "Sign In",
        html! {
            section class="panel accent" {
                h2 { "Admin Token" }
                p { "Enter the assigned administrator token for this browser session." }
                div class="form-grid" {
                    label class="span-2" {
                        "Token"
                        input type="password" data-token="" autocomplete="current-password";
                    }
                    div class="span-2" {
                        button type="button" data-save-token="" { "Save Token" }
                    }
                    p class="status span-2" data-token-status="" {}
                }
            }
        },
    )
}

pub fn dashboard_page() -> Markup {
    admin_layout(
        "Operations",
        html! {
            section class="grid" {
                (summary_card("Users", "/admin/users", "Assigned accounts"))
                (summary_card("Domains", "/admin/domains", "Verified mail domains"))
                (summary_card("Providers", "/admin/providers", "Delivery platforms"))
                (summary_card("Mailboxes", "/admin/mailboxes", "Managed addresses"))
            }
        },
    )
}

pub fn users_page() -> Markup {
    admin_layout(
        "Users",
        html! {
            section class="panel accent" {
                h2 { "Assign User" }
                form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/users" {
                    label {
                        "Display Name"
                        input name="display_name" autocomplete="name" required;
                    }
                    label {
                        "Primary Email"
                        input name="email" type="email" autocomplete="email" required;
                    }
                    label class="span-2" {
                        "Roles"
                        input name="roles" value="Admin";
                    }
                    div class="span-2" {
                        button type="submit" { "Create User" }
                    }
                    p class="status span-2" data-form-status="" {}
                }
            }
        },
    )
}

pub fn domains_page() -> Markup {
    admin_layout(
        "Domains",
        html! {
            section class="panel accent" {
                h2 { "Add Domain" }
                form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/domains" {
                    label class="span-2" {
                        "Domain"
                        input name="domain" placeholder="example.com" required;
                    }
                    div class="span-2" {
                        button type="submit" { "Create Domain" }
                    }
                    p class="status span-2" data-form-status="" {}
                }
            }
        },
    )
}

pub fn providers_page() -> Markup {
    admin_layout(
        "Providers",
        html! {
            section class="panel accent" {
                h2 { "Add Provider" }
                form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/provider-accounts" {
                    label {
                        "Name"
                        input name="name" placeholder="resend-prod" required;
                    }
                    label {
                        "Provider"
                        select name="provider_type" {
                            option value="resend" { "Resend" }
                        }
                    }
                    label class="span-2" {
                        "Domains"
                        input name="domains" placeholder="example.com, alerts.example.com" required;
                    }
                    label class="span-2" {
                        "Webhook Secret"
                        input name="webhook_secret" type="password" autocomplete="off";
                    }
                    div class="span-2" {
                        button type="submit" { "Create Provider" }
                    }
                    p class="status span-2" data-form-status="" {}
                }
            }
        },
    )
}

pub fn mailboxes_page() -> Markup {
    admin_layout(
        "Mailboxes",
        html! {
            section class="panel accent" {
                h2 { "Assign Mailbox" }
                form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/mailboxes" {
                    label {
                        "Owner Email"
                        input name="owner_email" type="email" required;
                    }
                    label {
                        "Mailbox Email"
                        input name="mailbox_email" type="email" required;
                    }
                    div class="span-2" {
                        button type="submit" { "Create Mailbox" }
                    }
                    p class="status span-2" data-form-status="" {}
                }
            }
        },
    )
}

pub fn audit_page() -> Markup {
    admin_layout(
        "Audit",
        html! {
            section class="panel accent" {
                h2 { "Audit Trail" }
                table class="table" {
                    thead {
                        tr {
                            th { "Source" }
                            th { "Scope" }
                            th { "Retention" }
                        }
                    }
                    tbody {
                        tr {
                            td { "Admin API" }
                            td { "User, domain, provider, and mailbox mutations" }
                            td { "Stored in the local .rnmdb database" }
                        }
                    }
                }
            }
        },
    )
}

fn admin_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "RNovEmail " (title) }
                style { (PreEscaped(ADMIN_CSS)) }
            }
            body {
                a class="skip" href="#content" { "Skip to content" }
                div class="shell" {
                    aside class="side" {
                        div class="brand" {
                            span class="mark" aria-hidden="true" { "R" }
                            span { "RNovEmail" }
                        }
                        nav class="nav" aria-label="Admin navigation" {
                            a href="/admin" { "Operations" }
                            a href="/admin/users" { "Users" }
                            a href="/admin/domains" { "Domains" }
                            a href="/admin/providers" { "Providers" }
                            a href="/admin/mailboxes" { "Mailboxes" }
                            a href="/admin/audit" { "Audit" }
                        }
                    }
                    main id="content" class="main" {
                        div class="topbar" {
                            div {
                                p class="eyebrow" { "Admin Console" }
                                h1 { (title) }
                            }
                            (token_panel())
                        }
                        div class="stack" {
                            (content)
                        }
                    }
                }
                script { (PreEscaped(ADMIN_JS)) }
            }
        }
    }
}

fn token_panel() -> Markup {
    html! {
        div class="token" {
            label {
                "Admin Token"
                input type="password" data-token="" autocomplete="current-password";
            }
            button type="button" data-save-token="" { "Save" }
            p class="status span-2" data-token-status="" {}
        }
    }
}

fn summary_card(title: &str, href: &str, detail: &str) -> Markup {
    html! {
        a class="panel" href=(href) {
            h2 { (title) }
            p { (detail) }
        }
    }
}
