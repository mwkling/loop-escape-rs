fn looper() {
    println!("Entering loop...");

    loop { }

    println!("Left loop...");
}

fn main() {
    println!("Example 2: Starting main...");

    looper();

    println!("*** ESCAPED **");
}
