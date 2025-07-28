use anycow::AnyCow;

// Lazy initialization works great in static contexts!
static GLOBAL_CONFIG: AnyCow<Vec<i32>> = AnyCow::lazy(|| vec![1, 2, 3]);

// Borrowed can also be used in const contexts
const BORROWED_STR: AnyCow<&str> = AnyCow::borrowed(&"hello world");

fn main() {
    println!("Const functions example:");
    
    // Access the borrowed const
    println!("Borrowed const: {}", *BORROWED_STR.borrow());
    
    // First access to lazy initializes it
    println!("Global config (first access): {:?}", *GLOBAL_CONFIG.borrow());
    
    // Update the lazy value atomically
    GLOBAL_CONFIG.try_replace(vec![4, 5, 6, 7]).unwrap();
    println!("Global config (after update): {:?}", *GLOBAL_CONFIG.borrow());
    
    // Another update
    GLOBAL_CONFIG.try_replace(vec![8, 9, 10]).unwrap();
    println!("Global config (final): {:?}", *GLOBAL_CONFIG.borrow());
}