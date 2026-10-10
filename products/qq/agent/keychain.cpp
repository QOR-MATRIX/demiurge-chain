#include "keychain.h"

#ifdef Q_OS_WIN
#define NOMINMAX
#include <windows.h>
#include <wincred.h>
#endif

namespace qq::keychain {

QString target()
{
    const QString set = qEnvironmentVariable("QQ_KEYCHAIN_TARGET");
    return set.isEmpty() ? QStringLiteral("QQ Studio/Anthropic API key") : set;
}

#ifdef Q_OS_WIN

QString store(const QString &secret)
{
    const std::wstring name = target().toStdWString();
    const QByteArray blob = secret.toUtf8();
    CREDENTIALW credential{};
    credential.Type = CRED_TYPE_GENERIC;
    credential.TargetName = const_cast<LPWSTR>(name.c_str());
    credential.CredentialBlobSize = DWORD(blob.size());
    credential.CredentialBlob = reinterpret_cast<LPBYTE>(const_cast<char *>(blob.constData()));
    credential.Persist = CRED_PERSIST_LOCAL_MACHINE;  // this user, on this computer; not carried to others
    if (!CredWriteW(&credential, 0))
        return QStringLiteral("The key could not be kept in Windows Credential Manager (error %1).").arg(GetLastError());
    return {};
}

QString load()
{
    const std::wstring name = target().toStdWString();
    PCREDENTIALW credential = nullptr;
    if (!CredReadW(name.c_str(), CRED_TYPE_GENERIC, 0, &credential))
        return {};
    const QString secret = QString::fromUtf8(reinterpret_cast<const char *>(credential->CredentialBlob),
                                             int(credential->CredentialBlobSize));
    SecureZeroMemory(credential->CredentialBlob, credential->CredentialBlobSize);
    CredFree(credential);
    return secret;
}

bool has()
{
    const std::wstring name = target().toStdWString();
    PCREDENTIALW credential = nullptr;
    if (!CredReadW(name.c_str(), CRED_TYPE_GENERIC, 0, &credential))
        return false;
    SecureZeroMemory(credential->CredentialBlob, credential->CredentialBlobSize);
    CredFree(credential);
    return true;
}

bool remove()
{
    const std::wstring name = target().toStdWString();
    return CredDeleteW(name.c_str(), CRED_TYPE_GENERIC, 0);
}

#else

QString store(const QString &)
{
    return QStringLiteral("There is no keychain for QQ Studio on this system yet.");
}
QString load() { return {}; }
bool has() { return false; }
bool remove() { return false; }

#endif

}  // namespace qq::keychain
