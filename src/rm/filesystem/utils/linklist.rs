struct ListNode {
    next: i32, 
    prev: i32,
}

pub struct LinkList {
    cap: i32,
    list_num: i32,
    a: Vec<ListNode>,
} 

impl LinkList {
    pub fn print_head(&self, list_id: i32, len: usize) {
        let mut node = list_id + self.cap;
        print!("head elem of {} :", list_id);
        for _ in 0..len {
            node = self.a[node as usize].next;
            print!("{} ", node);
        }
        println!();
    }

    pub fn print_tail(&self, list_id: i32, len: usize) {
        let mut node = list_id + self.cap;
        print!("tail elem of {} :", list_id);
        for _ in 0..len {
            // print!("{} ", self.a[node as usize].prev);
            node = self.a[node as usize].prev;
        }
        for _ in 0..len {
            print!("{} ", node);
            node = self.a[node as usize].next;
        }
        println!();
    }


    fn link(&mut self, prev: i32, next: i32) {
        self.a[prev as usize].next = next;
        self.a[next as usize].prev = prev;
    }

    pub fn del(&mut self, index: i32) {
        if self.a[index as usize].prev == index {
            return;
        }

        self.link(self.a[index as usize].prev, self.a[index as usize].next);
        self.link(index, index);
    }

    pub fn insert(&mut self, list_id: i32, elem: i32) {
        self.del(elem);
        let node = list_id + self.cap;
        let prev = self.a[node as usize].prev;
        self.link(prev, elem);
        self.link(elem, node);
    }

    pub fn insert_first(&mut self, list_id: i32, elem: i32) {
        self.del(elem);
        let node = list_id + self.cap;
        let next = self.a[node as usize].next;
        self.link(node, elem);
        self.link(elem, next);
    }

    pub fn get_first(&self, list_id: i32) -> i32 {
        self.a[(list_id + self.cap) as usize].next
    }

    pub fn next(&self, index: i32) -> i32 {
        self.a[index as usize].next
    }

    pub fn is_head(&self, index: i32) -> bool {
        index >= self.cap
    }

    pub fn is_alone(&self, index: i32) -> bool {
        self.a[index as usize].next == index
    }

    pub fn new(c: i32, n: i32) -> Self {
        let mut v: Vec<ListNode> = Vec::with_capacity((c + n) as usize);
        for i in 0..(c + n) {
            v.push(ListNode{next: i, prev: i})
        }

        Self {
            cap: c,
            list_num: n,
            a: v,
        }
    }
}
