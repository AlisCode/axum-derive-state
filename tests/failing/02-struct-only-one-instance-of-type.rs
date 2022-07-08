//! Only one instance of a type is allowed in a struct
//! The error should provide a helpful hint by suggesting to create an appropriate newtype

use derive_state::State;

#[derive(State)]
pub struct MyState {
    database: String,
    http_client: String,
}

fn main() {}
