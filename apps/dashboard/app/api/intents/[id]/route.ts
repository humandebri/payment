import { NextResponse } from 'next/server'
import { getPaymentsActor } from '../../../../src/lib/ic/agent'

export async function GET(_req: Request, { params }: { params: { id: string } }) {
  const actor = await getPaymentsActor()
  const res = await (actor as any).get_intent(params.id)
  return NextResponse.json(res)
}

