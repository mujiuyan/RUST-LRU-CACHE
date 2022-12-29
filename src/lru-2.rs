use std::convert::TryInto;
use std::collections::VecDeque;
use std::collections::HashMap;


struct LRUCache {
    dq: VecDeque<i32>,
    map: HashMap<i32, i32>,
    cap: usize,
}

struct TwoQCache {
    lru1: LRUCache,
    lru2: LRUCache,
    cap: usize,
}

impl TwoQCache {
    fn new(capacity: i32) -> Self {
        Self {
            lru1: LRUCache::new(capacity),
            lru2: LRUCache::new(capacity),
            cap: capacity.try_into().unwrap(),
        }
    }

    fn get(&mut self, key: i32) -> i32 {
        if self.lru1.map.contains_key(&key) {
            let v = self.lru1.map.get(&key).cloned().unwrap();
            self.lru1.dq.retain(|&x| x != key);
            self.lru1.map.remove(&key);
            self.lru2.put(key, v);
            return v;
        } else if self.lru2.map.contains_key(&key) {
            return self.lru2.get(key);
        }
        -1
    }

    fn put(&mut self, key: i32, value: i32) {
        if self.lru1.map.contains_key(&key) {
            self.lru1.dq.retain(|&x| x != key);
            self.lru1.map.remove(&key);
            self.lru2.put(key, value);
        } else if self.lru2.map.contains_key(&key) {
            self.lru2.put(key, value);
        } else {
            self.lru1.put(key, value);
        }
    }
}

impl LRUCache {

    fn new(capacity: i32) -> Self {
        Self {
            dq: VecDeque::new(),
            map: HashMap::new(),
            cap: capacity.try_into().unwrap(),
        }
    }
    
    fn get(&mut self, key: i32) -> i32 {
        if self.map.contains_key(&key) {
          self.dq.retain(|&x| x != key);
          self.dq.push_back(key);
          return self.map.get(&key).cloned().unwrap();
        }
        -1
    }
    
    fn put(&mut self, key: i32, value: i32) {
        if self.map.contains_key(&key) {
            self.dq.retain(|&x| x != key);
            self.dq.push_back(key);
            self.map.insert(key, value);
        } else {
            if self.map.len() == self.cap {
                let rm = self.dq.pop_front().unwrap();
                self.map.remove(&rm);
            }
            self.map.insert(key, value);
            self.dq.push_back(key);
        }
    }
}
 
#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test() {
        let mut cache = TwoQCache::new(2);
        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.get(1), 1);
        cache.put(1, 100);
        cache.put(3, 3);
        cache.put(3, 200);
        assert_eq!(cache.get(1), 100);
        cache.put(2, 500);
        assert_eq!(cache.get(3), -1);
        cache.put(4, 4);
        cache.put(5, 5);
        cache.put(6, 6);
        assert_eq!(cache.get(4), -1);
        cache.put(5, 700);
        assert_eq!(cache.get(1), -1);



        // let mut cache = LRUCache::new(2);
        // cache.put(1, 1);
        // cache.put(2, 2);
        // assert_eq!(cache.get(1), 1);
        // cache.put(3, 3);
        // assert_eq!(cache.get(2), -1);
        // cache.put(4, 4);
        // assert_eq!(cache.get(1), -1);
        // assert_eq!(cache.get(3), 3);
        // assert_eq!(cache.get(4), 4);
    }
}
