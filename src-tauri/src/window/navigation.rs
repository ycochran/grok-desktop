use std::{
    collections::VecDeque,
    io::Write,
    process::{Command, Stdio},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::persistence::state::PersistedStateStore;
use url::Url;

const MAX_HINT_RECORDS: usize = 24;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationHintRecord {
    pub timestamp: u64,
    pub source: String,
    pub target: String,
    pub classification: String,
    pub auth_flow: bool,
    pub note: String,
}

#[derive(Default)]
pub struct NavigationDiagnosticsStore {
    inner: Mutex<VecDeque<NavigationHintRecord>>,
}

impl NavigationDiagnosticsStore {
    pub fn push(&self, entry: NavigationHintRecord) {
        let mut guard = self.inner.lock().expect("navigation diagnostics mutex poisoned");
        guard.push_front(entry);
        while guard.len() > MAX_HINT_RECORDS {
            let _ = guard.pop_back();
        }
    }

    pub fn recent(&self) -> Vec<NavigationHintRecord> {
        self.inner
            .lock()
            .expect("navigation diagnostics mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.inner
            .lock()
            .expect("navigation diagnostics mutex poisoned")
            .clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDisposition {
    Embed,
    ExternalBrowser,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationDecision {
    pub disposition: NavigationDisposition,
    pub is_auth_flow: bool,
    pub message: &'static str,
}

pub fn classify(url: &Url) -> NavigationDecision {
    classify_with_auth_context(url, false)
}

pub fn classify_with_auth_context(url: &Url, auth_flow_active: bool) -> NavigationDecision {
    match url.scheme() {
        "http" | "https" => classify_http_url(url, auth_flow_active),
        "about" => NavigationDecision {
            disposition: NavigationDisposition::Embed,
            is_auth_flow: false,
            message: "Embedded Grok is initializing.",
        },
        "javascript" | "data" | "file" | "tauri" => NavigationDecision {
            disposition: NavigationDisposition::Deny,
            is_auth_flow: false,
            message: "That URL scheme is not allowed in the embedded Grok surface.",
        },
        _ => NavigationDecision {
            disposition: NavigationDisposition::ExternalBrowser,
            is_auth_flow: false,
            message: "The requested destination was handed off to the system browser.",
        },
    }
}

pub fn is_embeddable_content(url: &Url) -> bool {
    let decision = classify(url);
    decision.disposition == NavigationDisposition::Embed && !decision.is_auth_flow
}

pub fn is_auth_completion_landing(url: &Url, decision: NavigationDecision) -> bool {
    if decision.disposition != NavigationDisposition::Embed || decision.is_auth_flow {
        return false;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    if host_matches(host, &["grok.com"]) {
        return true;
    }

    if host_matches(host, &["x.ai"]) {
        let path = url.path();
        return path == "/"
            || path.starts_with("/grok")
            || path.starts_with("/i/grok")
            || path.starts_with("/chat")
            || path.starts_with("/c/");
    }

    host_matches(host, &["x.com", "twitter.com"]) && is_x_grok_path(url)
}

pub fn should_trigger_post_auth_reload(url: &Url, decision: NavigationDecision) -> bool {
    if decision.disposition != NavigationDisposition::Embed || !decision.is_auth_flow {
        return false;
    }

    if !is_auth_completion_related(url) {
        return false;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    host_matches(
        host,
        &[
            "accounts.x.ai",
            "auth.grokipedia.com",
            "auth.grokusercontent.com",
        ],
    )
}

pub fn log_decision(
    app: &AppHandle,
    context: &str,
    url: &Url,
    decision: NavigationDecision,
    auth_flow_active: bool,
) {
    record_decision(app, context, url, decision, auth_flow_active);

    let store = app.state::<PersistedStateStore>();
    let logging_enabled = {
        let guard = store.inner.lock().expect("state store mutex poisoned");
        guard.settings.navigation_debug_logging_enabled()
    };

    if !logging_enabled {
        return;
    }

    eprintln!(
        "[grok-nav] context={context} disposition={:?} auth={} auth_active={} url={}",
        decision.disposition,
        decision.is_auth_flow,
        auth_flow_active,
        summarize_url(url)
    );
}

pub fn record_auth_landing(app: &AppHandle, url: &Url, note: &'static str) {
    let store = app.state::<NavigationDiagnosticsStore>();
    store.push(NavigationHintRecord {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        source: String::from("auth-landing"),
        target: summarize_url(url),
        classification: String::from("Embed"),
        auth_flow: false,
        note: note.to_string(),
    });

    let settings_store = app.state::<PersistedStateStore>();
    let logging_enabled = {
        let guard = settings_store
            .inner
            .lock()
            .expect("state store mutex poisoned");
        guard.settings.navigation_debug_logging_enabled()
    };

    if logging_enabled {
        eprintln!("[grok-nav] context=auth_landing url={} note={}", summarize_url(url), note);
    }
}

pub fn format_hint_records(records: &[NavigationHintRecord]) -> String {
    if records.is_empty() {
        return String::from("No recent embedded navigation hints.");
    }

    records
        .iter()
        .map(|record| {
            format!(
                "{} | {} | {} | {} | auth={} | {}",
                record.timestamp,
                record.source,
                record.classification,
                record.target,
                record.auth_flow,
                record.note
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn copy_to_clipboard(contents: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to launch pbcopy: {error}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(contents.as_bytes())
            .map_err(|error| format!("Failed to write diagnostics to the clipboard: {error}"))?;
    }

    let status = child
        .wait()
        .map_err(|error| format!("Failed waiting for pbcopy: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(String::from("pbcopy exited unsuccessfully"))
    }
}

fn classify_http_url(url: &Url, auth_flow_active: bool) -> NavigationDecision {
    let Some(host) = url.host_str() else {
        return NavigationDecision {
            disposition: NavigationDisposition::Deny,
            is_auth_flow: false,
            message: "The embedded Grok surface rejected a malformed URL.",
        };
    };

    if auth_flow_active && is_known_auth_chain_host(host) {
        let auth_related = is_auth_completion_related(url);
        if is_auth_bridge_completion_host(host) && !auth_related {
            return NavigationDecision {
                disposition: NavigationDisposition::ExternalBrowser,
                is_auth_flow: false,
                message: "Unrelated bridge-host destinations open in the default browser.",
            };
        }

        return NavigationDecision {
            disposition: NavigationDisposition::Embed,
            is_auth_flow: auth_related,
            message: if host_matches(host, &["grok.com"]) && !auth_related {
                "Embedded Grok finished the sign-in flow and returned to the app."
            } else if auth_related {
                "Continue sign-in completion inside the embedded Grok surface."
            } else {
                "Auth-active mode kept this step inside the embedded Grok surface."
            },
        };
    }

    if host_matches(host, &["grok.com"]) {
        let is_auth_flow = is_auth_path(url) || is_sign_in_entry_path(url) || is_auth_completion_path(url);
        return NavigationDecision {
            disposition: NavigationDisposition::Embed,
            is_auth_flow,
            message: if is_auth_flow {
                "Continue authentication inside the embedded Grok surface."
            } else if auth_flow_active {
                "Embedded Grok finished the sign-in flow and returned to the app."
            } else {
                "Embedded Grok is ready."
            },
        };
    }

    if host_matches(host, &["x.ai"]) {
        if is_xai_allowed_path(url) {
            let is_auth_flow = is_xai_auth_path(url);
            return NavigationDecision {
                disposition: NavigationDisposition::Embed,
                is_auth_flow,
                message: if is_auth_flow {
                    "Continue xAI authentication inside the embedded Grok surface."
                } else if auth_flow_active {
                    "Complete xAI sign-in inside the embedded Grok surface."
                } else {
                    "Embedded Grok is ready."
                },
            };
        }

        return NavigationDecision {
            disposition: NavigationDisposition::ExternalBrowser,
            is_auth_flow: false,
            message: "Non-Grok xAI pages open in the default browser.",
        };
    }

    if host_matches(host, &["x.com", "twitter.com"]) {
        if is_x_allowed_path(url) {
            let is_auth_flow = is_x_auth_path(url);
            return NavigationDecision {
                disposition: NavigationDisposition::Embed,
                is_auth_flow,
                message: if is_auth_flow {
                    "Complete X authentication inside the embedded Grok surface."
                } else if auth_flow_active {
                    "Finish returning from X consent inside the embedded Grok surface."
                } else if is_x_grok_path(url) {
                    "Embedded Grok is ready."
                } else {
                    "Embedded Grok content stays in the child webview."
                },
            };
        }

        return NavigationDecision {
            disposition: NavigationDisposition::ExternalBrowser,
            is_auth_flow: false,
            message: "Non-Grok X pages open in the default browser.",
        };
    }

    if host_matches(
        host,
        &[
            "accounts.google.com",
            "appleid.apple.com",
            "login.apple.com",
            "signin.apple.com",
        ],
    ) {
        return NavigationDecision {
            disposition: NavigationDisposition::Embed,
            is_auth_flow: true,
            message: "Continue provider authentication inside the embedded Grok surface.",
        };
    }

    if host_matches(host, &["auth.x.com", "auth.twitter.com"]) {
        return NavigationDecision {
            disposition: NavigationDisposition::Embed,
            is_auth_flow: true,
            message: "Continue X authentication inside the embedded Grok surface.",
        };
    }

    if host_matches(host, &["challenges.cloudflare.com"]) {
        if is_cloudflare_auth_path(url) {
            return NavigationDecision {
                disposition: NavigationDisposition::Embed,
                is_auth_flow: true,
                message: "Complete the inline authentication challenge inside the embedded Grok surface.",
            };
        }

        return NavigationDecision {
            disposition: NavigationDisposition::ExternalBrowser,
            is_auth_flow: false,
            message: "Unrelated Cloudflare pages open in the default browser.",
        };
    }

    NavigationDecision {
        disposition: NavigationDisposition::ExternalBrowser,
        is_auth_flow: false,
        message: "That destination is outside the embedded Grok allowlist and was opened in the browser.",
    }
}

fn host_matches(host: &str, allowed_hosts: &[&str]) -> bool {
    allowed_hosts
        .iter()
        .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
}

fn is_auth_path(url: &Url) -> bool {
    let path = url.path();
    path.starts_with("/auth")
        || path.starts_with("/api/auth")
        || path.starts_with("/login")
        || path.starts_with("/signup")
        || path.starts_with("/sign-up")
        || path.starts_with("/signin")
        || path.starts_with("/sign-in")
        || path.starts_with("/oauth")
        || path.starts_with("/session")
}

fn is_auth_completion_path(url: &Url) -> bool {
    let path = url.path();
    path.starts_with("/callback")
        || path.starts_with("/auth/callback")
        || path.starts_with("/oauth/callback")
        || path.starts_with("/set-cookie")
        || path.starts_with("/set-session")
        || path.starts_with("/success")
        || path.starts_with("/complete")
        || path.starts_with("/continue")
        || path.starts_with("/verify")
        || path.starts_with("/exchange-token")
}

fn is_sign_in_entry_path(url: &Url) -> bool {
    let path = url.path();
    path.starts_with("/sign-in")
        || path.starts_with("/sign-up")
        || path.starts_with("/callback")
        || path.starts_with("/auth/callback")
}

fn is_x_allowed_path(url: &Url) -> bool {
    is_x_grok_path(url)
        || url.path().starts_with("/i/flow")
        || url.path().starts_with("/i/oauth2")
        || url.path().starts_with("/oauth")
        || url.path().starts_with("/login")
        || url.path().starts_with("/signup")
        || url.path().starts_with("/logout")
        || url.path().starts_with("/account/access")
        || url.path().starts_with("/account/login_challenge")
        || url.path().starts_with("/account/begin_password_reset")
        || url.path().starts_with("/account/confirm_email")
        || url.path().starts_with("/account/authentication_challenge")
        || url.path().starts_with("/account/duplication")
}

fn is_x_grok_path(url: &Url) -> bool {
    let path = url.path();
    path == "/i/grok" || path.starts_with("/i/grok/")
}

fn is_x_auth_path(url: &Url) -> bool {
    !is_x_grok_path(url)
}

fn is_xai_allowed_path(url: &Url) -> bool {
    let path = url.path();
    path == "/"
        || path.starts_with("/grok")
        || path.starts_with("/i/grok")
        || path.starts_with("/chat")
        || path.starts_with("/c/")
        || path.starts_with("/auth")
        || path.starts_with("/api/auth")
        || path.starts_with("/login")
        || path.starts_with("/signup")
        || path.starts_with("/sign-up")
        || path.starts_with("/signin")
        || path.starts_with("/sign-in")
        || path.starts_with("/oauth")
        || path.starts_with("/authorize")
        || path.starts_with("/exchange-token")
        || path.starts_with("/session")
        || path.starts_with("/callback")
        || path.starts_with("/auth/callback")
        || path.starts_with("/oauth/callback")
        || path.starts_with("/set-cookie")
        || path.starts_with("/set-session")
        || path.starts_with("/success")
        || path.starts_with("/complete")
        || path.starts_with("/continue")
        || path.starts_with("/verify")
        || path.starts_with("/check-login")
        || path.starts_with("/account/access")
        || path.starts_with("/account/login_challenge")
        || path.starts_with("/account/authentication_challenge")
        || path.starts_with("/account/duplication")
}

fn is_xai_auth_path(url: &Url) -> bool {
    is_auth_path(url)
        || is_auth_completion_path(url)
        || url.path().starts_with("/authorize")
        || url.path().starts_with("/exchange-token")
        || url.path().starts_with("/check-login")
        || url.path().starts_with("/account/access")
        || url.path().starts_with("/account/login_challenge")
}

fn is_cloudflare_auth_path(url: &Url) -> bool {
    url.path().starts_with("/cdn-cgi/")
}

fn is_known_auth_chain_host(host: &str) -> bool {
    host_matches(host, &["grok.com"])
        || host_matches(host, &["x.ai"])
        || host_matches(host, &["x.com", "twitter.com"])
        || host_matches(host, &["auth.x.com", "auth.twitter.com"])
        || host_matches(host, &["auth.grokipedia.com"])
        || host_matches(host, &["auth.grokusercontent.com"])
        || host_matches(host, &["accounts.google.com"])
        || host_matches(host, &["appleid.apple.com", "login.apple.com", "signin.apple.com"])
        || host_matches(host, &["challenges.cloudflare.com"])
}

fn is_auth_bridge_completion_host(host: &str) -> bool {
    host_matches(host, &["auth.grokipedia.com", "auth.grokusercontent.com"])
}

fn is_auth_completion_related(url: &Url) -> bool {
    if is_auth_path(url) || is_sign_in_entry_path(url) || is_auth_completion_path(url) {
        return true;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    if host_matches(host, &["accounts.google.com", "appleid.apple.com", "login.apple.com", "signin.apple.com"]) {
        return true;
    }

    if host_matches(host, &["auth.x.com", "auth.twitter.com"]) {
        return true;
    }

    if host_matches(host, &["challenges.cloudflare.com"]) {
        return is_cloudflare_auth_path(url);
    }

    false
}

fn record_decision(
    app: &AppHandle,
    context: &str,
    url: &Url,
    decision: NavigationDecision,
    auth_flow_active: bool,
) {
    let store = app.state::<NavigationDiagnosticsStore>();
    store.push(NavigationHintRecord {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
        source: normalize_source(context).to_string(),
        target: summarize_url(url),
        classification: match decision.disposition {
            NavigationDisposition::Embed => String::from("Embed"),
            NavigationDisposition::ExternalBrowser => String::from("ExternalBrowser"),
            NavigationDisposition::Deny => String::from("Deny"),
        },
        auth_flow: decision.is_auth_flow,
        note: format!(
            "{}{}",
            if auth_flow_active { "[auth-active] " } else { "" },
            decision.message
        ),
    });
}

fn normalize_source(context: &str) -> &str {
    match context {
        "page_load" => "redirect",
        "popup_request" => "popup",
        _ => "navigation",
    }
}

fn summarize_url(url: &Url) -> String {
    let host = url.host_str().unwrap_or(url.scheme());
    let path = sanitize_path(url.path());
    format!("{host}{path}")
}

fn sanitize_path(path: &str) -> String {
    let mut sanitized = String::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()).take(4) {
        sanitized.push('/');
        sanitized.push_str(if looks_sensitive(segment) { ":id" } else { segment });
    }

    if sanitized.is_empty() {
        sanitized.push('/');
    }

    sanitized
}

fn looks_sensitive(segment: &str) -> bool {
    segment.len() > 32
        || segment.contains('.')
        || segment.chars().any(|character| character.is_ascii_hexdigit()) && segment.len() > 12
}

#[cfg(test)]
mod tests {
    use super::{
        classify, classify_with_auth_context, should_trigger_post_auth_reload,
        NavigationDisposition,
    };
    use url::Url;

    #[test]
    fn embeds_known_xai_auth_path() {
        let decision = classify(&Url::parse("https://x.ai/account/access").expect("url should parse"));
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_accounts_xai_check_login_path() {
        let decision = classify(
            &Url::parse("https://accounts.x.ai/check-login").expect("url should parse"),
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_accounts_xai_sign_in_path() {
        let decision =
            classify(&Url::parse("https://accounts.x.ai/sign-in").expect("url should parse"));
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_accounts_xai_exchange_token_path() {
        let decision = classify(
            &Url::parse("https://accounts.x.ai/exchange-token/?state=redacted")
                .expect("url should parse"),
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_cloudflare_inline_auth_challenge() {
        let decision = classify(
            &Url::parse("https://challenges.cloudflare.com/cdn-cgi/challenge-platform/h/g/test")
                .expect("url should parse"),
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_accounts_xai_set_cookie_path() {
        let decision = classify(
            &Url::parse("https://accounts.x.ai/set-cookie/session")
                .expect("url should parse"),
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_accounts_xai_success_path() {
        let decision = classify(
            &Url::parse("https://accounts.x.ai/success/redirect")
                .expect("url should parse"),
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_accounts_xai_bridge_path_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://accounts.x.ai/post-consent-bridge")
                .expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(!decision.is_auth_flow);
    }

    #[test]
    fn keeps_accounts_xai_bridge_path_external_when_auth_is_inactive() {
        let decision = classify_with_auth_context(
            &Url::parse("https://accounts.x.ai/post-consent-bridge")
                .expect("url should parse"),
            false,
        );
        assert_eq!(decision.disposition, NavigationDisposition::ExternalBrowser);
        assert!(!decision.is_auth_flow);
    }

    #[test]
    fn keeps_known_auth_chain_host_embedded_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://accounts.x.ai/account/recovery-step")
                .expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(!decision.is_auth_flow);
    }

    #[test]
    fn keeps_unrelated_host_external_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://example.com/landing").expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::ExternalBrowser);
        assert!(!decision.is_auth_flow);
    }

    #[test]
    fn embeds_grokipedia_set_cookie_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://auth.grokipedia.com/set-cookie").expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn keeps_unrelated_grokipedia_path_external_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://auth.grokipedia.com/docs").expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::ExternalBrowser);
        assert!(!decision.is_auth_flow);
    }

    #[test]
    fn embeds_grokusercontent_set_cookie_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://auth.grokusercontent.com/set-cookie")
                .expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn embeds_grokusercontent_callback_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://auth.grokusercontent.com/callback/complete")
                .expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::Embed);
        assert!(decision.is_auth_flow);
    }

    #[test]
    fn keeps_unrelated_grokusercontent_path_external_when_auth_is_active() {
        let decision = classify_with_auth_context(
            &Url::parse("https://auth.grokusercontent.com/assets/help")
                .expect("url should parse"),
            true,
        );
        assert_eq!(decision.disposition, NavigationDisposition::ExternalBrowser);
        assert!(!decision.is_auth_flow);
    }

    #[test]
    fn triggers_post_auth_reload_for_cookie_bridge_completion() {
        let url =
            Url::parse("https://auth.grokusercontent.com/set-cookie").expect("url should parse");
        let decision = classify_with_auth_context(&url, true);
        assert!(should_trigger_post_auth_reload(&url, decision));
    }

    #[test]
    fn does_not_trigger_post_auth_reload_for_final_grok_landing() {
        let url = Url::parse("https://grok.com/").expect("url should parse");
        let decision = classify_with_auth_context(&url, true);
        assert!(!should_trigger_post_auth_reload(&url, decision));
    }

    #[test]
    fn pushes_non_grok_xai_pages_external() {
        let decision = classify(&Url::parse("https://x.ai/company").expect("url should parse"));
        assert_eq!(decision.disposition, NavigationDisposition::ExternalBrowser);
        assert!(!decision.is_auth_flow);
    }
}
