use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::{caller, data_certificate, set_certified_data, time};
use ic_cdk::call;
use ic_cdk_macros::{query, update};
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::collections::HashMap;

// ===== Types =====

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<[u8; 32]>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LedgerInfo {
    pub ledger_id: Principal,
    pub decimals: u8,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IntentStatus {
    RequiresApproval,
    Succeeded,
    Released,
    Refunded,
    Expired,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaymentIntent {
    pub id: String,
    pub merchant: Principal,
    pub payer: Option<Account>,
    pub escrow: Account,
    pub asset: String,
    pub amount: Nat,
    pub status: IntentStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub metadata: Vec<(String, String)>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateIntentArgs {
    pub asset: String,
    pub amount: Nat,
    pub expires_at: u64,
    pub metadata: Vec<(String, String)>,
}

#[derive(thiserror::Error, Debug, CandidType, Deserialize)]
pub enum Error {
    #[error("asset not registered")]
    AssetNotRegistered,
    #[error("intent not found")]
    NotFound,
    #[error("invalid state for operation")]
    InvalidState,
    #[error("expired")]
    Expired,
    #[error("unauthorized")]
    Unauthorized,
    #[error("other: {0}")]
    Other(String),
}

// ===== State =====

#[derive(Default)]
struct State {
    next_seq: u64,
    ledger_registry: HashMap<String, LedgerInfo>,
    intents: HashMap<String, PaymentIntent>,
    events: Vec<Event>,
    event_hashes: Vec<[u8; 32]>,
    event_prefixes: Vec<[u8; 32]>,
}

static STATE: Lazy<RwLock<State>> = Lazy::new(|| RwLock::new(State::default()));

// ===== Helpers =====

fn derive_escrow_subaccount(intent_id: &str) -> [u8; 32] {
    use blake3::Hasher;
    let mut h = Hasher::new();
    h.update(b"payments/escrow");
    h.update(intent_id.as_bytes());
    let hash = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

fn canister_account(sub: Option<[u8; 32]>) -> Account {
    Account {
        owner: ic_cdk::api::id(),
        subaccount: sub,
    }
}

fn is_controller(p: &Principal) -> bool {
    ic_cdk::api::is_controller(p)
}

// ===== ICRC Types (minimal) =====

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Icrc1TransferArg {
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Icrc1TransferError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    BadBurn { min_burn_amount: Nat },
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    InsufficientFunds { balance: Nat },
    TxTooLarge { allowed_size: Nat },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Icrc2TransferFromArg {
    pub from: Account,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub spender_subaccount: Option<Vec<u8>>,
    pub expected_allowance: Option<Nat>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum Icrc2TransferFromError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    InsufficientAllowance { allowance: Nat },
    BadFee { expected_fee: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TxTooLarge { allowed_size: Nat },
}

// ===== Events =====

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum EventKind {
    IntentCreated { id: String },
    Captured { id: String, amount: Nat },
    Released { id: String, total: Nat },
    Refunded { id: String, amount: Nat },
    Expired { id: String },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Event {
    pub ts: u64,
    pub kind: EventKind,
}

fn event_hash(e: &Event) -> [u8; 32] {
    let bytes = candid::encode_one(e).unwrap_or_default();
    let h = blake3::hash(&bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(h.as_bytes());
    out
}

fn update_certified_tip(st: &State) {
    let tip = st.event_prefixes.last().cloned().unwrap_or([0u8; 32]);
    set_certified_data(&tip);
}

fn append_event(kind: EventKind) {
    let mut st = STATE.write().unwrap();
    let e = Event { ts: time(), kind };
    let eh = event_hash(&e);
    let prev = st.event_prefixes.last().cloned().unwrap_or([0u8; 32]);
    let mut hasher = blake3::Hasher::new();
    hasher.update(&prev);
    hasher.update(&eh);
    let nh = hasher.finalize();
    let mut prefix = [0u8; 32];
    prefix.copy_from_slice(nh.as_bytes());
    st.events.push(e);
    st.event_hashes.push(eh);
    st.event_prefixes.push(prefix);
    update_certified_tip(&st);
}

#[query]
pub fn list_events(offset: u64, limit: u32) -> Vec<Event> {
    let st = STATE.read().unwrap();
    let start = offset as usize;
    let end = (start + limit as usize).min(st.events.len());
    st.events[start..end].to_vec()
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CertifiedEvents {
    pub events: Vec<Event>,
    pub prev_prefix: Option<Vec<u8>>, // 32 bytes
    pub tip_prefix: Option<Vec<u8>>,  // 32 bytes
    pub certificate: Option<Vec<u8>>,
}

#[query]
pub fn list_events_certified_from(offset: u64, limit: u32) -> CertifiedEvents {
    let st = STATE.read().unwrap();
    let start = offset as usize;
    let end = (start + limit as usize).min(st.events.len());
    let events = st.events[start..end].to_vec();
    let prev_prefix = if start == 0 {
        None
    } else {
        Some(st.event_prefixes[start - 1].to_vec())
    };
    let tip_prefix = st.event_prefixes.last().map(|x| x.to_vec());
    let certificate = data_certificate();
    CertifiedEvents {
        events,
        prev_prefix,
        tip_prefix,
        certificate,
    }
}

// ===== Registry APIs =====

#[update]
pub fn register_ledger(asset: String, ledger_id: Principal, decimals: u8) -> Result<(), Error> {
    let caller = caller();
    if !is_controller(&caller) {
        return Err(Error::Unauthorized);
    }
    let mut st = STATE.write().unwrap();
    st.ledger_registry
        .insert(asset, LedgerInfo { ledger_id, decimals });
    Ok(())
}

#[query]
pub fn get_ledger(asset: String) -> Option<LedgerInfo> {
    STATE.read().unwrap().ledger_registry.get(&asset).cloned()
}

// ===== Intent APIs =====

#[update]
pub fn create_intent(args: CreateIntentArgs) -> Result<PaymentIntent, Error> {
    // Validate asset
    if STATE.read().unwrap().ledger_registry.get(&args.asset).is_none() {
        return Err(Error::AssetNotRegistered);
    }
    let merchant = caller();
    let now = time();
    if args.expires_at <= now {
        return Err(Error::Expired);
    }
    let mut st = STATE.write().unwrap();
    // Generate ID: seq + merchant + now
    let seq = {
        let s = st.next_seq;
        st.next_seq += 1;
        s
    };
    let id = format!("pi_{}_{}_{}", seq, merchant.to_text(), now);
    let sub = derive_escrow_subaccount(&id);
    let escrow = canister_account(Some(sub));
    let intent = PaymentIntent {
        id: id.clone(),
        merchant,
        payer: None,
        escrow,
        asset: args.asset,
        amount: args.amount,
        status: IntentStatus::RequiresApproval,
        created_at: now,
        expires_at: args.expires_at,
        metadata: args.metadata,
    };
    st.intents.insert(id.clone(), intent.clone());
    append_event(EventKind::IntentCreated { id: id.clone() });
    Ok(intent)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CaptureArgs {
    pub intent_id: String,
    pub from: Account,
}

#[update]
pub async fn capture(args: CaptureArgs) -> Result<PaymentIntent, Error> {
    let now = time();
    let intent_id = args.intent_id.clone();
    {
        let mut st = STATE.write().unwrap();
        let intent = st.intents.get_mut(&intent_id).ok_or(Error::NotFound)?;
        if intent.merchant != caller() {
            return Err(Error::Unauthorized);
        }
        if intent.expires_at <= now {
            intent.status = IntentStatus::Expired;
            return Err(Error::Expired);
        }
        if intent.status != IntentStatus::RequiresApproval {
            return Err(Error::InvalidState);
        }
    }
    // Perform icrc2_transfer_from(from -> escrow)
    let (ledger, amount, escrow, asset) = {
        let st = STATE.read().unwrap();
        let intent = st.intents.get(&intent_id).unwrap();
        let ledger = st
            .ledger_registry
            .get(&intent.asset)
            .ok_or(Error::AssetNotRegistered)?
            .ledger_id;
        (ledger, intent.amount.clone(), intent.escrow.clone(), intent.asset.clone())
    };

    let arg = Icrc2TransferFromArg {
        from: args.from.clone(),
        to: escrow.clone(),
        amount: amount.clone(),
        fee: None,
        memo: None,
        created_at_time: Some(now),
        spender_subaccount: None,
        expected_allowance: None,
    };

    let res: (Result<Nat, Icrc2TransferFromError>,) = call(ledger, "icrc2_transfer_from", (arg,))
        .await
        .map_err(|e| Error::Other(format!("ledger call error: {}", e.1)))?;

    match res.0 {
        Ok(_block_idx) => {
            let mut st = STATE.write().unwrap();
            let intent = st.intents.get_mut(&intent_id).ok_or(Error::NotFound)?;
            // 単発フルキャプチャのみ許容
            intent.payer = Some(args.from);
            intent.status = IntentStatus::Succeeded;
            append_event(EventKind::Captured { id: intent_id.clone(), amount });
            Ok(intent.clone())
        }
        Err(e) => Err(Error::Other(format!("icrc2_transfer_from error: {:?}", e))),
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Split {
    pub to: Account,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ReleaseArgs {
    pub intent_id: String,
    pub splits: Vec<Split>,
}

#[update]
pub async fn release(args: ReleaseArgs) -> Result<PaymentIntent, Error> {
    // 前提: 単発フルキャプチャ後の一括リリース
    let now = time();
    let intent_id = args.intent_id.clone();
    let (ledger, escrow, amount_total, asset, merchant) = {
        let st = STATE.read().unwrap();
        let intent = st.intents.get(&intent_id).ok_or(Error::NotFound)?;
        if intent.expires_at <= now {
            return Err(Error::Expired);
        }
        if intent.status != IntentStatus::Succeeded {
            return Err(Error::InvalidState);
        }
        let ledger = st
            .ledger_registry
            .get(&intent.asset)
            .ok_or(Error::AssetNotRegistered)?
            .ledger_id;
        (
            ledger,
            intent.escrow.clone(),
            intent.amount.clone(),
            intent.asset.clone(),
            intent.merchant,
        )
    };
    if merchant != caller() {
        return Err(Error::Unauthorized);
    }

    // 簡易検証: splits 合計 <= intent.amount
    let mut sum: Nat = Nat::from(0u32);
    for s in &args.splits {
        sum += s.amount.clone();
    }
    if sum > amount_total {
        return Err(Error::Other("splits total exceeds captured amount".into()));
    }

    // escrow -> 各受益者へ ICRC-1 transfer（手数料は escrow から控除）
    for s in args.splits.iter() {
        let arg = Icrc1TransferArg {
            to: s.to.clone(),
            amount: s.amount.clone(),
            fee: None,
            memo: None,
            from_subaccount: escrow.subaccount.map(|x| x.to_vec()),
            created_at_time: Some(now),
        };
        let res: (Result<Nat, Icrc1TransferError>,) =
            call(ledger, "icrc1_transfer", (arg,))
                .await
                .map_err(|e| Error::Other(format!("ledger call error: {}", e.1)))?;
        if let Err(e) = res.0 {
            return Err(Error::Other(format!("icrc1_transfer error: {:?}", e)));
        }
    }

    let mut st = STATE.write().unwrap();
    let intent = st.intents.get_mut(&intent_id).ok_or(Error::NotFound)?;
    intent.status = IntentStatus::Released;
    append_event(EventKind::Released { id: intent_id.clone(), total: sum });
    Ok(intent.clone())
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RefundArgs {
    pub intent_id: String,
    pub amount: Nat,
}

#[update]
pub async fn refund(args: RefundArgs) -> Result<PaymentIntent, Error> {
    // MVP: フル返金のみ（amount == intent.amount）
    let now = time();
    let intent_id = args.intent_id.clone();
    let (ledger, escrow, payer, amount, status, merchant) = {
        let st = STATE.read().unwrap();
        let intent = st.intents.get(&intent_id).ok_or(Error::NotFound)?;
        if intent.expires_at <= now {
            return Err(Error::Expired);
        }
        (
            st
                .ledger_registry
                .get(&intent.asset)
                .ok_or(Error::AssetNotRegistered)?
                .ledger_id,
            intent.escrow.clone(),
            intent.payer.clone(),
            intent.amount.clone(),
            intent.status.clone(),
            intent.merchant,
        )
    };
    if merchant != caller() {
        return Err(Error::Unauthorized);
    }

    if status != IntentStatus::Succeeded {
        return Err(Error::InvalidState);
    }
    if args.amount != amount {
        return Err(Error::Other("only full refund supported in MVP".into()));
    }
    let payer = payer.ok_or(Error::Other("payer unknown for refund".into()))?;

    // escrow -> payer へ返金
    let arg = Icrc1TransferArg {
        to: payer,
        amount: amount.clone(),
        fee: None,
        memo: None,
        from_subaccount: escrow.subaccount.map(|x| x.to_vec()),
        created_at_time: Some(now),
    };
    let res: (Result<Nat, Icrc1TransferError>,) =
        call(ledger, "icrc1_transfer", (arg,))
            .await
            .map_err(|e| Error::Other(format!("ledger call error: {}", e.1)))?;
    if let Err(e) = res.0 {
        return Err(Error::Other(format!("icrc1_transfer error: {:?}", e)));
    }

    let mut st = STATE.write().unwrap();
    let intent = st.intents.get_mut(&intent_id).ok_or(Error::NotFound)?;
    intent.status = IntentStatus::Refunded;
    append_event(EventKind::Refunded { id: intent_id.clone(), amount });
    Ok(intent.clone())
}

#[query]
pub fn get_intent(intent_id: String) -> Option<PaymentIntent> {
    let mut st = STATE.write().unwrap();
    if let Some(intent) = st.intents.get_mut(&intent_id) {
        let now = time();
        if intent.status == IntentStatus::RequiresApproval && intent.expires_at <= now {
            intent.status = IntentStatus::Expired;
            append_event(EventKind::Expired { id: intent_id.clone() });
        }
        return Some(intent.clone());
    }
    None
}

// ===== Candid export =====

// candid の動的エクスポート関数は省略（DID は静的ファイルを使用）

// ===== Tests (unit) =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subaccount_is_deterministic() {
        let a = derive_escrow_subaccount("x");
        let b = derive_escrow_subaccount("x");
        assert_eq!(a, b);
    }
}
