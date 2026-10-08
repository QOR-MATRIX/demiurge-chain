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

/// Act on `url` for the ARQADE window: `true` when it should load there.
fn follow(app: &tauri::AppHandle, hosts: &[String], url: &Url) -> bool {
    let allowed: Vec<&str> = hosts.iter().map(String::as_str).collect();
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

    let start = Url::parse(ARQADE_URL).map_err(|e| e.to_string())?;
    let hosts = allowed_hosts(auth_endpoint);
    let (nav_app, nav_hosts) = (app.clone(), hosts.clone());
    let (pop_app, pop_hosts) = (app.clone(), hosts);

    tauri::WebviewWindowBuilder::new(app, WINDOW, tauri::WebviewUrl::External(start))
        .title("ARQADE")
        .inner_size(1280.0, 820.0)
        .min_inner_size(880.0, 600.0)
        .on_navigation(move |url| follow(&nav_app, &nav_hosts, url))
        // A page asking for a new window gets no window of the host's making: an address that may stay is still
        // opened in the browser, because a second ARQADE window would be a second, unguarded place to approve from.
        .on_new_window(move |url, _features| {
            let allowed: Vec<&str> = pop_hosts.iter().map(String::as_str).collect();
            if route(&url, &allowed) == Route::Stay {
                use tauri_plugin_opener::OpenerExt;
                let _ = pop_app.opener().open_url(url.as_str(), None::<&str>);
            } else {
                follow(&pop_app, &pop_hosts, &url);
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
