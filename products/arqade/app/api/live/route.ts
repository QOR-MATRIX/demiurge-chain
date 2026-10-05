import {api,identity,database,response} from '@/lib/arena-store';
export const dynamic='force-dynamic';
export async function GET(){return api(async()=>{
  const me=await identity(),db=database(),now=Date.now();
  const ranking=(game:string)=>db.prepare(`SELECT s.player,?::text AS game,SUM(s.wins) AS wins,SUM(s.losses) AS losses,SUM(s.draws) AS draws,SUM(s.points) AS points,p.alias FROM standings s JOIN players p ON p.id=s.player WHERE (?::text='all' OR s.game=?) AND s.wins+s.losses+s.draws>0 GROUP BY s.player,p.alias ORDER BY points DESC,wins DESC,s.player ASC LIMIT 100`).bind(game,game,game);
  const results=await db.batch([
    db.prepare('SELECT COUNT(*) AS count FROM players WHERE seen>?').bind(now-45000),
    db.prepare(`SELECT m.id,m.game,m.status,m.moves,m.updated,p.alias,CASE WHEN m.host=? THEN 1 WHEN m.guest=? THEN 2 ELSE 0 END AS seat FROM matches m JOIN players p ON p.id=m.host WHERE (m.status='waiting' AND m.deadline>?) OR (m.status='active' AND (m.updated>? OR m.host=? OR m.guest=?)) OR ((m.host=? OR m.guest=?) AND m.status='done' AND m.updated>?) ORDER BY seat DESC,m.created DESC LIMIT 30`).bind(me.id,me.id,now,now-180000,me.id,me.id,me.id,me.id,now-3600000),
    ranking('all'),ranking('flux'),ranking('reversi'),
    db.prepare('SELECT m.id,m.player,m.body,m.created,p.alias FROM messages m JOIN players p ON p.id=m.player WHERE m.deleted=0 AND m.created>? ORDER BY m.created DESC LIMIT 60').bind(now-604800000)
  ]);
  return response({me,online:(results[0].results[0] as {count:number}).count,matches:results[1].results,standings:[...results[2].results,...results[3].results,...results[4].results],messages:results[5].results.reverse(),serverNow:now,refreshMs:2000});
})}
