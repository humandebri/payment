import { IDL } from '@dfinity/candid'

// Minimal IDL factory matching canisters/payments/payments.did
export const idlFactory = ({ IDL: I = IDL }) => {
  const Account = I.Record({ owner: I.Principal, subaccount: I.Opt(I.Vec(I.Nat8)) })
  const LedgerInfo = I.Record({ ledger_id: I.Principal, decimals: I.Nat8 })
  const IntentStatus = I.Variant({ RequiresApproval: I.Null, Succeeded: I.Null, Released: I.Null, Refunded: I.Null, Expired: I.Null })
  const PaymentIntent = I.Record({
    id: I.Text,
    merchant: I.Principal,
    payer: I.Opt(Account),
    escrow: Account,
    asset: I.Text,
    amount: I.Nat,
    status: IntentStatus,
    created_at: I.Nat64,
    expires_at: I.Nat64,
    metadata: I.Vec(I.Tuple(I.Text, I.Text)),
  })
  const CreateIntentArgs = I.Record({ asset: I.Text, amount: I.Nat, expires_at: I.Nat64, metadata: I.Vec(I.Tuple(I.Text, I.Text)) })
  const Error = I.Variant({
    AssetNotRegistered: I.Null,
    NotFound: I.Null,
    InvalidState: I.Null,
    Expired: I.Null,
    Unauthorized: I.Null,
    Other: I.Text,
  })
  const Split = I.Record({ to: Account, amount: I.Nat })
  const CaptureArgs = I.Record({ intent_id: I.Text, from: Account })
  const ReleaseArgs = I.Record({ intent_id: I.Text, splits: I.Vec(Split) })
  const RefundArgs = I.Record({ intent_id: I.Text, amount: I.Nat })
  const EventKind = I.Variant({
    IntentCreated: I.Record({ id: I.Text }),
    Captured: I.Record({ id: I.Text, amount: I.Nat }),
    Released: I.Record({ id: I.Text, total: I.Nat }),
    Refunded: I.Record({ id: I.Text, amount: I.Nat }),
    Expired: I.Record({ id: I.Text }),
  })
  const Event = I.Record({ ts: I.Nat64, kind: EventKind })
  const CertifiedEvents = I.Record({
    events: I.Vec(Event),
    prev_prefix: I.Opt(I.Vec(I.Nat8)),
    tip_prefix: I.Opt(I.Vec(I.Nat8)),
    certificate: I.Opt(I.Vec(I.Nat8)),
  })
  return I.Service({
    register_ledger: I.Func([I.Text, I.Principal, I.Nat8], [I.Variant({ ok: I.Null, err: Error })], []),
    get_ledger: I.Func([I.Text], [I.Opt(LedgerInfo)], ['query']),
    create_intent: I.Func([CreateIntentArgs], [I.Variant({ ok: PaymentIntent, err: Error })], []),
    capture: I.Func([CaptureArgs], [I.Variant({ ok: PaymentIntent, err: Error })], []),
    release: I.Func([ReleaseArgs], [I.Variant({ ok: PaymentIntent, err: Error })], []),
    refund: I.Func([RefundArgs], [I.Variant({ ok: PaymentIntent, err: Error })], []),
    get_intent: I.Func([I.Text], [I.Opt(PaymentIntent)], ['query']),
    list_events: I.Func([I.Nat64, I.Nat32], [I.Vec(Event)], ['query']),
    list_events_certified_from: I.Func([I.Nat64, I.Nat32], [CertifiedEvents], ['query'])
  })
}

