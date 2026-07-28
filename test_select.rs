#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::broadcast::channel::<String>(1);
    tx.send("A".into()).unwrap();
    tx.send("B".into()).unwrap();
    tx.send("C".into()).unwrap(); // This causes Lagged in rx later
    
    loop {
        tokio::select! {
            Ok(v) = rx.recv() => {
                println!("Got: {}", v);
            },
            else => {
                println!("Else branch!");
                break;
            }
        }
    }
}
