use anycow::AnyCow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

// Global configuration that's lazily initialized
static CONFIG: AnyCow<HashMap<String, String>> = AnyCow::lazy(|| {
    println!("Initializing global config...");
    let mut config = HashMap::new();
    config.insert("app_name".to_string(), "MyApp".to_string());
    config.insert("version".to_string(), "1.0.0".to_string());
    config.insert("debug".to_string(), "false".to_string());
    config
});

// A counter to track how many times expensive computation runs
static COMPUTATION_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Expensive computation that we want to lazy-load and cache
static EXPENSIVE_RESULT: AnyCow<Vec<u64>> = AnyCow::lazy(|| {
    let count = COMPUTATION_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("Running expensive computation #{}", count + 1);

    // Simulate expensive computation
    (1..=10).map(|i| i * i * i).collect()
});

fn main() {
    println!("=== AnyCow Lazy Example ===\n");

    // 1. Demonstrate that lazy initialization doesn't happen until first access
    println!("1. Created lazy statics, but nothing initialized yet");
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 2. First access to CONFIG initializes it
    println!("2. First access to CONFIG:");
    let app_name = CONFIG.borrow().get("app_name").cloned().unwrap_or_default();
    println!("   App name: {app_name}");
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 3. Subsequent accesses don't re-initialize
    println!("3. Second access to CONFIG (no re-initialization):");
    let version = CONFIG.borrow().get("version").cloned().unwrap_or_default();
    println!("   Version: {version}");
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 4. Update the config atomically
    println!("4. Updating CONFIG atomically:");
    let mut new_config = HashMap::new();
    new_config.insert("app_name".to_string(), "MyApp Pro".to_string());
    new_config.insert("version".to_string(), "2.0.0".to_string());
    new_config.insert("debug".to_string(), "true".to_string());
    new_config.insert("theme".to_string(), "dark".to_string());

    CONFIG.try_replace(new_config).unwrap();

    let updated_app_name = CONFIG.borrow().get("app_name").cloned().unwrap_or_default();
    let theme = CONFIG.borrow().get("theme").cloned().unwrap_or_default();
    println!("   Updated app name: {updated_app_name}");
    println!("   New theme: {theme}");
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 5. Now access the expensive computation for the first time
    println!("5. First access to EXPENSIVE_RESULT (will initialize):");
    let result = EXPENSIVE_RESULT.borrow();
    println!("   Cubes: {:?}", *result);
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 6. Access it again (no re-computation)
    println!("6. Second access to EXPENSIVE_RESULT (no re-computation):");
    let result2 = EXPENSIVE_RESULT.borrow();
    println!("   Cubes again: {:?}", *result2);
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 7. Update the expensive result
    println!("7. Updating EXPENSIVE_RESULT:");
    let new_result: Vec<u64> = (1..=5).map(|i| i * i).collect();
    EXPENSIVE_RESULT.try_replace(new_result).unwrap();

    let updated_result = EXPENSIVE_RESULT.borrow();
    println!("   Updated to squares: {:?}", *updated_result);
    println!(
        "   Computation counter: {}\n",
        COMPUTATION_COUNTER.load(Ordering::SeqCst)
    );

    // 8. Demonstrate local lazy usage
    println!("8. Local lazy usage:");
    let local_lazy = AnyCow::lazy(|| {
        println!("   Initializing local lazy value...");
        String::from("Hello from local lazy!")
    });

    println!("   Created local lazy (not initialized yet)");
    println!("   Value: {}", *local_lazy.borrow());
    println!("   Second access: {}", *local_lazy.borrow());

    println!("\n=== Summary ===");
    println!("- Lazy initialization only happens on first access");
    println!("- Subsequent accesses are fast (no re-initialization)");
    println!("- Atomic updates work just like with Updatable variant");
    println!("- Perfect for static/const contexts");
    println!("- Thread-safe and lock-free");
}
