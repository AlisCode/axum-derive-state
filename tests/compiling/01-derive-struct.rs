// Should derive the state just fine

use derive_state::State;

pub struct Database;
pub struct HttpClient;

#[derive(State)]
pub struct MyAppState {
    pub database: Database,
    pub http_client: HttpClient,
}

fn main() {}
