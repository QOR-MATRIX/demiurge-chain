// The DevTools port of the Chrome ARQADE's phone check started, as Chrome itself names it.
//
// The check starts Chrome with `--remote-debugging-port=0`, so Chrome takes a port no other process holds and names it
// on stderr ("DevTools listening on ws://127.0.0.1:<port>/…"). The check used to draw a port at random and throw Chrome's
// output away, so when Chrome opened no page (22 October 2026, in the same minute as the launcher's checks on another
// runner) nothing said why. A drawn port could be taken; a chosen one cannot. If Chrome names no port, the error now
// says whether it exited and what it last said.
//
// The check spawns Chrome with stderr piped (`stdio: ['ignore', 'ignore', 'pipe']`) and passes it here. Its stderr is
// read for as long as it runs, so a full pipe can never stall it. tools/qor-launcher/scripts/chrome.mjs is the same
// helper; this package does not reach into that one.

const LISTENING = /DevTools listening on ws:\/\/[^:/]+:(\d+)\//;

export function devtoolsPort(browser, wait = 30000) {
  let said = '';
  browser.stderr.setEncoding('utf8');
  browser.stderr.on('data', (chunk) => {
    said = (said + chunk).slice(-4000);
  });
  const tail = () => said.trim().split('\n').slice(-8).join('\n') || '(nothing)';
  return new Promise((resolve, reject) => {
    const exited = (how) => {
      clearTimeout(timer);
      reject(new Error(`Chrome exited (${how}) before naming a DevTools port; it last said:\n${tail()}`));
    };
    const timer = setTimeout(() => {
      reject(new Error(`Chrome named no DevTools port within ${wait / 1000} s; it last said:\n${tail()}`));
    }, wait);
    const look = () => {
      const found = LISTENING.exec(said);
      if (!found) return;
      clearTimeout(timer);
      browser.stderr.off('data', look);
      resolve(Number(found[1]));
    };
    browser.stderr.on('data', look);
    browser.once('error', (e) => {
      clearTimeout(timer);
      reject(new Error(`Chrome could not be started: ${e.message}`));
    });
    // 'close', not 'exit': it comes after Chrome's last words have been read.
    browser.once('close', (code, signal) => {
      if (!LISTENING.test(said)) exited(signal ?? code);
    });
  });
}
