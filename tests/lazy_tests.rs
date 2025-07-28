use std::ops::Deref;
use anycow::AnyCow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

static INIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn expensive_init() -> Vec<i32> {
    INIT_COUNTER.fetch_add(1, Ordering::SeqCst);
    vec![1, 2, 3, 4, 5]
}

#[test]
fn test_lazy_initialization() {
    // Reset counter
    INIT_COUNTER.store(0, Ordering::SeqCst);
    
    // Create lazy AnyCow
    let lazy_cow = AnyCow::lazy(expensive_init);
    
    // Verify it's recognized as lazy
    assert!(lazy_cow.is_lazy());
    assert!(!lazy_cow.is_updatable());
    assert!(!lazy_cow.is_owned());
    assert!(!lazy_cow.is_shared());
    assert!(!lazy_cow.is_borrowed());
    
    // Check that init hasn't been called yet
    assert_eq!(INIT_COUNTER.load(Ordering::SeqCst), 0);
    
    // First access should initialize
    let value = lazy_cow.borrow();
    assert_eq!(*value, vec![1, 2, 3, 4, 5]);
    assert_eq!(INIT_COUNTER.load(Ordering::SeqCst), 1);
    
    // Second access should not re-initialize
    let value2 = lazy_cow.borrow();
    assert_eq!(*value2, vec![1, 2, 3, 4, 5]);
    assert_eq!(INIT_COUNTER.load(Ordering::SeqCst), 1);
}

#[test]
fn test_lazy_update() {
    let lazy_cow = AnyCow::lazy(|| vec![10, 20, 30]);
    
    // Initialize by reading
    assert_eq!(*lazy_cow.borrow(), vec![10, 20, 30]);
    
    // Update atomically
    assert!(lazy_cow.try_replace(vec![40, 50, 60]).is_ok());
    assert_eq!(*lazy_cow.borrow(), vec![40, 50, 60]);
    
    // Update again
    assert!(lazy_cow.try_replace(vec![70, 80, 90]).is_ok());
    assert_eq!(*lazy_cow.borrow(), vec![70, 80, 90]);
}

#[test]
fn test_lazy_const_context() {
    // This demonstrates that lazy can be used in const contexts
    const LAZY_CONST: AnyCow<String> = AnyCow::lazy(|| String::from("hello"));
    
    assert!(LAZY_CONST.is_lazy());
    assert_eq!(*LAZY_CONST.borrow(), "hello");
}

static GLOBAL_LAZY: AnyCow<i32> = AnyCow::lazy(|| 42);

#[test]
fn test_lazy_static_context() {
    // Test that lazy works in static contexts
    assert_eq!(*GLOBAL_LAZY.borrow(), 42);
    
    // Update the static
    GLOBAL_LAZY.try_replace(100).unwrap();
    assert_eq!(*GLOBAL_LAZY.borrow(), 100);
}

#[test]
fn test_lazy_clone() {
    let lazy_cow = AnyCow::lazy(|| String::from("original"));
    
    // Clone of lazy variant should always initialize and create an updatable
    let cloned = lazy_cow.clone();
    assert!(cloned.is_updatable());
    assert!(!cloned.is_lazy());
    
    // The original should still be lazy but now initialized
    assert!(lazy_cow.is_lazy());
    
    // Both should have the same data
    assert_eq!(*lazy_cow.borrow(), "original");
    assert_eq!(*cloned.borrow(), "original");
    
    // Clone after initialization should also create an updatable with the same data
    let cloned_after = lazy_cow.clone();
    assert!(cloned_after.is_updatable());
    assert_eq!(*cloned_after.borrow(), "original");
}

#[test]
fn test_lazy_into_owned() {
    let lazy_cow = AnyCow::lazy(|| String::from("test"));
    let owned = lazy_cow.into_owned();
    assert_eq!(owned, "test");
}

#[test]
fn test_lazy_to_arc() {
    let lazy_cow = AnyCow::lazy(|| String::from("arc test"));
    let arc = lazy_cow.to_arc();
    assert_eq!(*arc, "arc test");
}

#[test]
fn test_replace_clone() {
    let lazy_cow = AnyCow::lazy(|| String::from("replace test"));

    assert_eq!(lazy_cow.borrow().deref(), "replace test");

    // Replace with a new value
    let new_value = String::from("new value");
    assert!(lazy_cow.try_replace(new_value.clone()).is_ok());
    assert_eq!(*lazy_cow.borrow(), "new value");

    // Ensure the original value is still accessible
    let cloned = lazy_cow.clone();
    assert_eq!(*cloned.borrow(), "new value");

    assert!(lazy_cow.try_replace(String::from("test value")).is_ok());
    assert_eq!(*lazy_cow.borrow(), "test value");
    assert_eq!(*cloned.borrow(), "new value");
}

#[test]
fn test_to_shared_borrowed() {
    let data = "hello";
    let borrowed = AnyCow::borrowed(&data);
    let shared = borrowed.to_shared();
    
    // Should still be borrowed (zero-cost)
    assert!(shared.is_borrowed());
    assert_eq!(*shared.borrow(), "hello");
}

#[test]
fn test_to_shared_owned() {
    let owned = AnyCow::owned(String::from("world"));
    let shared = owned.to_shared();
    
    // Should now be shared
    assert!(shared.is_shared());
    assert_eq!(*shared.borrow(), "world");
}

#[test]
fn test_to_shared_shared() {
    let arc_data = Arc::new(String::from("shared"));
    let shared = AnyCow::shared(arc_data);
    let new_shared = shared.to_shared();
    
    // Should remain shared
    assert!(new_shared.is_shared());
    assert_eq!(*new_shared.borrow(), "shared");
}

#[test]
fn test_to_shared_updatable() {
    let updatable = AnyCow::updatable(vec![1, 2, 3]);
    let shared = updatable.to_shared();
    
    // Should now be shared
    assert!(shared.is_shared());
    assert_eq!(*shared.borrow(), vec![1, 2, 3]);
}

#[test]
fn test_to_shared_lazy() {
    let lazy = AnyCow::lazy(|| vec![4, 5, 6]);
    let shared = lazy.to_shared();
    
    // Should now be shared
    assert!(shared.is_shared());
    assert_eq!(*shared.borrow(), vec![4, 5, 6]);
}

#[test]
fn test_to_shared_vs_to_arc() {
    let data = "test";
    let borrowed = AnyCow::borrowed(&data);
    
    // to_shared keeps borrowed as borrowed
    let shared = borrowed.to_shared();
    assert!(shared.is_borrowed());
    
    // to_arc converts borrowed to owned Arc
    let arc = borrowed.to_arc();
    // We can't check if it's shared because to_arc returns Arc<T>, not AnyCow
    assert_eq!(*arc, "test");
}
