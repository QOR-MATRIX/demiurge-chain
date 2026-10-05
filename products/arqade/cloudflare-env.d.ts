declare namespace Cloudflare {
  interface Env {
    DB?: D1Database;
    BUCKET?: R2Bucket;
    /** QOR ID sign-in (P7.3). The secret is set in the host's environment, never in the repository. */
    QOR_ID_URL?: string;
    QOR_CLIENT_ID?: string;
    QOR_CLIENT_SECRET?: string;
    QOR_REDIRECT_URI?: string;
  }
}
