// Shared by the tests: the site's TypeScript compiled as it is, and a Postgres (PGlite, Postgres compiled
// to WASM) with the site's own migrations applied, behind the same statement interface production uses.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';
import {PGlite} from '@electric-sql/pglite';

// One folder per test process: the test files run at once, and writing the same compiled file from two of them let
// one import the other's half-written copy ("dbModule.statements is not a function", CI, 6 October 2026).
const dir=path.resolve('work/tests',String(process.pid));fs.mkdirSync(dir,{recursive:true});

/** Compile `source` to work/tests/<name>.mjs, applying exact text replacements first, each of which must match. */
export function compile(source,name,replacements=[]){
  let s=fs.readFileSync(source,'utf8');
  for(const [a,b]of replacements){if(!s.includes(a))throw new Error(`${source}: replacement not found: ${a}`);s=s.replace(a,b)}
  const out=path.join(dir,name+'.mjs');
  fs.writeFileSync(out,ts.transpileModule(s,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
  return pathToFileURL(out).href;
}

const dbModule=await import(compile('lib/db.ts','db'));

/** A fresh database with every migration applied, and the site's statement interface over it. */
export async function postgres(){
  const pg=new PGlite({parsers:{20:Number,1700:Number}});
  for(const f of fs.readdirSync('db/migrations').filter(f=>f.endsWith('.sql')).sort())await pg.exec(fs.readFileSync(path.join('db/migrations',f),'utf8'));
  const on=q=>({query:async(sql,params)=>{const r=await q.query(sql,params);return{rows:r.rows,affectedRows:r.affectedRows??0}}});
  const runner={...on(pg),transaction:fn=>pg.transaction(tx=>fn(on(tx)))};
  const one=async(sql,params=[])=>(await pg.query(sql,params)).rows[0]??null;
  return {pg,one,db:dbModule.statements(runner)};
}
