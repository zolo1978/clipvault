use std::sync::LazyLock;
use regex::Regex;
use crate::models::SensitiveKind;

struct Patterns {
    private_key: Regex,
    aws_access_key: Regex,
    github_token: Regex,
    stripe_key: Regex,
    openai_key: Regex,
    anthropic_key: Regex,
    google_api_key: Regex,
    slack_token: Regex,
    jwt: Regex,
    bearer_token: Regex,
    connection_string: Regex,
}

static PATTERNS: LazyLock<Patterns> = LazyLock::new(|| Patterns {
    private_key: Regex::new(
        r"-----BEGIN\s+(?:RSA\s+|EC\s+|DSA\s+|OPENSSH\s+|ENCRYPTED\s+)?PRIVATE\s+KEY-----"
    ).unwrap(),

    aws_access_key: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),

    github_token: Regex::new(r"\bgh[pousr]_[A-Za-z0-9_]{36,}\b").unwrap(),

    stripe_key: Regex::new(r"\b[sr]k_(test|live)_[A-Za-z0-9]{24,}\b").unwrap(),

    openai_key: Regex::new(
        r"\bsk-[A-Za-z0-9]{20,}T3BlbkFJ[A-Za-z0-9]+|\bsk-proj-[A-Za-z0-9_-]+"
    ).unwrap(),

    anthropic_key: Regex::new(r"\bsk-ant-[A-Za-z0-9_-]{20,}").unwrap(),

    google_api_key: Regex::new(r"\bAIza[A-Za-z0-9_-]{35}\b").unwrap(),

    slack_token: Regex::new(
        r"\bxox[pboa]-[0-9]{10,}-[0-9]{10,}-[0-9]{10,}-[a-z0-9]{24,}"
    ).unwrap(),

    jwt: Regex::new(
        r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"
    ).unwrap(),

    bearer_token: Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),

    connection_string: Regex::new(
        r"(?i)(?:postgres(?:ql)?|mongodb(?:\+srv)?|mysql|redis)://[^\s:@]{3,20}:[^\s:@]{3,20}@[^\s]+"
    ).unwrap(),
});

/// Scan text for sensitive patterns. Returns the first match in priority order.
pub fn scan_text(text: &str) -> Option<SensitiveKind> {
    let p = &*PATTERNS;

    if p.private_key.is_match(text) {
        return Some(SensitiveKind::PrivateKey);
    }
    if p.aws_access_key.is_match(text) {
        return Some(SensitiveKind::ApiKey("AWS".into()));
    }
    if p.github_token.is_match(text) {
        return Some(SensitiveKind::ApiKey("GitHub".into()));
    }
    if p.stripe_key.is_match(text) {
        return Some(SensitiveKind::ApiKey("Stripe".into()));
    }
    if p.openai_key.is_match(text) {
        return Some(SensitiveKind::ApiKey("OpenAI".into()));
    }
    if p.anthropic_key.is_match(text) {
        return Some(SensitiveKind::ApiKey("Anthropic".into()));
    }
    if p.google_api_key.is_match(text) {
        return Some(SensitiveKind::ApiKey("Google".into()));
    }
    if p.slack_token.is_match(text) {
        return Some(SensitiveKind::ApiKey("Slack".into()));
    }
    if p.jwt.is_match(text) {
        return Some(SensitiveKind::Jwt);
    }
    if p.bearer_token.is_match(text) {
        return Some(SensitiveKind::BearerToken);
    }
    if p.connection_string.is_match(text) {
        return Some(SensitiveKind::ConnectionString);
    }
    None
}
