use maud::{DOCTYPE, Markup, html};

pub fn login_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head { title { "RNovEmail Admin" } }
            body {
                main {
                    h1 { "RNovEmail Admin" }
                    form method="post" action="/admin/session" {
                        input type="password" name="token" autocomplete="current-password";
                        button type="submit" { "Sign in" }
                    }
                }
            }
        }
    }
}

pub fn dashboard_page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head { title { "RNovEmail Operations" } }
            body {
                main {
                    h1 { "Operations" }
                    nav {
                        a href="/admin/domains" { "Domains" }
                        a href="/admin/providers" { "Providers" }
                        a href="/admin/mailboxes" { "Mailboxes" }
                        a href="/admin/audit" { "Audit" }
                    }
                }
            }
        }
    }
}
