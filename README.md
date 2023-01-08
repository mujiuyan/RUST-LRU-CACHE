# RUST-LRU-CACHE
## 0 项目介绍
本项目实现了LRU Cache、LRU-K Cache和two queue算法。代码位于src目录下。其中，lib.rs中实现了LRU Cache，lru-k.rs中实现了LRU-K Cache，lru-2.rs中实现了two queue。下面详细介绍算法原理和实现。
## 1.1 LRU算法简介
LRU（Least recently used）算法的核心思想为：最近被访问的数据，未来也很有可能被访问，因此保留最近访问的数据，根据数据的历史访问记录来进行淘汰最近没有访问的数据。
## 1.2 LRU实现方法
- 使用哈希表来记录每个数据在双向链表中的位置，实现O(1)的查找速度。
- 链表按照使用顺序存储数据，链表头部的数据是最近访问过的，尾部的数据是最近最久未使用的。
- 为使代码更加简介，使用Unsafe Rust中的裸指针来实现双向链表。
## 2.1 LRU-K算法简介
LRU-K对LRU进行了改进，将“最近使用过1次”的判断标准扩展为“最近使用过K次”。其主要目的是为了解决LRU算法“缓存污染”的问题，也就是说没有到达K次访问的数据并不会被缓存，这也意味着需要对于缓存数据的访问次数进行计数，并且访问记录不能无限记录，也需要使用替换算法进行替换。当需要淘汰数据时，LRU-K会淘汰第K次访问时间距当前时间最大的数据。
## 2.2 LRU-K实现方法
- 记录每个数据的访问次数，当访问次数达到k时，表明该数据需要缓存。若历史队列已满，需要根据一定的策略淘汰数据。
- 保存已经访问了k次的数据。若缓存队列已满，需要淘汰掉第k次访问距现在最久的数据。
## 2.3 LRU-K算法流程
以网页缓存为例，算法流程如下：
- 用户发起URL请求。
- 若该URL已在cache中，则直接从cache中获取网页信息，否则从服务器发起请求。
- 将该URL在历史队列中记录的访问次数加一，并将该记录放在历史队列队首。
- 若历史队列已满，则删除历史队列队尾的元素。
- 若该URL在历史队列中的访问次数达到k，则将该网页信息放在缓存队列队首。
- 若缓存队列已满，则删除缓存队列队尾的元素。
## 2.4 LRU-K关键代码
下述代码定义了双向链表中的节点，每个节点包含一个K、V键值对和两个指针，分别指向前一个节点和后一个节点。
```
struct Node<K, V> {
    k: K,
    v: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}
```
下述代码定义了实现LRU-K的结构体，该结构体中分别实现了历史队列、缓存队列。head、tail、map、capacity等实现了历史队列，count_head、count_tail、count_map、capacity2等实现了缓存队列。
```
pub struct LruCacheK<K, V> {
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
```
下述代码实现了在双向队列里的插入和删除操作。通过调整节点的prev、next、调用析构函数来实现节点的插入和删除。
```
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
```
```
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
```
下述函数实现了从该LRU-K缓存中获取元素，如果未保存在LRU中，则返回空。获取元素时，首先将该元素在历史队列中记录的访问次数加一，判断是否大于等于K来决定是否需要加入到缓存队列中或更新其在缓存队列中的位置，随后访问缓存队列确定该值是否被缓存，若已被缓存，则将该值取出，否则返回空。
```
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
```
下述函数实现了向LRU-K缓存中添加元素。添加元素时，若元素未保存在缓存队列中，直接将该元素放在缓存队列队首，若该元素已在缓存队列中存在，则从队列中将元素删去，之后将该元素加到队首。
```
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
```