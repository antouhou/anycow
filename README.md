# 🐄 AnyCow

[![Crates.io](https://img.shields.io/crates/v/anycow.svg)](https://crates.io/crates/anycow)
[![Documentation](https://docs.rs/anycow/badge.svg)](https://docs.rs/anycow)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> A supercharged container for read-heavy, occasionally-updated data structures

**AnyCow** is a versatile, high-performance container that extends the concept of `Cow` (Clone-on-Write) with multiple storage strategies optimized for different use cases. Perfect for scenarios where you need to read values frequently but update them only occasionally.

## 🚀 Features

- **Multiple Storage Strategies**: Choose the right storage for your use case
  - `Borrowed` - Zero-cost references for temporary data
  - `Owned` - Heap-allocated owned data via `Box<T>`
  - `Shared` - `Arc<T>` for shared immutable data
  - `Updatable` - Lock-free atomic updates using `arc-swap`

- **Lock-Free Updates**: The `Updatable` variant uses `arc-swap` for atomic, lock-free updates
- **Flexible API**: Easy conversion between different storage types
- **Zero-Cost Abstractions**: Minimal overhead for common operations
- **Thread-Safe**: Share data safely across threads with `Shared` and `Updatable` variants

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anycow = "0.1"
```

## 🎯 Use Cases

AnyCow shines in scenarios where you have:

- **Configuration data** that's read frequently but updated occasionally
- **Cached values** that need atomic updates without locks
- **Shared state** across multiple threads with infrequent modifications
- **Hot paths** where you want to minimize allocation overhead
- **APIs** that need to accept both borrowed and owned data flexibly

## 🔥 Quick Start

```rust
use anycow::AnyCow;

// Create from different sources
let borrowed = AnyCow::borrowed(&"hello");
let owned = AnyCow::owned(String::from("world"));
let shared = AnyCow::shared(std::sync::Arc::new(42));
let updatable = AnyCow::updatable(vec![1, 2, 3]);

// Read values efficiently
println!("{}", *borrowed.borrow()); // "hello"
println!("{}", *owned.borrow());    // "world"

// Atomic updates (lock-free!)
updatable.try_replace(vec![4, 5, 6]).unwrap();
```

## 💡 Examples

### Reading Configuration

```rust
use anycow::AnyCow;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Config {
    max_connections: usize,
    timeout_ms: u64,
}

// Create updatable config
let config = AnyCow::updatable(Config {
    max_connections: 100,
    timeout_ms: 5000,
});

// Read frequently (very fast)
let current_config = config.borrow();
println!("Max connections: {}", current_config.max_connections);

// Update occasionally (atomic, lock-free)
config.try_replace(Config {
    max_connections: 200,
    timeout_ms: 3000,
}).unwrap();
```

### Flexible API Design

```rust
use anycow::AnyCow;

fn process_data<'a>(data: AnyCow<'a, str>) {
    // Works with borrowed, owned, or shared data
    println!("Processing: {}", *data.borrow());
}

// All of these work!
process_data(AnyCow::borrowed("borrowed string"));
process_data(AnyCow::owned(String::from("owned string")));
process_data(AnyCow::shared(std::sync::Arc::new(String::from("shared string"))));
```

### Cache with Atomic Updates

```rust
use anycow::AnyCow;
use std::thread;
use std::sync::Arc;

let cache = Arc::new(AnyCow::updatable(vec![1, 2, 3]));

// Spawn reader threads
let cache_clone = cache.clone();
let reader = thread::spawn(move || {
    for _ in 0..1000 {
        let data = cache_clone.borrow();
        println!("Sum: {}", data.iter().sum::<i32>());
    }
});

// Update cache atomically
cache.try_replace(vec![4, 5, 6, 7, 8]).unwrap();

reader.join().unwrap();
```

## 🧠 Storage Strategy Guide

| Variant | Best For | Thread Safe | Mutable | Memory |
|---------|----------|-------------|---------|--------|
| `Borrowed` | Temporary refs, hot paths | ❌ | ❌ | Zero-copy |
| `Owned` | Exclusive ownership | ❌ | ✅ | Heap |
| `Shared` | Read-only sharing | ✅ | ❌ | Shared |
| `Updatable` | Concurrent reads + atomic updates | ✅ | Via `try_replace()` | Shared + Atomic |

## 🔧 API Reference

### Construction
```rust
AnyCow::borrowed(&value)    // From reference
AnyCow::owned(value)        // From owned value (boxed)
AnyCow::shared(arc)         // From Arc<T>
AnyCow::updatable(value)    // Create updatable variant
```

### Access
```rust
container.borrow()          // Get reference to value
container.to_mut()          // Get mutable reference (COW)
container.into_owned()      // Convert to owned value
container.to_arc()          // Convert to Arc<T>
```

### Updates
```rust
container.try_replace(new_value)  // Atomic update (Updatable only)
```

## ⚡ Performance

AnyCow is designed for performance:

- **Zero-cost borrowing**: No allocation for `Borrowed` variant
- **Lock-free updates**: `Updatable` uses `arc-swap` for atomic operations
- **Minimal overhead**: Smart enum design with efficient memory layout
- **Branch prediction friendly**: Common operations are optimized

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [`arc-swap`](https://crates.io/crates/arc-swap) for lock-free atomic operations
- Inspired by the standard library's `Cow` but supercharged for modern use cases

---

Made with ❤️ for the Rust community. Happy coding! 🦀
