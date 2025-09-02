use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::{caller, time};
use ic_cdk_macros::{query, update};
use once_cell::sync::Lazy;
use std::cell::RefCell;
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
}

static STATE: Lazy<RefCell<State>> = Lazy::new(|| RefCell::new(State::default()));

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
    ic_cdk::api::is_controller(*p)
}

// ===== Registry APIs =====

#[update]
pub fn register_ledger(asset: String, ledger_id: Principal, decimals: u8) -> Result<(), Error> {
    let caller = caller();
    if !is_controller(&caller) {
        return Err(Error::Unauthorized);
    }
    let mut st = STATE.borrow_mut();
    st.ledger_registry
        .insert(asset, LedgerInfo { ledger_id, decimals });
    Ok(())
}

#[query]
pub fn get_ledger(asset: String) -> Option<LedgerInfo> {
    STATE.borrow().ledger_registry.get(&asset).cloned()
}

// ===== Intent APIs =====

#[update]
pub fn create_intent(args: CreateIntentArgs) -> Result<PaymentIntent, Error> {
    // Validate asset
    if STATE.borrow().ledger_registry.get(&args.asset).is_none() {
        return Err(Error::AssetNotRegistered);
    }
    let merchant = caller();
    let now = time();
    if args.expires_at <= now {
        return Err(Error::Expired);
    }
    let mut st = STATE.borrow_mut();
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
    Ok(intent)
}

#[update]
pub fn capture(intent_id: String) -> Result<PaymentIntent, Error> {
    // MVP stub: only transition RequiresApproval -> Succeeded
    let now = time();
    let mut st = STATE.borrow_mut();
    let intent = st.intents.get_mut(&intent_id).ok_or(Error::NotFound)?;
    if intent.expires_at <= now {
        intent.status = IntentStatus::Expired;
        return Err(Error::Expired);
    }
    if intent.status != IntentStatus::RequiresApproval {
        return Err(Error::InvalidState);
    }
    // NOTE: In production, call ICRC-2 transfer_from here.
    intent.status = IntentStatus::Succeeded;
    Ok(intent.clone())
}

#[query]
pub fn get_intent(intent_id: String) -> Option<PaymentIntent> {
    STATE.borrow().intents.get(&intent_id).cloned()
}

// ===== Candid export =====

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    ic_cdk::export_candid!()
}

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

