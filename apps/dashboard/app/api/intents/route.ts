import { NextResponse } from 'next/server'
import { getPaymentsActor } from '../../../src/lib/ic/agent'

export async function GET() {
  // 簡易: イベントから Intent 作成イベントを抽出
  const actor = await getPaymentsActor()
  const data = await (actor as any).list_events(BigInt(0), 200)
  const ids = Array.from(
    new Set(
      data
        .map((e: any) => e.kind && 'IntentCreated' in e.kind ? e.kind.IntentCreated.id : null)
        .filter(Boolean)
    )
  )
  const intents = [] as any[]
  for (const id of ids) {
    const res = await (actor as any).get_intent(id)
    if ('Some' in res) intents.push(res.Some)
  }
  return NextResponse.json({ intents })
}

export async function POST(req: Request) {
  const body = await req.json()
  const { asset, amount, expires_at, metadata } = body
  const actor = await getPaymentsActor()
  const res = await (actor as any).create_intent({ asset, amount, expires_at, metadata })
  return NextResponse.json(res)
}

