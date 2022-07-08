// Should derive the state just fine
// This should allow to transform :
// * from MyAppState to Database
// * from MyAppState to HttpClient

use derive_state::State;

pub struct Database;
pub struct HttpClient;

#[derive(State)]
pub struct MyAppState {
    pub database: Database,
    pub http_client: HttpClient,
}

fn test_impl_from<F, T: From<F>>() {}

fn main() {
    test_impl_from::<MyAppState, Database>();
    test_impl_from::<MyAppState, HttpClient>();
}
