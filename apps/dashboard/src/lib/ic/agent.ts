import { HttpAgent, Actor } from '@dfinity/agent'
import { idlFactory } from './payments.idl'

export type PaymentsActor = ReturnType<typeof Actor.createActor<ReturnType<typeof idlFactory>>>;

export async function getAgent() {
  const host = process.env.DFX_NETWORK === 'local' || process.env.NEXT_PUBLIC_DFX_NETWORK === 'local'
    ? 'http://127.0.0.1:4943'
    : undefined
  const agent = new HttpAgent({ host })
  if (host && process.env.NODE_ENV !== 'production') {
    // Fetch root key for local
    await agent.fetchRootKey()
  }
  return agent
}

export async function getPaymentsActor() {
  const canisterId = process.env.PAYMENTS_CANISTER_ID || process.env.NEXT_PUBLIC_PAYMENTS_CANISTER_ID
  if (!canisterId) throw new Error('PAYMENTS_CANISTER_ID is not set')
  const agent = await getAgent()
  const actor = Actor.createActor(idlFactory as any, { agent, canisterId }) as any
  return actor
}

