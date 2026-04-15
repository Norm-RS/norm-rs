use lending_state_rs::LoanApplication;
use rfe_types::{rust_decimal::Decimal, ClientId};

fn main() {
    let draft = LoanApplication::new(ClientId::new(), Decimal::new(1000, 0));
    let scoring = draft.submit_to_scoring();
    let pdn = scoring.scoring_done(true).unwrap();

    let rejected = pdn.pdn_done(true).unwrap_err(); // It's Rejected now

    // ERROR: Rejected has no `disburse` method
    let disbursed = rejected.disburse();
}
