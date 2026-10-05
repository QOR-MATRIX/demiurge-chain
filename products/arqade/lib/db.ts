// ARQADE's database (ADR-074): Postgres, through a small statement interface the store and the
// sign-in module share. Statements are written with `?` placeholders, numbered here as `$1, $2…`;
// values are always bound, never written into the SQL.
//
// The interface is the one the arcade was written against (prepare, bind, run, all, first, batch), so
// the rules in arena-store.ts did not change when the host did. Tests pass PGlite, a Postgres in WASM.

export type Row = Record<string, unknown>;
export type Executor = { query(sql: string, params: unknown[]): Promise<{ rows: Row[]; affectedRows: number }> };
export type Runner = Executor & { transaction<T>(fn: (tx: Executor) => Promise<T>): Promise<T> };

export type Result<T = Row> = { results: T[]; meta: { changes: number } };
export type Statement = {
  bind(...values: unknown[]): Statement;
  run(): Promise<Result>;
  all<T = Row>(): Promise<Result<T>>;
  first<T = Row>(): Promise<T | null>;
  exec(on: Executor): Promise<Result>;
};
export type Db = { prepare(sql: string): Statement; batch(statements: Statement[]): Promise<Result[]> };

/** `?` placeholders as Postgres's `$1, $2…`. No statement here has a `?` inside a string literal. */
export function numbered(sql: string): string {
  let n = 0;
  return sql.replace(/\?/g, () => `$${++n}`);
}

export function statements(runner: Runner): Db {
  const statement = (sql: string, values: unknown[]): Statement => {
    const text = numbered(sql);
    const exec = async (on: Executor): Promise<Result> => {
      const r = await on.query(text, values);
      return { results: r.rows, meta: { changes: r.affectedRows } };
    };
    return {
      bind: (...v) => statement(sql, v),
      exec,
      run: () => exec(runner),
      all: async <T,>() => (await exec(runner)) as unknown as Result<T>,
      first: async <T,>() => ((await exec(runner)).results[0] as T | undefined) ?? null,
    };
  };
  return {
    prepare: (sql) => statement(sql, []),
    // All or nothing, in order, on one connection: what the arcade's batches rely on.
    batch: (list) => runner.transaction(async (tx) => {
      const out: Result[] = [];
      for (const s of list) out.push(await s.exec(tx));
      return out;
    }),
  };
}

export class Unavailable extends Error {}

/** What may be logged about a database or sign-in failure: its kind and Postgres's code, never its text,
 * which can quote the values a statement carried. */
export function failure(e: unknown): string {
  if (!(e instanceof Error)) return 'unknown';
  const code = (e as { code?: unknown }).code;
  return typeof code === 'string' && /^[0-9A-Z]{5}$/.test(code) ? `${e.name} ${code}` : e.name;
}

type Pool = import('pg').Pool;
const shared = globalThis as unknown as { __arqadePool?: Pool; __arqadeDb?: Db };

/** The connection string Vercel's Postgres integration sets, or null. */
export function databaseUrl(env: Record<string, string | undefined> = process.env): string | null {
  return env.DATABASE_URL || env.POSTGRES_URL || null;
}

async function pool(): Promise<Pool> {
  if (shared.__arqadePool) return shared.__arqadePool;
  const url = databaseUrl();
  if (!url) throw new Unavailable('DATABASE_URL is not set.');
  const pg = (await import('pg')).default;
  // bigint (20) and numeric (1700) as numbers: every one here is a count or a millisecond time,
  // far below 2^53. Not money: CGT never passes through this database.
  const types = { getTypeParser: (oid: number, format?: 'text' | 'binary') => (oid === 20 || oid === 1700 ? Number : pg.types.getTypeParser(oid, format)) };
  shared.__arqadePool = new pg.Pool({ connectionString: url, max: 5, idleTimeoutMillis: 10_000, types });
  return shared.__arqadePool;
}

function pgRunner(): Runner {
  const query = async (on: { query: Pool['query'] }, sql: string, params: unknown[]) => {
    const r = await on.query(sql, params as unknown[]);
    return { rows: r.rows as Row[], affectedRows: r.rowCount ?? 0 };
  };
  return {
    query: async (sql, params) => query(await pool(), sql, params),
    transaction: async (fn) => {
      const client = await (await pool()).connect();
      try {
        await client.query('BEGIN');
        const out = await fn({ query: (sql, params) => query(client, sql, params) });
        await client.query('COMMIT');
        return out;
      } catch (e) {
        await client.query('ROLLBACK').catch(() => undefined);
        throw e;
      } finally {
        client.release();
      }
    },
  };
}

/** The site's database. Throws `Unavailable` when no database is connected. */
export function database(): Db {
  if (!databaseUrl()) throw new Unavailable('DATABASE_URL is not set.');
  shared.__arqadeDb ??= statements(pgRunner());
  return shared.__arqadeDb;
}
