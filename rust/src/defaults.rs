//! Bundled default C++ and Markdown competitive programming cheatsheet snippets.

use crate::export::import::ImportRecord;

/// Return the bundled default cheatsheets for C++ and competitive programming.
pub fn default_snippets() -> Vec<ImportRecord> {
    vec![
        ImportRecord {
            id: None,
            title: "Vector - STL Cheatsheet".to_string(),
            description: Some("C++ vector methods, time complexities, and usage examples".to_string()),
            use_case: Some("Fast lookup for std::vector methods and amortized complexities".to_string()),
            tags: Some("stl, vector, cheatsheet, cpp".to_string()),
            code: r#"#include <vector>

std::vector<int> a;                  // Declaration - O(1)
std::vector<int> a(n, 0);            // Initialisation with size n - O(n)
a.push_back(x);                      // Amortized O(1)
a.pop_back();                        // O(1)
a.size(); a.empty(); a.clear();      // O(1) size/empty, O(n) clear
a.front(); a.back();                 // O(1) element access
a.insert(a.begin() + i, x);          // O(n) insert
a.erase(a.begin() + i);              // O(n) erase
a.resize(n);                         // O(n)
a.reserve(n);                        // Preallocate buffer - O(n)"#.to_string(),
            language: Some("cpp".to_string()),
            created_at: None,
            updated_at: None,
        },
        ImportRecord {
            id: None,
            title: "Algorithms & Numeric - STL Cheatsheet".to_string(),
            description: Some("Standard C++ algorithms: binary search, sorting, heaps, numeric".to_string()),
            use_case: Some("Quick reference for std::lower_bound, sort, nth_element, accumulate".to_string()),
            tags: Some("stl, algorithm, search, sort, math".to_string()),
            code: r#"#include <algorithm>
#include <numeric>

std::sort(a.begin(), a.end());                              // O(n log n) intro-sort
std::stable_sort(a.begin(), a.end());                       // O(n log n) merge-sort
std::binary_search(a.begin(), a.end(), x);                  // O(log n) returns bool
auto it = std::lower_bound(a.begin(), a.end(), x);          // O(log n) first >= x
auto it = std::upper_bound(a.begin(), a.end(), x);          // O(log n) first > x
std::nth_element(a.begin(), a.begin() + k, a.end());        // Average O(n) k-th element
long long sum = std::accumulate(a.begin(), a.end(), 0LL);   // O(n) sum
long long g = std::gcd(a, b);                               // O(log min(a,b)) GCD
std::reverse(a.begin(), a.end());                           // O(n) reverse
std::next_permutation(a.begin(), a.end());                  // O(n) lexicographical perm"#.to_string(),
            language: Some("cpp".to_string()),
            created_at: None,
            updated_at: None,
        },
        ImportRecord {
            id: None,
            title: "Set & Map - STL Cheatsheet".to_string(),
            description: Some("Balanced BST (std::set, std::map) and Hash Table (unordered) methods".to_string()),
            use_case: Some("Lookup operations and iterator traversal for ordered/unordered associative containers".to_string()),
            tags: Some("stl, map, set, hash, tree".to_string()),
            code: r#"#include <set>
#include <map>
#include <unordered_map>

std::set<int> s;                     // Red-Black tree - O(log n) ops
s.insert(x);                         // O(log n)
s.count(x);                          // O(log n)
s.erase(x);                          // O(log n)
auto it = s.lower_bound(x);          // O(log n)

std::map<string, int> mp;            // Key-Value map - O(log n)
mp[key] = val;                       // O(log n)

std::unordered_map<int, int> ump;    // Hash table - O(1) average
ump.reserve(1024);                   // Prevent re-hashing
ump.max_load_factor(0.25);           // Anti-hash collision tuning"#.to_string(),
            language: Some("cpp".to_string()),
            created_at: None,
            updated_at: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_snippets_content() {
        let snippets = default_snippets();
        assert_eq!(snippets.len(), 3);
        assert!(snippets[0].title.contains("Vector"));
        assert!(snippets[1].title.contains("Algorithms"));
        assert!(snippets[2].title.contains("Set & Map"));
        for s in &snippets {
            assert!(!s.code.is_empty());
            assert_eq!(s.language.as_deref(), Some("cpp"));
        }
    }
}
