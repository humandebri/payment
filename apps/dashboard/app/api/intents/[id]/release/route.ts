import { NextResponse } from 'next/server'
import { getPaymentsActor } from '../../../../../src/lib/ic/agent'
import { Principal } from '@dfinity/principal'

export async function POST(req: Request, { params }: { params: { id: string } }) {
  const body = await req.json().catch(() => ({}))
  const splits = Array.isArray(body?.splits) ? body.splits : []
  const normalized = splits.map((s: any) => ({
    to: { owner: Principal.fromText(s.to), subaccount: [] },
    amount: BigInt(s.amount),
  }))
  const actor = await getPaymentsActor()
  const res = await (actor as any).release({ intent_id: params.id, splits: normalized })
  return NextResponse.json(res)
}

