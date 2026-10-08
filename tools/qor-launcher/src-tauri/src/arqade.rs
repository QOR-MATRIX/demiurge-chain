//! ARQADE in a window of its own (ADR-078 decision 5, DIRECTION P7.18).
//!
//! The launcher opens ARQADE's website in a second window, not inside its own: the main window's content security
//! policy refuses frames (`frame-src 'none'`), and a page from the network must never share a webview with the
//! interface that draws the host's approvals. The window is given no capability (`capabilities/default.json` names
//! only `main`), so ARQADE's page cannot call the host at all.
//!
//! What the window may load is decided here, by [`route`], on every navigation and every request for a new window:
//! ARQADE itself and QOR ID (where its sign-in happens) stay in the window; a `qor://pay` link is handed to the same
//! path a link from the browser takes, so it is checked, asked in the host dialog and paid there, with no round trip
//! through the operating system; any other web address opens in the person's own browser; anything else is refused.
//!
//! **Signed in with the launcher's session.** When the launcher is signed in to QOR ID, the window opens at ARQADE's
//! own sign-in, and the launcher stops the window on its way to QOR ID's sign-in page (`/oauth/authorize`). It sends
//! that address to QOR ID with its own token (`/api/v1/oauth/handoff`), and loads in the window the redirect QOR ID
//! answers with, which must land on ARQADE. The code in it is ARQADE's ordinary single-use code, bound to the PKCE
//! challenge whose verifier only ARQADE holds, so nothing new travels in an address. Without a session, or if QOR ID
//! refuses, the window loads QOR ID's page as it would have, and the person signs in there.

use std::collections::HashSet;
use std::sync::Arc;

use tauri::Manager;
use tauri::Url;

/// ARQADE's address (ADR-074).
pub const ARQADE_URL: &str = "https://qor-arqade-tau.vercel.app/";

/// The window's label. Not in any capability, so the page in it reaches no host command.
pub const WINDOW: &str = "arqade";

/// What happens to an address the ARQADE window is asked to load.
#[derive(Debug, PartialEq, Eq)]
pub enum Route {
    /// Load it in the ARQADE window.
    Stay,
    /// A `qor://` link: hand it to the pay path, and load nothing.
    Pay,
    /// Another web address: open it in the person's browser, and load nothing here.
    Browser,
    /// Load nothing.
    Refuse,
}

/// Where `url` goes. `allowed` is the hosts that may load in the window: ARQADE's and QOR ID's.
pub fn route(url: &Url, allowed: &[&str]) -> Route {
    match url.scheme() {
        "qor" => Route::Pay,
        "https" => match url.host_str() {
            Some(host) if allowed.iter().any(|a| a.eq_ignore_ascii_case(host)) => Route::Stay,
            Some(_) => Route::Browser,
            None => Route::Refuse,
        },
        // Plain http only ever leaves for the browser: nothing in this window is fetched unencrypted.
        "http" if url.host_str().is_some() => Route::Browser,
        _ => Route::Refuse,
    }
}

/// The hosts that may load in the window: ARQADE's, and the QOR ID the launcher is set to (its sign-in page).
fn allowed_hosts(auth_endpoint: &str) -> Vec<String> {
    let mut hosts = vec![host_of(ARQADE_URL).unwrap_or_default()];
    if let Some(h) = host_of(auth_endpoint) {
        hosts.push(h);
    }
    hosts.retain(|h| !h.is_empty());
    hosts
}

fn host_of(address: &str) -> Option<String> {
    Url::parse(address).ok()?.host_str().map(str::to_owned)
}

/// Whether `url` is QOR ID's sign-in page for an app, on the QOR ID the launcher uses: where the launcher can sign the
/// person in with its own session instead.
pub fn is_sign_in(url: &Url, qor_id_host: &str) -> bool {
    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|h| h.eq_ignore_ascii_case(qor_id_host))
        && url.path() == "/oauth/authorize"
}

/// Whether an address QOR ID answered a handoff with lands on ARQADE, over https. Anything else is not loaded.
pub fn lands_on_arqade(redirect: &Url) -> bool {
    redirect.scheme() == "https" && redirect.host_str() == host_of(ARQADE_URL).as_deref()
}

/// Where the window opens: at ARQADE's sign-in when the launcher is signed in, so the person arrives signed in.
pub fn start_address(signed_in: bool) -> String {
    if signed_in {
        format!("{ARQADE_URL}api/auth/login")
    } else {
        ARQADE_URL.to_string()
    }
}

/// What the window's guard knows: the hosts it may load, QOR ID's host, and the sign-in pages it should let load
/// because the launcher could not sign the person in itself.
#[derive(Clone)]
struct Guard {
    hosts: Vec<String>,
    qor_id_host: Option<String>,
    passed: Arc<parking_lot::Mutex<HashSet<String>>>,
}

fn signed_in(app: &tauri::AppHandle) -> bool {
    app.try_state::<crate::AppState>()
        .is_some_and(|state| state.identity.session().is_some())
}

/// Ask QOR ID to sign the person in for this sign-in page, then load where it says, if that is ARQADE. Otherwise the
/// page itself loads, once.
fn sign_in_with_session(
    app: tauri::AppHandle,
    passed: Arc<parking_lot::Mutex<HashSet<String>>>,
    page: Url,
) {
    tauri::async_runtime::spawn(async move {
        let answer = match app.try_state::<crate::AppState>() {
            Some(state) => state.identity.handoff(page.as_str()).await,
            None => Err(crate::error::QorError::NotAuthenticated),
        };
        let next = match answer.map(|r| Url::parse(&r)) {
            Ok(Ok(redirect)) if lands_on_arqade(&redirect) => redirect,
            other => {
                if let Err(e) = other {
                    tracing::info!("ARQADE's sign-in falls back to QOR ID's page: {e}");
                }
                passed.lock().insert(page.to_string());
                page
            }
        };
        if let Some(window) = app.get_webview_window(WINDOW) {
            if let Err(e) = window.navigate(next) {
                tracing::warn!("ARQADE's window could not be moved on from its sign-in: {e}");
            }
        }
    });
}

/// Act on `url` for the ARQADE window: `true` when it should load there.
fn follow(app: &tauri::AppHandle, guard: &Guard, url: &Url) -> bool {
    if guard
        .qor_id_host
        .as_deref()
        .is_some_and(|qor| is_sign_in(url, qor))
    {
        // Let through once after the launcher could not sign the person in itself.
        if guard.passed.lock().remove(url.as_str()) {
            return true;
        }
        if signed_in(app) {
            sign_in_with_session(app.clone(), guard.passed.clone(), url.clone());
            return false;
        }
    }
    let allowed: Vec<&str> = guard.hosts.iter().map(String::as_str).collect();
    match route(url, &allowed) {
        Route::Stay => true,
        Route::Pay => {
            crate::handle_pay_link(app.clone(), url.to_string());
            false
        }
        Route::Browser => {
            use tauri_plugin_opener::OpenerExt;
            if let Err(e) = app.opener().open_url(url.as_str(), None::<&str>) {
                tracing::warn!("ARQADE asked for a page the browser could not open: {e}");
            }
            false
        }
        Route::Refuse => false,
    }
}

/// Open ARQADE in its own window, or bring that window forward if it is open.
pub fn open(app: &tauri::AppHandle, auth_endpoint: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let start = Url::parse(&start_address(signed_in(app))).map_err(|e| e.to_string())?;
    let guard = Guard {
        hosts: allowed_hosts(auth_endpoint),
        qor_id_host: host_of(auth_endpoint),
        passed: Arc::default(),
    };
    let (nav_app, nav_guard) = (app.clone(), guard.clone());
    let (pop_app, pop_guard) = (app.clone(), guard);

    tauri::WebviewWindowBuilder::new(app, WINDOW, tauri::WebviewUrl::External(start))
        .title("ARQADE")
        .inner_size(1280.0, 820.0)
        .min_inner_size(880.0, 600.0)
        .on_navigation(move |url| follow(&nav_app, &nav_guard, url))
        // A page asking for a new window gets no window of the host's making: an address that may stay is still
        // opened in the browser, because a second ARQADE window would be a second, unguarded place to approve from.
        .on_new_window(move |url, _features| {
            let allowed: Vec<&str> = pop_guard.hosts.iter().map(String::as_str).collect();
            if route(&url, &allowed) == Route::Stay {
                use tauri_plugin_opener::OpenerExt;
                let _ = pop_app.opener().open_url(url.as_str(), None::<&str>);
            } else {
                follow(&pop_app, &pop_guard, &url);
            }
            tauri::webview::NewWindowResponse::Deny
        })
        .build()
        .map(|_| ())
        .map_err(|e| format!("ARQADE's window could not be opened: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOSTS: &[&str] = &["qor-arqade-tau.vercel.app", "id.qorsync.dev"];

    fn at(address: &str) -> Route {
        route(&Url::parse(address).expect("a URL"), HOSTS)
    }

    #[test]
    fn arqade_and_qor_id_load_in_the_window() {
        assert_eq!(
            at("https://qor-arqade-tau.vercel.app/games/flux"),
            Route::Stay
        );
        assert_eq!(
            at("https://id.qorsync.dev/oauth/authorize?client_id=arqade"),
            Route::Stay
        );
        assert_eq!(at("https://ID.QORSYNC.DEV/"), Route::Stay);
    }

    #[test]
    fn a_pay_link_goes_to_the_pay_path() {
        assert_eq!(at("qor://pay?r=00&s=00"), Route::Pay);
    }

    #[test]
    fn any_other_web_address_leaves_for_the_browser() {
        assert_eq!(at("https://example.com/"), Route::Browser);
        // A look-alike host is another host.
        assert_eq!(
            at("https://qor-arqade-tau.vercel.app.example.com/"),
            Route::Browser
        );
        assert_eq!(
            at("https://evil.example/qor-arqade-tau.vercel.app"),
            Route::Browser
        );
        // Never unencrypted in the window, even for an allowed host.
        assert_eq!(at("http://qor-arqade-tau.vercel.app/"), Route::Browser);
    }

    #[test]
    fn anything_else_is_refused() {
        for address in [
            "file:///C:/Windows/win.ini",
            "javascript:alert(1)",
            "data:text/html,hi",
            "ftp://example.com/",
        ] {
            assert_eq!(at(address), Route::Refuse, "{address}");
        }
    }

    #[test]
    fn qor_id_s_sign_in_page_is_recognised_only_on_the_qor_id_in_use() {
        let at = |a: &str| Url::parse(a).expect("a URL");
        let host = "id.qorsync.dev";
        assert!(is_sign_in(
            &at("https://id.qorsync.dev/oauth/authorize?client_id=arqade"),
            host
        ));
        assert!(is_sign_in(
            &at("https://ID.qorsync.dev/oauth/authorize"),
            host
        ));
        assert!(!is_sign_in(&at("https://id.qorsync.dev/account"), host));
        assert!(!is_sign_in(
            &at("https://id.qorsync.dev/oauth/authorize/x"),
            host
        ));
        assert!(!is_sign_in(
            &at("https://evil.example/oauth/authorize"),
            host
        ));
        assert!(!is_sign_in(
            &at("http://id.qorsync.dev/oauth/authorize"),
            host
        ));
    }

    #[test]
    fn only_a_redirect_to_arqade_over_https_is_loaded_after_a_handoff() {
        let at = |a: &str| Url::parse(a).expect("a URL");
        assert!(lands_on_arqade(&at(
            "https://qor-arqade-tau.vercel.app/api/auth/callback?code=c&state=s"
        )));
        assert!(!lands_on_arqade(&at(
            "http://qor-arqade-tau.vercel.app/api/auth/callback"
        )));
        assert!(!lands_on_arqade(&at(
            "https://qor-arqade-tau.vercel.app.example.com/api/auth/callback"
        )));
        assert!(!lands_on_arqade(&at("https://id.qorsync.dev/account")));
        assert!(!lands_on_arqade(&at("qor://pay?r=00&s=00")));
    }

    #[test]
    fn a_signed_in_launcher_opens_arqade_at_its_sign_in() {
        assert_eq!(
            start_address(true),
            "https://qor-arqade-tau.vercel.app/api/auth/login"
        );
        assert_eq!(start_address(false), "https://qor-arqade-tau.vercel.app/");
    }

    #[test]
    fn the_allowed_hosts_are_arqade_and_the_configured_qor_id() {
        assert_eq!(
            allowed_hosts("https://id.qorsync.dev"),
            vec![
                "qor-arqade-tau.vercel.app".to_string(),
                "id.qorsync.dev".to_string()
            ]
        );
        assert_eq!(
            allowed_hosts("http://127.0.0.1:8080"),
            vec![
                "qor-arqade-tau.vercel.app".to_string(),
                "127.0.0.1".to_string()
            ]
        );
        // A local QOR ID over plain http is allowed by host, but `route` still sends http to the browser.
        assert_eq!(
            allowed_hosts("not a url"),
            vec!["qor-arqade-tau.vercel.app".to_string()]
        );
    }
}
