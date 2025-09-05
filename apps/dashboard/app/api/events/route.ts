import { NextResponse } from 'next/server'
import { getPaymentsActor } from '../../../src/lib/ic/agent'

export async function GET(req: Request) {
  const { searchParams } = new URL(req.url)
  const offset = BigInt(searchParams.get('offset') ?? '0')
  const limit = Number(searchParams.get('limit') ?? '50')
  const actor = await getPaymentsActor()
  const res = await (actor as any).list_events_certified_from(offset, limit)
  return NextResponse.json(res)
}

