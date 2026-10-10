// The provider's API key, kept in the operating system's keychain (DIRECTION P3.3, ADR-083 decision 4).
//
// On Windows that is the Credential Manager, a generic credential for this user that only this user can read. The key
// is never written to a file, a setting or a log, and is held in memory only while a request is being made. Elsewhere
// there is no keychain yet, and every call says so.
//
// QQ_KEYCHAIN_TARGET names another credential: how a test keeps clear of the creator's own key.

#pragma once

#include <QString>

namespace qq::keychain {

/// The credential's name in the keychain.
QString target();

/// Keep `secret` (replacing any kept before). "" or why it could not be kept.
QString store(const QString &secret);

/// The secret kept, or an empty string if there is none (or no keychain).
QString load();

/// Whether a secret is kept, without reading it.
bool has();

/// Forget the secret. True if one was there.
bool remove();

}  // namespace qq::keychain
