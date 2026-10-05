// Applies db/migrations/*.sql in order, each once, recording it in arqade_migrations (ADR-074).
//
// Runs before every build. It migrates only the production database: on Vercel when VERCEL_ENV is
// "production", or anywhere when ARQADE_MIGRATE=1 is set by hand. A preview build of an unmerged branch
// never changes the live database. With no database configured it does nothing.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'db', 'migrations');
const url = process.env.DATABASE_URL || process.env.POSTGRES_URL;
const wanted = process.env.ARQADE_MIGRATE === '1' || process.env.VERCEL_ENV === 'production';

if (!url || !wanted) {
  console.log(`migrate: skipped (${!url ? 'no database configured' : 'not a production build'})`);
  process.exit(0);
}

const pg = (await import('pg')).default;
const client = new pg.Client({ connectionString: url });
await client.connect();
try {
  // One migrator at a time, should two builds overlap.
  await client.query('SELECT pg_advisory_lock(7413)');
  await client.query('CREATE TABLE IF NOT EXISTS arqade_migrations (name text PRIMARY KEY, applied bigint NOT NULL)');
  const done = new Set((await client.query('SELECT name FROM arqade_migrations')).rows.map((r) => r.name));
  for (const name of fs.readdirSync(dir).filter((f) => f.endsWith('.sql')).sort()) {
    if (done.has(name)) continue;
    await client.query('BEGIN');
    try {
      await client.query(fs.readFileSync(path.join(dir, name), 'utf8'));
      await client.query('INSERT INTO arqade_migrations (name, applied) VALUES ($1, $2)', [name, Date.now()]);
      await client.query('COMMIT');
      console.log(`migrate: applied ${name}`);
    } catch (e) {
      await client.query('ROLLBACK');
      throw e;
    }
  }
  console.log('migrate: up to date');
} finally {
  await client.end();
}
