import { NextResponse } from 'next/server'
import { getPaymentsActor } from '../../../../../src/lib/ic/agent'
import { Principal } from '@dfinity/principal'

export async function POST(req: Request, { params }: { params: { id: string } }) {
  const body = await req.json().catch(() => ({}))
  const ownerText = body?.owner as string
  const subHex = (body?.subaccount as string | undefined)?.trim()
  if (!ownerText) return NextResponse.json({ err: 'owner required' }, { status: 400 })
  let sub: number[] | undefined
  if (subHex) {
    const clean = subHex.replace(/^0x/, '')
    if (clean.length % 2 !== 0) return NextResponse.json({ err: 'invalid subaccount hex' }, { status: 400 })
    sub = Array.from(Buffer.from(clean, 'hex'))
  }
  const actor = await getPaymentsActor()
  const res = await (actor as any).capture({ intent_id: params.id, from: { owner: Principal.fromText(ownerText), subaccount: sub ? [sub] : [] } })
  return NextResponse.json(res)
}

