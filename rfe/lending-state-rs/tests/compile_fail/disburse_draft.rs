use lending_state_rs::LoanApplication;
use rfe_types::{rust_decimal::Decimal, ClientId};

fn main() {
    let draft = LoanApplication::new(ClientId::new(), Decimal::new(1000, 0));

    // ERROR: Draft has no `disburse` method, only Approved does.
    let disbursed = draft.disburse();
}
