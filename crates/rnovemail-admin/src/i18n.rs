#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lang {
    Zh,
    Ja,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Text {
    Add,
    AdminConsole,
    ApiKey,
    Configured,
    Audit,
    AuditRetention,
    AuditScope,
    AuditSource,
    Create,
    Dark,
    Delete,
    Disabled,
    Domains,
    Email,
    Enabled,
    Inbound,
    LanguageZh,
    LanguageJa,
    Light,
    Login,
    Logout,
    Mailboxes,
    Name,
    Operations,
    Outbound,
    Password,
    Portal,
    Providers,
    Roles,
    Save,
    Status,
    Theme,
    Users,
    WebhookSecret,
}

impl Lang {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some(value) if value.eq_ignore_ascii_case("ja") => Self::Ja,
            _ => Self::Zh,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::Ja => "ja",
        }
    }
}

pub fn text(lang: Lang, key: Text) -> &'static str {
    match lang {
        Lang::Zh => zh(key),
        Lang::Ja => ja(key),
    }
}

fn zh(key: Text) -> &'static str {
    match key {
        Text::Add => "添加",
        Text::AdminConsole => "管理控制台",
        Text::ApiKey => "API Key",
        Text::Configured => "已配置",
        Text::Audit => "审计",
        Text::AuditRetention => "保存在本地 .rnmdb 数据库",
        Text::AuditScope => "用户、域名、服务商和邮箱变更",
        Text::AuditSource => "管理操作",
        Text::Create => "创建",
        Text::Dark => "暗色",
        Text::Delete => "删除",
        Text::Disabled => "停用",
        Text::Domains => "域名",
        Text::Email => "邮箱",
        Text::Enabled => "启用",
        Text::Inbound => "入站",
        Text::LanguageZh => "中文",
        Text::LanguageJa => "日本語",
        Text::Light => "亮色",
        Text::Login => "登录",
        Text::Logout => "退出登录",
        Text::Mailboxes => "邮箱",
        Text::Name => "名称",
        Text::Operations => "概览",
        Text::Outbound => "出站",
        Text::Password => "密钥",
        Text::Portal => "用户门户",
        Text::Providers => "服务商",
        Text::Roles => "角色",
        Text::Save => "保存",
        Text::Status => "状态",
        Text::Theme => "主题",
        Text::Users => "用户",
        Text::WebhookSecret => "Webhook 密钥",
    }
}

fn ja(key: Text) -> &'static str {
    match key {
        Text::Add => "追加",
        Text::AdminConsole => "管理コンソール",
        Text::ApiKey => "API Key",
        Text::Configured => "設定済み",
        Text::Audit => "監査",
        Text::AuditRetention => "ローカル .rnmdb データベースに保存",
        Text::AuditScope => "ユーザー、ドメイン、プロバイダー、メールボックスの変更",
        Text::AuditSource => "管理操作",
        Text::Create => "作成",
        Text::Dark => "ダーク",
        Text::Delete => "削除",
        Text::Disabled => "無効",
        Text::Domains => "ドメイン",
        Text::Email => "メール",
        Text::Enabled => "有効",
        Text::Inbound => "受信",
        Text::LanguageZh => "中文",
        Text::LanguageJa => "日本語",
        Text::Light => "ライト",
        Text::Login => "ログイン",
        Text::Logout => "ログアウト",
        Text::Mailboxes => "メールボックス",
        Text::Name => "名前",
        Text::Operations => "概要",
        Text::Outbound => "送信",
        Text::Password => "シークレット",
        Text::Portal => "ユーザーポータル",
        Text::Providers => "プロバイダー",
        Text::Roles => "ロール",
        Text::Save => "保存",
        Text::Status => "状態",
        Text::Theme => "テーマ",
        Text::Users => "ユーザー",
        Text::WebhookSecret => "Webhook シークレット",
    }
}
