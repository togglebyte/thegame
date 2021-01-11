use common::models::{Auth, Message};

fn main() {
    let msg = Message::Auth(Auth::SignIn("florp".into(), "blarp".into()));
    eprintln!("{:?}", Message::from_bytes(&msg.to_bytes()));
}
