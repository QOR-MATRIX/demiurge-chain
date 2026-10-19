/* The QOR Launcher core's bridge (ADR-104 decision 3), as the Qt shell calls it. The functions are
 * tools/qor-launcher/core/src/bridge.rs.
 *
 * A request is one zero-terminated UTF-8 JSON object, {"command": "...", "args": {...}}. The answer is one too:
 * {"ok": <answer>} or {"error": {"kind": "...", "message": "..."}}. Every answer qor_core_call returns is given back
 * to qor_core_free, once. Amounts cross as decimal strings of embers: a u128 does not fit in a double. */

#ifndef QOR_CORE_H
#define QOR_CORE_H

#ifdef __cplusplus
extern "C" {
#endif

/* The core's version. Static: never freed. */
const char *qor_core_version(void);

/* Answer one request. Never returns null. */
char *qor_core_call(const char *request);

/* Give back an answer qor_core_call made. Null is ignored. */
void qor_core_free(char *answer);

#ifdef __cplusplus
}
#endif

#endif /* QOR_CORE_H */
