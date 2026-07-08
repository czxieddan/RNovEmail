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
  background: var(--blue);
  border-radius: 8px;
  color: #fff;
  display: inline-grid;
  height: 34px;
  place-items: center;
  width: 34px;
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
select {
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
select:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.16);
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
@media (max-width: 900px) {
  .shell { grid-template-columns: 1fr; }
  .side { border-bottom: 1px solid var(--line); border-right: 0; }
  .main { padding: 20px; }
  .topbar { align-items: stretch; flex-direction: column; }
  .actions,
  .form-grid,
  .grid,
  .row-form { grid-template-columns: 1fr; justify-content: stretch; }
  .span-2 { grid-column: auto; }
}
"#;

const ADMIN_JS: &str = r#"
document.querySelectorAll("[data-api-form]").forEach((form) => {
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const status = form.querySelector("[data-form-status]");
    status.textContent = "Saving";
    status.dataset.state = "";

    try {
      const response = await fetch(form.dataset.endpoint, {
        method: form.dataset.method || "POST",
        credentials: "same-origin",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(formPayload(form))
      });
      const text = await response.text();
      status.textContent = response.ok ? "Saved" : text || response.statusText;
      status.dataset.state = response.ok ? "ok" : "error";
      if (response.ok && form.dataset.reset !== "false") form.reset();
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
    if (key === "domains" || key === "roles") payload[key] = splitList(text);
    else if (key === "enabled" || key.endsWith("_enabled")) payload[key] = text === "true";
    else payload[key] = text;
  });
  return payload;
}

function splitList(value) {
  return value.split(",").map((item) => item.trim()).filter(Boolean);
}
"#;

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
    app_layout(
        ctx,
        text(ctx.lang, Text::Portal),
        false,
        html! {
            section class="panel accent" {
                h2 { (data.email) }
                div class="table-wrap" {
                    table class="table" {
                        thead {
                            tr {
                                th { (text(ctx.lang, Text::Mailboxes)) }
                                th { (text(ctx.lang, Text::Status)) }
                                th { (text(ctx.lang, Text::Inbound)) }
                                th { (text(ctx.lang, Text::Outbound)) }
                            }
                        }
                        tbody {
                            @for mailbox in &data.mailboxes {
                                tr {
                                    td { (&mailbox.email) }
                                    td { (&mailbox.status) }
                                    td { (bool_text(ctx.lang, mailbox.inbound_enabled)) }
                                    td { (bool_text(ctx.lang, mailbox.outbound_enabled)) }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
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
            form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/users" {
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
            form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/domains" {
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
                            tr { td { (&domain.domain) } }
                        }
                    }
                }
            }
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
            form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/provider-accounts" {
                (field(ctx, Text::Name, "name", "text", ""))
                label { (text(ctx.lang, Text::Providers)) select name="provider_type" { option value="resend" { "Resend" } } }
                (field(ctx, Text::Domains, "domains", "text", ""))
                (field(ctx, Text::WebhookSecret, "webhook_secret", "password", ""))
                (submit_row(ctx, Text::Create))
            }
        }
    }
}

fn provider_update_form(ctx: &PageContext, provider: &crate::ProviderRow) -> Markup {
    html! {
        form class="row-form" data-api-form="" data-reset="false" data-method="PATCH" data-endpoint=(format!("/api/v1/admin/provider-accounts/{}", provider.id)) {
            input name="name" value=(&provider.name);
            input name="domains" value=(&provider.domains);
            select name="enabled" {
                option value="true" selected[provider.enabled] { (text(ctx.lang, Text::Enabled)) }
                option value="false" selected[!provider.enabled] { (text(ctx.lang, Text::Disabled)) }
            }
            button type="submit" { (text(ctx.lang, Text::Save)) }
            p class="status" data-form-status="" {}
        }
    }
}

fn mailboxes(ctx: &PageContext, data: &AdminData) -> Markup {
    html! {
        section class="panel accent" {
            h2 { (text(ctx.lang, Text::Add)) " " (text(ctx.lang, Text::Mailboxes)) }
            form class="form-grid" data-api-form="" data-endpoint="/api/v1/admin/mailboxes" {
                (field(ctx, Text::Email, "owner_email", "email", ""))
                (field(ctx, Text::Mailboxes, "mailbox_email", "email", ""))
                (submit_row(ctx, Text::Create))
            }
        }
        section class="panel" {
            h2 { (text(ctx.lang, Text::Mailboxes)) }
            div class="table-wrap" {
                table class="table" {
                    tbody {
                        @for mailbox in &data.mailboxes {
                            tr { td { (mailbox_update_form(ctx, mailbox)) } }
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

fn app_layout(ctx: &PageContext, title: &str, admin: bool, content: Markup) -> Markup {
    base_page(
        ctx,
        title,
        html! {
            a class="skip" href="#content" { "Skip" }
            div class="shell" {
                aside class="side" {
                    div class="brand" {
                        span class="mark" aria-hidden="true" { "R" }
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
                            (language_links(ctx))
                            (theme_link(ctx))
                            form method="post" action="/logout" {
                                button class="secondary" type="submit" { (text(ctx.lang, Text::Logout)) }
                            }
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
                title { "RNovEmail " (title) }
                style { (PreEscaped(ADMIN_CSS)) }
            }
            body { (body) }
        }
    }
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

fn bool_text(lang: Lang, value: bool) -> &'static str {
    match value {
        true => text(lang, Text::Enabled),
        false => text(lang, Text::Disabled),
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
