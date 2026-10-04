import {api,body,identity,response,gameType,createMatch,snapshot} from '@/lib/arena-store';
export const dynamic='force-dynamic';
export async function POST(req:Request){return api(async()=>{const data=await body(req),me=await identity(),m=await createMatch(me.id,gameType(data.game));return response(await snapshot(m,me.id),201)})}
