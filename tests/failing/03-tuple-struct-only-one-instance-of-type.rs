//! Only one instance of a type is allowed in the state

use derive_state::State;

#[derive(State)]
pub struct MyState(String, String);

fn main() {}
