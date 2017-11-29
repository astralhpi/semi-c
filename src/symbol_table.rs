use std::collections::{HashMap, LinkedList};
use std::hash::Hash;
use std::cmp::Eq;


pub struct SymbolTable<K: Hash + Eq, V>{
    pub list: LinkedList<Option<HashMap<K, V>>>,
}


impl<K: Hash + Eq, V> SymbolTable<K, V> {
    pub fn new() -> SymbolTable<K, V> {
        SymbolTable { list: LinkedList::new() }
    }

    pub fn push_scope(&mut self) {
        self.list.push_front(Some(HashMap::new()))
    }

    pub fn drop_scope(&mut self) {
        match self.list.pop_front() {
            None => panic!("SymbolTable: no scope to drop"),
            _ => {}
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let err = "SymbolTable: no scope to add";
        let node = self.list.front_mut().expect(err);
        match node {
            &mut Some(ref mut t) => t.insert(key, value),
            &mut None => panic!(err),
        };
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.get_with_depth(key) {
            None => Option::None,
            Some((value, _)) => Option::Some(value)
        }

    }

    pub fn get_with_depth(&self, key: &K) -> Option<(&V, usize)> {
        let mut depth = self.list.len();
        for op in &self.list {
            match op {
                &Some(ref table) => match table.get(key) {
                    Some(v) => return Some((v, depth)),
                    None => depth -= 1,
                },
                &None => return None,
            };
        }
        Option::None
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.get_mut_with_depth(key) {
            None => Option::None,
            Some((mut value, _)) => Option::Some(value)
        }

    }


    pub fn get_mut_with_depth(&mut self, key: &K) -> Option<(&mut V, usize)> {
        let mut depth = self.list.len();
        for mut op in &mut self.list {
            match op {
                &mut Some(ref mut table) => match table.get_mut(key) {
                    Some(v) => return Some((v, depth)),
                    None => depth -= 1,
                },
                &mut None => return None,
            };
        }
        Option::None
    }
    
    pub fn depth(&self) -> usize {
        self.list.len()
    }
}


#[test]
fn sym_table() {
    let mut table: SymbolTable<String, String> = SymbolTable::new();
    assert_eq!(table.depth(), 0);
    table.push_scope();
    assert_eq!(table.depth(), 1);
    table.drop_scope();
    assert_eq!(table.depth(), 0);

    table.push_scope();
    table.insert("a".to_string(), "1".to_string());
    table.push_scope();
    table.insert("a".to_string(), "2".to_string());
    assert_eq!(table.get(&"a".to_string()).unwrap(), "2");
    table.drop_scope();
    assert_eq!(table.get(&"a".to_string()).unwrap(), "1");

}
