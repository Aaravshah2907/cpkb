"""Bundled default C++ competitive-programming cheatsheet snippets."""

from __future__ import annotations

from .config import max_snippets


USE_CASE = "When you want a quick summary of available default options and their syntax."
TAGS = "cheat sheet, helpful"


CHEATSHEETS = [
    {
        "key": "vector",
        "container": "Vector",
        "header": "<vector>",
        "methods": [
            ("Declaration", "vector<T>", "vector<int> a;", "O(1)"),
            ("Initialization", "constructed vector", "vector<int> a(n, 0);", "O(n)"),
            ("push_back()", "void", "a.push_back(x);", "Amortized O(1)"),
            ("pop_back()", "void", "a.pop_back();", "O(1)"),
            ("size()", "size_t", "a.size();", "O(1)"),
            ("empty()", "bool", "a.empty();", "O(1)"),
            ("clear()", "void", "a.clear();", "O(n)"),
            ("front()/back()", "reference", "a.front(); a.back();", "O(1)"),
            ("insert()", "iterator", "a.insert(pos, x);", "O(n)"),
            ("erase()", "iterator", "a.erase(pos);", "O(n)"),
            ("resize()", "void", "a.resize(n);", "O(n)"),
            ("assign()", "void", "a.assign(n, value);", "O(n)"),
            ("swap()", "void", "a.swap(b);", "O(1)"),
            ("capacity()", "size_t", "a.capacity();", "O(1)"),
            ("reserve()", "void", "a.reserve(n);", "O(n) when reallocating"),
            ("shrink_to_fit()", "void", "a.shrink_to_fit();", "O(n)"),
            ("emplace_back()", "reference/void", "a.emplace_back(args);", "Amortized O(1)"),
            ("cbegin()/cend()", "const_iterator", "a.cbegin(); a.cend();", "O(1)"),
        ],
    },
    {
        "key": "pair",
        "container": "Pair",
        "header": "<utility>",
        "methods": [
            ("pair<T,U>", "pair object", "pair<int,int> p;", "O(1)"),
            ("make_pair()", "pair object", "auto p = make_pair(a, b);", "O(1)"),
            ("brace init", "pair object", "pair<int,int> p = {a, b};", "O(1)"),
            ("first/second", "member reference", "p.first; p.second;", "O(1)"),
            ("nested pairs", "nested object", "pair<int, pair<int,int>> p;", "O(1)"),
            ("comparison", "bool", "p1 < p2;", "O(1) for primitive fields"),
            ("structured binding", "copied/bound values", "auto [x, y] = p;", "O(1)"),
        ],
    },
    {
        "key": "iterator",
        "container": "Iterator",
        "header": "<iterator>",
        "methods": [
            ("begin()/end()", "iterator", "begin(a); end(a);", "O(1)"),
            ("rbegin()/rend()", "reverse_iterator", "a.rbegin(); a.rend();", "O(1)"),
            ("range loop", "element references", "for (auto &x : a) {}", "O(n)"),
            ("distance()", "difference", "distance(it1, it2);", "O(1) random access, else O(n)"),
            ("advance()", "void", "advance(it, k);", "O(1) random access, else O(k)"),
            ("next()/prev()", "iterator", "next(it); prev(it);", "O(1) or O(k)"),
            ("iterator arithmetic", "iterator", "it + k; it - k;", "O(1) random access"),
        ],
    },
    {
        "key": "algorithm",
        "container": "Algorithm's Library",
        "header": "<algorithm>, <numeric>",
        "methods": [
            ("find()", "iterator", "find(a.begin(), a.end(), x);", "O(n)"),
            ("count()", "integer", "count(a.begin(), a.end(), x);", "O(n)"),
            ("find_if()/count_if()", "iterator/integer", "find_if(a.begin(), a.end(), pred);", "O(n)"),
            ("binary_search()", "bool", "binary_search(a.begin(), a.end(), x);", "O(log n)"),
            ("lower_bound()", "iterator", "lower_bound(a.begin(), a.end(), x);", "O(log n)"),
            ("upper_bound()", "iterator", "upper_bound(a.begin(), a.end(), x);", "O(log n)"),
            ("equal_range()", "iterator pair", "equal_range(a.begin(), a.end(), x);", "O(log n)"),
            ("sort()", "void", "sort(a.begin(), a.end());", "O(n log n)"),
            ("stable_sort()", "void", "stable_sort(a.begin(), a.end());", "O(n log n)"),
            ("partial_sort()", "void", "partial_sort(a.begin(), a.begin()+k, a.end());", "O(n log k)"),
            ("nth_element()", "void", "nth_element(a.begin(), a.begin()+k, a.end());", "Average O(n)"),
            ("reverse()/rotate()", "void", "reverse(a.begin(), a.end());", "O(n)"),
            ("next_permutation()", "bool", "next_permutation(a.begin(), a.end());", "O(n)"),
            ("accumulate()", "value", "accumulate(a.begin(), a.end(), 0LL);", "O(n)"),
            ("iota()", "void", "iota(a.begin(), a.end(), 0);", "O(n)"),
            ("partial_sum()", "output iterator", "partial_sum(a.begin(), a.end(), out);", "O(n)"),
            ("gcd()/lcm()", "value", "gcd(a, b); lcm(a, b);", "O(log min(a,b))"),
            ("min_element()/max_element()", "iterator", "min_element(a.begin(), a.end());", "O(n)"),
            ("heap operations", "void", "make_heap(a.begin(), a.end());", "O(n) make, O(log n) push/pop"),
        ],
    },
    {
        "key": "string",
        "container": "String",
        "header": "<string>, <sstream>, <cctype>",
        "methods": [
            ("substr()", "string", "s.substr(pos, len);", "O(len)"),
            ("find()", "size_t", "s.find(pattern);", "O(n*m) worst-case"),
            ("erase()/insert()", "string&", "s.erase(pos, len); s.insert(pos, t);", "O(n)"),
            ("append()/replace()", "string&", "s.append(t); s.replace(pos, len, t);", "O(n + |t|)"),
            ("compare()", "int", "s.compare(t);", "O(min(n,m))"),
            ("starts_with()/ends_with()", "bool", "s.starts_with(t); s.ends_with(t);", "O(|t|), C++20"),
            ("find_first_of()", "size_t", "s.find_first_of(chars);", "O(n*m) worst-case"),
            ("find_last_of()", "size_t", "s.find_last_of(chars);", "O(n*m) worst-case"),
            ("stoi()/stoll()", "number", "stoll(s);", "O(n)"),
            ("stof()/stod()", "floating number", "stod(s);", "O(n)"),
            ("to_string()", "string", "to_string(x);", "O(digits)"),
            ("stringstream", "stream", "stringstream ss(s);", "O(n)"),
            ("getline()", "istream&", "getline(cin, s);", "O(n)"),
            ("isdigit()/isalpha()", "int", "isdigit(c); isalpha(c);", "O(1)"),
            ("tolower()/toupper()", "int", "tolower(c); toupper(c);", "O(1)"),
        ],
    },
    {
        "key": "stack",
        "container": "Stack",
        "header": "<stack>",
        "methods": [
            ("push()", "void", "st.push(x);", "O(1)"),
            ("pop()", "void", "st.pop();", "O(1)"),
            ("top()", "reference", "st.top();", "O(1)"),
            ("empty()", "bool", "st.empty();", "O(1)"),
            ("size()", "size_t", "st.size();", "O(1)"),
            ("emplace()", "reference/void", "st.emplace(args);", "O(1)"),
            ("underlying container", "adapter detail", "stack<int, vector<int>> st;", "Container-dependent"),
        ],
    },
    {
        "key": "queue",
        "container": "Queue",
        "header": "<queue>",
        "methods": [
            ("push()", "void", "q.push(x);", "O(1)"),
            ("pop()", "void", "q.pop();", "O(1)"),
            ("front()/back()", "reference", "q.front(); q.back();", "O(1)"),
            ("empty()", "bool", "q.empty();", "O(1)"),
            ("size()", "size_t", "q.size();", "O(1)"),
            ("emplace()", "reference/void", "q.emplace(args);", "O(1)"),
            ("underlying container", "adapter detail", "queue<int, deque<int>> q;", "Container-dependent"),
        ],
    },
    {
        "key": "deque",
        "container": "Deque",
        "header": "<deque>",
        "methods": [
            ("push_front()/push_back()", "void", "dq.push_front(x); dq.push_back(x);", "O(1)"),
            ("pop_front()/pop_back()", "void", "dq.pop_front(); dq.pop_back();", "O(1)"),
            ("front()/back()", "reference", "dq.front(); dq.back();", "O(1)"),
            ("operator[]", "reference", "dq[i];", "O(1)"),
            ("insert()/erase()", "iterator", "dq.insert(pos, x); dq.erase(pos);", "O(n)"),
            ("clear()", "void", "dq.clear();", "O(n)"),
            ("size()/empty()", "size_t/bool", "dq.size(); dq.empty();", "O(1)"),
            ("begin()/end()", "iterator", "dq.begin(); dq.end();", "O(1)"),
        ],
    },
    {
        "key": "set",
        "container": "Set",
        "header": "<set>",
        "methods": [
            ("insert()", "iterator,bool", "s.insert(x);", "O(log n)"),
            ("erase(value)", "count", "s.erase(x);", "O(log n)"),
            ("find()", "iterator", "s.find(x);", "O(log n)"),
            ("count()", "0 or 1", "s.count(x);", "O(log n)"),
            ("contains()", "bool", "s.contains(x);", "O(log n), C++20"),
            ("lower_bound()/upper_bound()", "iterator", "s.lower_bound(x);", "O(log n)"),
            ("equal_range()", "iterator pair", "s.equal_range(x);", "O(log n)"),
            ("size()/empty()", "size_t/bool", "s.size(); s.empty();", "O(1)"),
            ("begin()/rbegin()", "iterator", "*s.begin(); *s.rbegin();", "O(1)"),
        ],
    },
    {
        "key": "multiset",
        "container": "Multiset",
        "header": "<set>",
        "methods": [
            ("insert()", "iterator", "ms.insert(x);", "O(log n)"),
            ("erase(value)", "count", "ms.erase(x);", "O(k + log n)"),
            ("erase(iterator)", "iterator", "ms.erase(ms.find(x));", "Amortized O(1) after find"),
            ("count()", "count", "ms.count(x);", "O(k + log n)"),
            ("contains()", "bool", "ms.contains(x);", "O(log n), C++20"),
            ("equal_range()", "iterator pair", "ms.equal_range(x);", "O(log n)"),
            ("lower_bound()/upper_bound()", "iterator", "ms.lower_bound(x);", "O(log n)"),
            ("size()/empty()", "size_t/bool", "ms.size(); ms.empty();", "O(1)"),
        ],
    },
    {
        "key": "ordered_map",
        "container": "Ordered Map",
        "header": "<map>",
        "methods": [
            ("operator[]", "mapped reference", "mp[key]++;", "O(log n)"),
            ("at()", "mapped reference", "mp.at(key);", "O(log n)"),
            ("insert()", "iterator,bool", "mp.insert({key, value});", "O(log n)"),
            ("erase()", "count", "mp.erase(key);", "O(log n)"),
            ("find()", "iterator", "mp.find(key);", "O(log n)"),
            ("count()/contains()", "integer/bool", "mp.count(key); mp.contains(key);", "O(log n)"),
            ("lower_bound()/upper_bound()", "iterator", "mp.lower_bound(key);", "O(log n)"),
            ("begin()/rbegin()", "iterator", "mp.begin(); mp.rbegin();", "O(1)"),
        ],
    },
    {
        "key": "unordered_map",
        "container": "Unordered Map",
        "header": "<unordered_map>",
        "methods": [
            ("operator[]", "mapped reference", "ump[key]++;", "Average O(1)"),
            ("at()", "mapped reference", "ump.at(key);", "Average O(1)"),
            ("insert()", "iterator,bool", "ump.insert({key, value});", "Average O(1)"),
            ("erase()", "count", "ump.erase(key);", "Average O(1)"),
            ("find()", "iterator", "ump.find(key);", "Average O(1)"),
            ("count()/contains()", "integer/bool", "ump.count(key); ump.contains(key);", "Average O(1)"),
            ("load_factor()", "float", "ump.load_factor();", "O(1)"),
            ("max_load_factor()", "float/void", "ump.max_load_factor(0.7);", "O(1)"),
            ("reserve()/rehash()", "void", "ump.reserve(n);", "O(n)"),
        ],
    },
    {
        "key": "priority_queue",
        "container": "Priority Queue",
        "header": "<queue>",
        "methods": [
            ("push()", "void", "pq.push(x);", "O(log n)"),
            ("pop()", "void", "pq.pop();", "O(log n)"),
            ("top()", "const reference", "pq.top();", "O(1)"),
            ("empty()/size()", "bool/size_t", "pq.empty(); pq.size();", "O(1)"),
            ("emplace()", "void", "pq.emplace(args);", "O(log n)"),
            ("min heap", "priority_queue type", "priority_queue<int, vector<int>, greater<int>> pq;", "O(1) construct"),
            ("custom comparator", "priority_queue type", "priority_queue<T, vector<T>, Cmp> pq;", "O(1) construct"),
        ],
    },
    {
        "key": "bitset",
        "container": "Bitset",
        "header": "<bitset>",
        "methods": [
            ("declaration", "bitset", "bitset<32> b(x);", "O(N/word)"),
            ("operator[]", "bit reference/bool", "b[i];", "O(1)"),
            ("set()/reset()/flip()", "bitset&", "b.set(i); b.reset(i); b.flip(i);", "O(1) one bit"),
            ("count()", "size_t", "b.count();", "O(N/word)"),
            ("any()/none()/all()", "bool", "b.any(); b.none(); b.all();", "O(N/word)"),
            ("bit operations", "bitset", "a & b; a | b; a ^ b; ~a;", "O(N/word)"),
            ("shifts", "bitset", "b << k; b >> k;", "O(N/word)"),
            ("to_ulong()/to_string()", "number/string", "b.to_string();", "O(N)"),
        ],
    },
    {
        "key": "tuple",
        "container": "Tuple",
        "header": "<tuple>",
        "methods": [
            ("tuple<T...>", "tuple object", "tuple<int,int,string> t;", "O(1)"),
            ("make_tuple()", "tuple object", "auto t = make_tuple(a, b, c);", "O(1)"),
            ("get<>()", "element reference", "get<0>(t);", "O(1)"),
            ("structured binding", "copied/bound values", "auto [a, b, c] = t;", "O(1)"),
            ("tie()", "tuple of refs", "tie(a, b, c) = t;", "O(1)"),
            ("tuple_cat()", "tuple object", "tuple_cat(t1, t2);", "O(total fields)"),
            ("comparison", "bool", "t1 < t2;", "O(fields)"),
        ],
    },
    {
        "key": "stl_utilities",
        "container": "STL Utilities",
        "header": "<utility>, <functional>",
        "methods": [
            ("swap()", "void", "swap(a, b);", "O(1) or type-dependent"),
            ("move()", "rvalue reference", "auto y = move(x);", "O(1) itself"),
            ("std::function", "call wrapper", "function<int(int)> f;", "Call overhead O(1)"),
            ("lambda", "callable object", "auto cmp = [](int a, int b){ return a > b; };", "O(1) create"),
            ("custom comparator", "callable", "sort(a.begin(), a.end(), cmp);", "O(n log n) sort"),
            ("greater<>/less<>", "comparator", "sort(a.begin(), a.end(), greater<int>());", "O(n log n) sort"),
            ("min()/max()", "value", "min(a, b); max({a, b, c});", "O(1) or O(k)"),
        ],
    },
]


def _table(methods: list[tuple[str, str, str, str]]) -> str:
    rows = [
        "| Method_name | What it returns | Syntax | Time Complexity |",
        "|-------------|-----------------|--------|-----------------|",
    ]
    for name, returns, syntax, complexity in methods:
        rows.append(f"| {name} | {returns} | `{syntax}` | {complexity} |")
    return "\n".join(rows)


def default_snippets(app_dir) -> list[dict[str, str]]:
    """Return default cheatsheet snippets with configured-width special IDs."""
    width = len(str(max_snippets(app_dir)))
    snippets = []
    for index, sheet in enumerate(CHEATSHEETS, start=1):
        container = sheet["container"]
        code = f"""/*
Cheat Sheet for {container} present in headerfile:{sheet["header"]}

{_table(sheet["methods"])}

*/"""
        snippets.append(
            {
                "id": f"cp_{index:0{width}d}",
                "title": f"{container} Method Info",
                "description": f"This contains a commented cpp table cheatsheet for the {container}",
                "use_case": USE_CASE,
                "tags": TAGS,
                "code": code,
            }
        )
    return snippets
