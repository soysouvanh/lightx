use tracing::{info_span, field};

fn main() {
    let span = info_span!("my_span", user.id = field::Empty);
    let _e = span.enter();
    
    tracing::Span::current().record("user.id", &"123");
    println!("Compiled and ran successfully!");
}
