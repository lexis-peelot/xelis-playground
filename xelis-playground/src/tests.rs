use indexmap::IndexMap;
use xelis_common::config::MAX_GAS_USAGE_PER_TX;

use crate::Silex;

/// Helper to compile and run Silex code, returning the execution result
async fn run_silex_code(code: &str) -> crate::ExecutionResult {
    let silex = Silex::new();
    let program = silex.compile_internal(code).expect("Failed to compile the program");
    let entries = program.entries();
    let e = entries.get(0).expect("No entry found");
    silex
        .execute_program_internal(
            program,
            e.chunk_id,
            Some(MAX_GAS_USAGE_PER_TX),
            IndexMap::new(),
            vec![],
            vec![],
        )
        .await
        .expect("Failed to execute the program")
}

/// Helper to compile and run Silex code, expecting success (return 0)
async fn run_silex_code_expect_success(code: &str) {
    let result = run_silex_code(code).await;
    assert_eq!(
        result.value(),
        "0",
        "Expected return value 0, got {}. Logs: {:?}",
        result.value(),
        result.logs()
    );
}

#[tokio::test]
async fn test_hello_world() {
    let code = r#"
        entry hello_world() {
            println("Hello, world!");
            return 0;
        }
    "#;

    let result = run_silex_code(code).await;
    assert_eq!(result.value(), "0");
}

// ============================================================================
// BTreeStore Tests
// ============================================================================

#[tokio::test]
async fn test_btree_store_new() {
    let code = r#"
        entry test_btree_new() {
            let store = BTreeStore::new(b"test_namespace");
            let len = store.len();
            if len != 0 {
                return 1;
            }
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_insert_and_len() {
    let code = r#"
        entry test_btree_insert() {
            let store = BTreeStore::new(b"test");
            
            // Insert values
            store.insert(b"key1", "hello");
            store.insert(b"key2", 42u64);
            store.insert(b"key3", true);
            
            // Check length
            if store.len() != 3 {
                return 1;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_get_returns_optional() {
    let code = r#"
        entry test_btree_get() {
            let store = BTreeStore::new(b"test");
            
            // Get from empty store returns null
            let val = store.get(b"nonexistent");
            if val.is_some() {
                return 1;
            }
            
            // Insert and get
            store.insert(b"key", 123u64);
            let val2 = store.get(b"key");
            if val2.is_none() {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_len() {
    let code = r#"
        entry test_btree_len() {
            let store = BTreeStore::new(b"test");
            
            if store.len() != 0 {
                return 1;
            }
            
            store.insert(b"a", 1u64);
            if store.len() != 1 {
                return 2;
            }
            
            store.insert(b"b", 2u64);
            if store.len() != 2 {
                return 3;
            }
            
            store.insert(b"c", 3u64);
            if store.len() != 3 {
                return 4;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_delete() {
    let code = r#"
        entry test_btree_delete() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"key1", 100u64);
            store.insert(b"key2", 200u64);
            
            if store.len() != 2 {
                return 1;
            }
            
            // Delete returns the old value (optional<any>)
            let deleted = store.delete(b"key1");
            if deleted.is_none() {
                return 2;
            }
            
            if store.len() != 1 {
                return 3;
            }
            
            // Getting deleted key should return null
            let val = store.get(b"key1");
            if val.is_some() {
                return 4;
            }
            
            // Other key should still exist
            let val2 = store.get(b"key2");
            if val2.is_none() {
                return 5;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_update() {
    let code = r#"
        entry test_btree_update() {
            let store = BTreeStore::new(b"test");
            
            // Insert first value
            store.insert(b"key", "original");
            
            if store.len() != 1 {
                return 1;
            }
            
            // Insert with same key creates a new entry (duplicate keys allowed)
            // Note: BTreeStore allows duplicate keys
            store.insert(b"key", "updated");
            
            // Length increases because duplicates are allowed
            if store.len() != 2 {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_seek_first() {
    let code = r#"
        entry test_btree_seek_first() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"b", 2u64);
            store.insert(b"a", 1u64);
            store.insert(b"c", 3u64);
            
            // Seek to first element (ascending)
            let result = store.seek(b"", BTreeSeekBias::First, true);
            if result.is_none() {
                return 1;
            }
            
            // Unwrap the tuple
            let tuple = result.unwrap();
            let cursor = tuple.0;
            let item = tuple.1;
            
            // First key should be "a" (lexicographically smallest)
            if item.key != b"a" {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_seek_last() {
    let code = r#"
        entry test_btree_seek_last() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"b", 2u64);
            store.insert(b"a", 1u64);
            store.insert(b"c", 3u64);
            
            // Seek to last element
            let result = store.seek(b"", BTreeSeekBias::Last, false);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let item = tuple.1;
            
            // Last key should be "c" (lexicographically largest)
            if item.key != b"c" {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_cursor_iteration() {
    let code = r#"
        entry test_btree_cursor_iteration() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"a", 1u64);
            store.insert(b"b", 2u64);
            store.insert(b"c", 3u64);
            
            let result = store.seek(b"", BTreeSeekBias::First, true);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let cursor = tuple.0;
            let first_item = tuple.1;
            
            if first_item.key != b"a" {
                return 2;
            }
            
            // Move to next
            let next = cursor.next();
            if next.is_none() {
                return 3;
            }
            
            let next_item = next.unwrap();
            if next_item.key != b"b" {
                return 4;
            }
            
            // Move to next again
            let next2 = cursor.next();
            if next2.is_none() {
                return 5;
            }
            
            let next_item2 = next2.unwrap();
            if next_item2.key != b"c" {
                return 6;
            }
            
            // No more elements
            let next3 = cursor.next();
            if next3.is_some() {
                return 7;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_seek_exact() {
    let code = r#"
        entry test_btree_seek_exact() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"apple", 1u64);
            store.insert(b"banana", 2u64);
            store.insert(b"cherry", 3u64);
            
            // Seek exact match
            let result = store.seek(b"banana", BTreeSeekBias::Exact, true);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let item = tuple.1;
            
            if item.key != b"banana" {
                return 2;
            }
            
            // Seek non-existent key (exact) should return None
            let result2 = store.seek(b"blueberry", BTreeSeekBias::Exact, true);
            if result2.is_some() {
                return 3;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_seek_greater_or_equal() {
    let code = r#"
        entry test_btree_seek_ge() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"a", 1u64);
            store.insert(b"c", 3u64);
            store.insert(b"e", 5u64);
            
            // Seek >= "b" should find "c"
            let result = store.seek(b"b", BTreeSeekBias::GreaterOrEqual, true);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let item = tuple.1;
            
            if item.key != b"c" {
                return 2;
            }
            
            // Seek >= "c" should find "c" (exact match)
            let result2 = store.seek(b"c", BTreeSeekBias::GreaterOrEqual, true);
            if result2.is_none() {
                return 3;
            }
            
            let tuple2 = result2.unwrap();
            let item2 = tuple2.1;
            
            if item2.key != b"c" {
                return 4;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_multiple_namespaces() {
    let code = r#"
        entry test_btree_namespaces() {
            let store1 = BTreeStore::new(b"namespace1");
            let store2 = BTreeStore::new(b"namespace2");
            
            store1.insert(b"key", 111u64);
            store2.insert(b"key", 222u64);
            
            // Same key, different namespaces - both should exist
            let val1 = store1.get(b"key");
            let val2 = store2.get(b"key");
            
            if val1.is_none() {
                return 1;
            }
            if val2.is_none() {
                return 2;
            }
            
            // Lengths are independent
            if store1.len() != 1 {
                return 3;
            }
            if store2.len() != 1 {
                return 4;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_cursor_delete() {
    let code = r#"
        entry test_btree_cursor_delete() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"a", 1u64);
            store.insert(b"b", 2u64);
            store.insert(b"c", 3u64);
            
            // Seek to "b"
            let result = store.seek(b"b", BTreeSeekBias::Exact, true);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let cursor = tuple.0;
            let item = tuple.1;
            
            if item.key != b"b" {
                return 2;
            }
            
            // Delete at cursor
            let deleted = cursor.delete();
            if !deleted {
                return 3;
            }
            
            // Length should be 2 now
            if store.len() != 2 {
                return 4;
            }
            
            // "b" should no longer exist
            let val = store.get(b"b");
            if val.is_some() {
                return 5;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_empty_seek() {
    let code = r#"
        entry test_btree_empty_seek() {
            let store = BTreeStore::new(b"test");
            
            // Seek on empty store should return None
            let result = store.seek(b"any", BTreeSeekBias::First, true);
            if result.is_some() {
                return 1;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_complex_values() {
    let code = r#"
        struct Person {
            name: string,
            age: u64
        }
        
        entry test_btree_complex_values() {
            let store = BTreeStore::new(b"people");
            
            let alice = Person { name: "Alice", age: 30 };
            let bob = Person { name: "Bob", age: 25 };
            
            store.insert(b"alice", alice);
            store.insert(b"bob", bob);
            
            // Verify both were inserted
            if store.len() != 2 {
                return 1;
            }
            
            // Get and verify
            let retrieved = store.get(b"alice");
            if retrieved.is_none() {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_bytes_keys() {
    let code = r#"
        entry test_btree_bytes_keys() {
            let store = BTreeStore::new(b"test");
            
            // Use various byte keys (no empty keys allowed)
            store.insert(b"\x00\x01\x02", 1u64);
            store.insert(b"hello world", 2u64);
            store.insert(b"x", 3u64);
            
            // Verify all were inserted
            if store.len() != 3 {
                return 1;
            }
            
            let val1 = store.get(b"\x00\x01\x02");
            if val1.is_none() {
                return 2;
            }
            
            let val2 = store.get(b"hello world");
            if val2.is_none() {
                return 3;
            }
            
            let val3 = store.get(b"x");
            if val3.is_none() {
                return 4;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_seek_less_or_equal() {
    let code = r#"
        entry test_btree_seek_le() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"a", 1u64);
            store.insert(b"c", 3u64);
            store.insert(b"e", 5u64);
            
            // Seek <= "d" should find "c"
            let result = store.seek(b"d", BTreeSeekBias::LessOrEqual, true);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let item = tuple.1;
            
            if item.key != b"c" {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_descending_iteration() {
    let code = r#"
        entry test_btree_descending() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"a", 1u64);
            store.insert(b"b", 2u64);
            store.insert(b"c", 3u64);
            
            // Start from last, iterate descending
            let result = store.seek(b"", BTreeSeekBias::Last, false);
            if result.is_none() {
                return 1;
            }
            
            let tuple = result.unwrap();
            let cursor = tuple.0;
            let item = tuple.1;
            
            // First item should be "c" (last in ascending order)
            if item.key != b"c" {
                return 2;
            }
            
            // Next should be "b"
            let next = cursor.next();
            if next.is_none() {
                return 3;
            }
            
            let next_item = next.unwrap();
            if next_item.key != b"b" {
                return 4;
            }
            
            // Next should be "a"
            let next2 = cursor.next();
            if next2.is_none() {
                return 5;
            }
            
            let next_item2 = next2.unwrap();
            if next_item2.key != b"a" {
                return 6;
            }
            
            // No more elements
            let next3 = cursor.next();
            if next3.is_some() {
                return 7;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}

#[tokio::test]
async fn test_btree_store_delete_nonexistent() {
    let code = r#"
        entry test_btree_delete_nonexistent() {
            let store = BTreeStore::new(b"test");
            
            store.insert(b"key", 123u64);
            
            // Delete non-existent key - verify length doesn't change
            store.delete(b"nonexistent");
            
            // Original key should still exist
            if store.len() != 1 {
                return 1;
            }
            
            // The original key should still be retrievable
            let val = store.get(b"key");
            if val.is_none() {
                return 2;
            }
            
            return 0;
        }
    "#;

    run_silex_code_expect_success(code).await;
}
