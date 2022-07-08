// Should derive the state just fine

use derive_state::State;

pub struct Database;
pub struct HttpClient;

#[derive(State)]
pub struct MyAppState(Database, HttpClient);

fn test_impl_from<F, T: From<F>>() {}

fn main() {
    test_impl_from::<MyAppState, Database>();
    test_impl_from::<MyAppState, HttpClient>();
}
