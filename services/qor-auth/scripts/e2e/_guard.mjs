// Every end-to-end script here calls this before it registers anything.
//
// The scripts register addresses at example.invalid, which nobody holds. Against a service that sends
// through the real Resend, each one is sent as real mail, bounces, and counts against the sending
// domain. That happened once, from a machine with a real key in its environment.
//
// The service says whether its mail leaves the machine it runs on, as `email_leaves_this_machine` on
// GET /health. A script runs only when that is exactly `false`: email is off, or goes to a stand-in for
// Resend on that machine. A service that does not say (an older build, or something that is not
// QOR ID) is refused too, because not knowing is not the same as safe. There is no switch to run
// anyway: these scripts have no business sending real mail.
//
// Start a service these scripts can run against with:
//   powershell -File tools/qor-launcher/scripts/start-local.ps1 -NoChain -EmailStandIn
// or by hand, with RESEND_API_URL=http://127.0.0.1:59925, or with RESEND_API_KEY and EMAIL_FROM unset.
export async function refuseRealEmail(base) {
  const origin = new URL(base).origin;
  let health = null;
  let status = 'no answer';
  try {
    const res = await fetch(`${origin}/health`);
    status = res.status;
    health = await res.json();
  } catch {}
  const leaves = health?.email_leaves_this_machine;
  if (leaves === false) return;
  const why =
    leaves === true
      ? 'it sends email through the real Resend, and this script registers addresses nobody holds'
      : `GET /health (${status}) does not say whether its email leaves the machine, so it may`;
  console.error(`REFUSED  not running against ${origin}: ${why}.`);
  console.error('         Nothing was registered and nothing was sent. See scripts/e2e/_guard.mjs.');
  process.exit(2);
}
