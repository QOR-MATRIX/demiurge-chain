#!/bin/sh
# The Demiurge node's boot script, for the hosted devnet
# (docs/architecture/DEVNET_PLAN.md §1.4 and §2).
#
# One image, two roles, chosen by DEMIURGE_ROLE:
#
#   validator  authors and votes. Nothing public: RPC stays on localhost.
#   rpc        serves public RPC with safe methods only. No session key, never
#              a validator.
#
# ON FIRST BOOT it makes this node's keys on its own disk (/data, a volume):
#
#   - the network key (libp2p), with `key generate-node-key --file`;
#   - for a validator, an Aura key (Sr25519) and a GRANDPA key (Ed25519),
#     inserted into the keystore at /data/keystore.
#
# Every secret goes to a file on /data with mode 600, and nowhere else: not to
# standard output, not to standard error, and not onto a command line. `key
# generate` prints its secret phrase to standard output, so that output is sent
# to a file; the phrase reaches `key insert` as the path of a file holding it,
# never as an argument. The intermediate files are removed once the keystore
# holds the key. The keystore files themselves are written 600 by the node.
#
# On every boot it then prints ONLY public values, each on a line of its own
# beginning `DEMIURGE_PUBLIC`, so they are easy to find in a log:
#
#   DEMIURGE_PUBLIC role=validator
#   DEMIURGE_PUBLIC peer_id=12D3Koo...
#   DEMIURGE_PUBLIC aura_ss58=5...         (validator only)
#   DEMIURGE_PUBLIC aura_hex=0x...         (validator only)
#   DEMIURGE_PUBLIC grandpa_ss58=5...      (validator only)
#   DEMIURGE_PUBLIC grandpa_hex=0x...      (validator only)
#
# A later boot finds the keys and reuses them; the public values are read back
# from the node key file and the keystore, so they are the keys actually in use.
#
# UNTIL A CHAIN SPECIFICATION EXISTS it does not start a chain. It prints the
# public values and waits, checking for the file every 30 seconds. The public
# values go into the specification (chain_spec.rs, `devnet_chain_spec`), so the
# specification cannot exist before this first boot (the plan's step order).
#
# Environment:
#
#   DEMIURGE_ROLE         required: `validator` or `rpc`.
#   DEMIURGE_CHAIN_SPEC   the chain specification file. Default /data/spec.json.
#   DEMIURGE_BOOTNODES    space-separated multiaddresses, passed as --bootnodes.
#                         Validator B and the RPC node name validator A, e.g.
#                         /dns/<service>.railway.internal/tcp/30333/p2p/<peer id>
#   DEMIURGE_NODE_NAME    optional --name.
#   DEMIURGE_RPC_CORS     rpc role only, optional --rpc-cors. Unset keeps the SDK
#                         default, which refuses any Host header but localhost's:
#                         see the note where it is passed, below.
#   RPC limits, rpc role only, each passed only when set. No number is chosen
#   here: the plan sets them from measurement, and unset means the SDK default.
#     DEMIURGE_RPC_RATE_LIMIT                       --rpc-rate-limit
#     DEMIURGE_RPC_MAX_CONNECTIONS                  --rpc-max-connections
#     DEMIURGE_RPC_MAX_SUBSCRIPTIONS_PER_CONNECTION --rpc-max-subscriptions-per-connection
#     DEMIURGE_RPC_MAX_BATCH_REQUEST_LEN            --rpc-max-batch-request-len
#
# Users. The container starts as root only to give /data to the unprivileged
# user `demiurge` (uid 10001), because a Railway volume is mounted owned by root.
# It then drops to that user before any key is made, and the node never runs as
# root. So a Railway service needs no RAILWAY_RUN_UID.

set -eu

NODE=/usr/local/bin/demiurge-node
DATA=/data
NODE_KEY="$DATA/node-key"
KEYSTORE="$DATA/keystore"
SPEC="${DEMIURGE_CHAIN_SPEC:-$DATA/spec.json}"
RUN_USER=demiurge
RUN_UID=10001

# The keystore names each file <key type in hex><public key in hex>.
AURA_TYPE_HEX=61757261    # "aura"
GRANDPA_TYPE_HEX=6772616e # "gran"

say() { echo "demiurge-entrypoint: $*" >&2; }
fail() {
	say "$*"
	exit 1
}

ROLE="${DEMIURGE_ROLE:-}"
case "$ROLE" in
validator | rpc) ;;
*) fail "DEMIURGE_ROLE must be 'validator' or 'rpc' (it is '$ROLE')" ;;
esac

# Every file made from here on is readable by its owner only.
umask 077

# ---- Root: hand /data to the node's user, then drop to it --------------------
if [ "$(id -u)" = 0 ]; then
	mkdir -p "$DATA"
	if [ "$(stat -c %u "$DATA")" != "$RUN_UID" ]; then
		chown -R "$RUN_USER:$RUN_USER" "$DATA"
	fi
	chmod 700 "$DATA"
	export HOME="$DATA"
	exec setpriv --reuid="$RUN_USER" --regid="$RUN_USER" --init-groups -- "$0" "$@"
fi

# ---- Keys: made once, on this disk -------------------------------------------
if [ ! -s "$NODE_KEY" ]; then
	say "first boot: making this node's network key"
	# It writes the secret to the file and the peer id to standard error. The
	# peer id is printed below from the file, so this command prints nothing.
	"$NODE" key generate-node-key --file "$NODE_KEY" >/dev/null 2>&1 ||
		fail "could not make the network key"
	chmod 600 "$NODE_KEY"
fi

# The keystore file holding the one key of a type, or nothing.
key_file() {
	found=""
	for f in "$KEYSTORE/$1"*; do
		[ -e "$f" ] || continue
		[ -z "$found" ] || fail "the keystore holds more than one key of type $1; refusing to guess which is in use"
		found="$f"
	done
	echo "$found"
}

# make_session_key <key type> <key type in hex> <scheme>
make_session_key() {
	existing="$(key_file "$2")" || exit 1
	[ -z "$existing" ] || return 0
	say "first boot: making this validator's $1 key ($3)"
	mkdir -p "$KEYSTORE"
	chmod 700 "$KEYSTORE"
	generated="$DATA/.keygen-$1.json"
	phrase="$DATA/.keygen-$1.phrase"
	# The secret phrase goes from the command to a 600 file, and from that file
	# to another, never through standard output or an argument.
	"$NODE" key generate --scheme "$3" --output-type json >"$generated" 2>/dev/null ||
		fail "could not generate the $1 key"
	jq -er .secretPhrase <"$generated" >"$phrase" 2>/dev/null ||
		fail "could not read the generated $1 key"
	rm -f "$generated"
	"$NODE" key insert --keystore-path "$KEYSTORE" --key-type "$1" --scheme "$3" \
		--suri "$phrase" >/dev/null 2>&1 ||
		fail "could not insert the $1 key into the keystore"
	rm -f "$phrase"
}

if [ "$ROLE" = validator ]; then
	make_session_key aura "$AURA_TYPE_HEX" sr25519
	make_session_key gran "$GRANDPA_TYPE_HEX" ed25519
fi

# ---- Public values, every boot -----------------------------------------------
peer_id="$("$NODE" key inspect-node-key --file "$NODE_KEY" 2>/dev/null)" ||
	fail "could not read the network key"

# public_lines <name> <key type in hex> <scheme>: the key's SS58 form and hex,
# read from the keystore file's NAME (the public key). The file's content, the
# secret, is never read here.
public_lines() {
	file="$(key_file "$2")" || exit 1
	[ -n "$file" ] || fail "the keystore has no $1 key"
	hex="0x${file##*/$2}"
	ss58="$("$NODE" key inspect --public --scheme "$3" --output-type json "$hex" 2>/dev/null |
		jq -er .ss58PublicKey)" || fail "could not read the $1 public key"
	echo "DEMIURGE_PUBLIC ${1}_ss58=$ss58"
	echo "DEMIURGE_PUBLIC ${1}_hex=$hex"
}

echo "DEMIURGE_PUBLIC role=$ROLE"
echo "DEMIURGE_PUBLIC peer_id=$peer_id"
if [ "$ROLE" = validator ]; then
	public_lines aura "$AURA_TYPE_HEX" sr25519
	public_lines grandpa "$GRANDPA_TYPE_HEX" ed25519
fi

# ---- No specification yet: wait, do not start a chain ------------------------
if [ ! -s "$SPEC" ]; then
	say "no chain specification at $SPEC: not starting a chain. Waiting for it (checked every 30 seconds)."
	trap 'exit 0' TERM INT
	while [ ! -s "$SPEC" ]; do
		sleep 30 &
		wait $! || true
	done
	trap - TERM INT
	say "found $SPEC"
fi

# ---- Start the node ----------------------------------------------------------
# --allow-private-ip: at Alpha the nodes peer only over the host's private
# network (the plan's §2). A `Live` chain refuses private addresses unless told,
# and every address on that network is private.
set -- \
	--chain "$SPEC" \
	--base-path "$DATA" \
	--node-key-file "$NODE_KEY" \
	--allow-private-ip \
	--no-telemetry

[ -z "${DEMIURGE_NODE_NAME:-}" ] || set -- "$@" --name "$DEMIURGE_NODE_NAME"

for bootnode in ${DEMIURGE_BOOTNODES:-}; do
	set -- "$@" --bootnodes "$bootnode"
done

case "$ROLE" in
validator)
	# RPC stays on its default, localhost. --force-authoring keeps blocks coming
	# while the other validator restarts (the plan's §2: finality still waits).
	set -- "$@" --validator --force-authoring --keystore-path "$KEYSTORE"
	;;
rpc)
	# Safe methods only: author_insertKey, author_rotateKeys and the other
	# methods that change a node are refused.
	set -- "$@" --rpc-external --rpc-methods safe --rpc-port 9944
	# --rpc-cors stays at the SDK default unless DEMIURGE_RPC_CORS is set. Note
	# what that default does in sc-rpc-server 31.0.0 (utils.rs, host_filtering):
	# any CORS list, the default included, also turns on a Host-header filter that
	# admits only localhost, 127.0.0.1 and [::1] on the RPC port. A request
	# arriving under a public name (Host: rpc.qorsync.dev) is refused with 403,
	# /health/readiness included. Only `all` turns that filter off.
	[ -z "${DEMIURGE_RPC_CORS:-}" ] ||
		set -- "$@" --rpc-cors "$DEMIURGE_RPC_CORS"
	[ -z "${DEMIURGE_RPC_RATE_LIMIT:-}" ] ||
		set -- "$@" --rpc-rate-limit "$DEMIURGE_RPC_RATE_LIMIT"
	[ -z "${DEMIURGE_RPC_MAX_CONNECTIONS:-}" ] ||
		set -- "$@" --rpc-max-connections "$DEMIURGE_RPC_MAX_CONNECTIONS"
	[ -z "${DEMIURGE_RPC_MAX_SUBSCRIPTIONS_PER_CONNECTION:-}" ] ||
		set -- "$@" --rpc-max-subscriptions-per-connection "$DEMIURGE_RPC_MAX_SUBSCRIPTIONS_PER_CONNECTION"
	[ -z "${DEMIURGE_RPC_MAX_BATCH_REQUEST_LEN:-}" ] ||
		set -- "$@" --rpc-max-batch-request-len "$DEMIURGE_RPC_MAX_BATCH_REQUEST_LEN"
	;;
esac

say "starting the $ROLE node on $SPEC"
exec "$NODE" "$@"
