import {api,body,identity,response,getMatch,expire,snapshot,actionMatch,database} from '@/lib/arena-store';
import {reportTaskForRequest} from '@/lib/qor-deps';
export const dynamic='force-dynamic';
type Context={params:Promise<{id:string}>};
// A finished match is one of ARQADE's tasks (ADR-078). Each player is reported once, from their own request.
async function firstMatch(view:{status:string;seat:number},player:string){if(view.status!=='done'||view.seat===0)return;const r=await database().prepare('UPDATE players SET first_match_reported=1 WHERE id=? AND first_match_reported=0').bind(player).run();if(r.meta.changes)await reportTaskForRequest('first-match')}
export async function GET(req:Request,ctx:Context){return api(async()=>{const me=await identity(),{id}=await ctx.params;const view=await snapshot(await expire(await getMatch(id)),me.id);await firstMatch(view,me.id);return response(view)})}
export async function POST(req:Request,ctx:Context){return api(async()=>{const data=await body(req),me=await identity(),{id}=await ctx.params,m=await getMatch(id);const view=await snapshot(await actionMatch(m,me.id,data.action,data.cell,data.revision),me.id);await firstMatch(view,me.id);return response(view)})}
