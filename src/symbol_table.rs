use crate::symbol::Symbol;
use std::ops::Index;
use std::collections::HashMap;

pub struct SymbolTable<T> {
  count: Vec<usize>,
  undo: Vec<(Symbol, Option<T>)>,
  table: HashMap<Symbol, T>,
}

impl<T> SymbolTable<T> {
  pub fn new() -> Self {
    Self {
      count: Vec::new(),
      undo: Vec::new(),
      table: HashMap::new(),
    }
  }

  pub fn put_scope(&mut self) {
    self.count.push(0);
  }

  pub fn pop_scope(&mut self) {
    for _ in 0 .. self.count.pop().unwrap() {
      match self.undo.pop().unwrap() {
        (key, None) => {
          let _ = self.table.remove(&key);
        }
        (key, Some(value)) => {
          let _ = self.table.insert(key, value);
        }
      }
    }
  }

  pub fn insert(&mut self, key: Symbol, value: T) {
    match self.count.last_mut() {
      None => {
        let _ = self.table.insert(key, value);
      }
      Some(n) => {
        *n += 1;
        self.undo.push((key, self.table.insert(key, value)));
      }
    }
  }

  pub fn get(&self, key: Symbol) -> Option<&T> {
    return self.table.get(&key);
  }

  pub fn get_mut(&mut self, key: Symbol) -> Option<&mut T> {
    return self.table.get_mut(&key);
  }
}

impl<T> Index<Symbol> for SymbolTable<T> {
  type Output = T;

  fn index(&self, index: Symbol) -> &Self::Output {
    return &self.table[&index];
  }
}
