use std::{borrow::Borrow, collections::HashMap, hash::Hash, marker::PhantomData, ptr::NonNull};

struct Node<K, V> {
    k: K,
    v: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

struct MyKey<K, V>(NonNull<Node<K, V>>);

impl<K: Hash + Eq, V> Borrow<K> for MyKey<K, V> {
    fn borrow(&self) -> &K {
        unsafe { &self.0.as_ref().k }
    }
}

impl<K: Hash, V> Hash for MyKey<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { self.0.as_ref().k.hash(state) }
    }
}

impl<K: Eq, V> PartialEq for MyKey<K, V> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref().k.eq(&other.0.as_ref().k) }
    }
}

impl<K: Eq, V> Eq for MyKey<K, V> {}

impl<K, V> Node<K, V> {
    fn new(k: K, v: V) -> Self {
        Self {
            k,
            v,
            prev: None,
            next: None,
        }
    }
}

pub struct LruCache<K, V> {
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    map: HashMap<MyKey<K, V>, NonNull<Node<K, V>>>,
    capacity: usize,
    count_head: Option<NonNull<Node<K, usize>>>,
    count_tail: Option<NonNull<Node<K, usize>>>,
    count_map: HashMap<MyKey<K, usize>, NonNull<Node<K, usize>>>,
    capacity2: usize,
    k: usize,
    marker: PhantomData<Node<K, V>>,
}

impl<K: Hash + Eq + PartialEq, V> LruCache<K, V> {
    pub fn new(capacity: usize, capacity2: usize, k: usize) -> Self {
        Self {
            head: None,
            tail: None,
            map: HashMap::new(),
            capacity,
            count_head: None,
            count_tail: None,
            count_map: HashMap::new(),
            capacity2,
            k,
            marker: PhantomData,
        }
    }

    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        let list_node = Box::leak(Box::new(Node::new(k, v))).into();
        let old_list_node = self.map.remove(&MyKey(list_node)).map(|list_node| {
            self.erase(list_node);
            list_node
        });
        if old_list_node.is_none() && self.map.len() >= self.capacity {
            let tail = self.tail.unwrap();
            self.erase(tail);
            self.map.remove(&MyKey(tail));
        }
        self.insert(list_node);
        self.map.insert(MyKey(list_node), list_node);
        old_list_node.map(|list_node| unsafe {
            let list_node = Box::from_raw(list_node.as_ptr());
            list_node.v
        })
    }

    fn put2(&mut self, k: K, v: usize) -> Option<usize> {
        let list_node = Box::leak(Box::new(Node::new(k, v))).into();
        let old_list_node = self.count_map.remove(&MyKey(list_node)).map(|list_node| {
            self.erase2(list_node);
            list_node
        });
        if old_list_node.is_none() && self.count_map.len() >= self.capacity2 {
            let tail = self.tail.unwrap();
            self.erase2(tail);
            self.count_map.remove(&MyKey(tail));
        }
        self.insert2(list_node);
        self.count_map.insert(MyKey(list_node), list_node);
        old_list_node.map(|list_node| unsafe {
            let list_node = Box::from_raw(list_node.as_ptr());
            list_node.v
        })
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        let count = 1;
        if let Some(list_node) = self.count_map.get(k) {
            let list_node = *list_node;
            self.erase2(list_node);
            self.insert2(list_node);
            unsafe {
                count = Some(&list_node.as_ref().v) + 1;
                self.put2(k, count);
            }
        } else {
            self.put2(k, count);
        }

        if let Some(list_node) = self.map.get(k) {
            let list_node = *list_node;
            if (count >= self.k) {
                self.erase(list_node);
                self.insert(list_node);
            }
            unsafe { Some(&list_node.as_ref().v) }
        } else {
            None
        }
    }

    fn erase(&mut self, mut list_node: NonNull<Node<K, V>>) {
        unsafe {
            match list_node.as_mut().prev {
                Some(mut prev) => {
                    prev.as_mut().next = list_node.as_ref().next;
                }
                None => {
                    self.head = list_node.as_ref().next;
                }
            }
            match list_node.as_mut().next {
                Some(mut next) => {
                    next.as_mut().prev = list_node.as_ref().prev;
                }
                None => {
                    self.tail = list_node.as_ref().prev;
                }
            }

            list_node.as_mut().prev = None;
            list_node.as_mut().next = None;
        }
    }

    fn insert(&mut self, mut list_node: NonNull<Node<K, V>>) {
        match self.head {
            Some(mut head) => {
                unsafe {
                    head.as_mut().prev = Some(list_node);
                    list_node.as_mut().next = Some(head);
                    list_node.as_mut().prev = None;
                }
                self.head = Some(list_node);
            }
            None => {
                unsafe {
                    list_node.as_mut().prev = None;
                    list_node.as_mut().next = None;
                }
                self.head = Some(list_node);
                self.tail = Some(list_node);
            }
        }
    }
}

impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        while let Some(list_node) = self.head.take() {
            unsafe {
                self.head = list_node.as_ref().next;
                drop(list_node.as_ptr());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}