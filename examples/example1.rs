use std::arch::asm;

fn looper() {
    println!("Entering loop...");

    // Equivalent to loop { }
    unsafe {
        asm!(
            "123:",
            "b 123b"
        )
    }

    println!("Left loop...");
}

fn main() {
    println!("Example 1: Starting main...");

    looper();

    println!("*** ESCAPED **");
}
