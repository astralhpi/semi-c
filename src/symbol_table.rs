use std::collections::{HashMap};
use std::hash::Hash;
use std::cmp::Eq;

struct Node<K: Hash + Eq, V>{
    parent: Option<Box<Node<K, V>>>,
    table: HashMap<K, V>,
}

impl <K: Hash + Eq, V> Node<K, V> {
    fn next<'a>(&'a self) -> Option<&'a Node<K,V>> {
        match self.parent {
            Option::Some(ref b) => Option::Some(b.as_ref()),
            Option::None => Option::None
        }
    }

}
pub struct SymbolTable<K: Hash + Eq, V>{
    node: Node<K, V>,
    depth: u32,
}


impl<K: Hash + Eq, V> SymbolTable<K, V> {
    pub fn new() -> SymbolTable<K, V> {
        let node: Node<K, V> = Node {
            parent: Option::None,
            table: HashMap::new()
        };
        SymbolTable { node, depth: 0 }
    }

    #[must_use]
    pub fn push_scope(mut self) -> SymbolTable<K, V>{
        self.node = Node {
            parent: Option::Some(Box::new(self.node)),
            table: HashMap::new()
        };
        self.depth += 1;
        self
    }
    #[must_use]
    pub fn drop_scope(mut self) -> SymbolTable<K, V>{
        match self.node.parent {
            Some(node) => {
                self.node = *node;
                self.depth -= 1;
                self
            },
            _ => panic!("SymbolTable: no scope to drop")
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.node.table.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.get_with_depth(key) {
            Option::None => Option::None,
            Option::Some((value, _)) => Option::Some(value)
        }

    }

    pub fn get_with_depth(&self, key: &K) -> Option<(&V, u32)> {
        let mut depth = self.depth;
        let mut node = &self.node;
        loop {
            let value = node.table.get(&key);
            match value {
                Option::Some(ref v) => {
                    return Option::Some((v, depth));
                },
                Option::None => {
                    depth -= 1;
                    match node.next() {
                        Option::Some(n) => node = n,
                        Option::None => return Option::None
                    }
                }
            }
        }


    }
}


#[test]
fn sym_table() {
    let mut table: SymbolTable<String, String> = SymbolTable::new();
    assert_eq!(table.depth, 0);
    table = table.push_scope();
    assert_eq!(table.depth, 1);
    table = table.drop_scope();
    assert_eq!(table.depth, 0);

    table.insert("a".to_string(), "1".to_string());
    table = table.push_scope();
    table.insert("a".to_string(), "2".to_string());
    assert_eq!(table.get(&"a".to_string()).unwrap(), "2");
    table = table.drop_scope();
    assert_eq!(table.get(&"a".to_string()).unwrap(), "1");

}
