import {api,body,identity,response,getMatch,expire,snapshot,actionMatch} from '@/lib/arena-store';
export const dynamic='force-dynamic';
type Context={params:Promise<{id:string}>};
export async function GET(req:Request,ctx:Context){return api(async()=>{const me=await identity(),{id}=await ctx.params;return response(await snapshot(await expire(await getMatch(id)),me.id))})}
export async function POST(req:Request,ctx:Context){return api(async()=>{const data=await body(req),me=await identity(),{id}=await ctx.params,m=await getMatch(id);return response(await snapshot(await actionMatch(m,me.id,data.action,data.cell,data.revision),me.id))})}
