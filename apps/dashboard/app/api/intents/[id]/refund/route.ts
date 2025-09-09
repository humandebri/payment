import { NextResponse } from 'next/server'
import { getPaymentsActor } from '../../../../../src/lib/ic/agent'

export async function POST(req: Request, { params }: { params: { id: string } }) {
  const body = await req.json().catch(() => ({}))
  const amount = body?.amount ? BigInt(body.amount) : undefined
  const actor = await getPaymentsActor()
  const res = await (actor as any).refund({ intent_id: params.id, amount })
  return NextResponse.json(res)
}

