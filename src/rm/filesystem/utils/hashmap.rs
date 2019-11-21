use super::linklist::LinkList;

#[derive(Debug)]
struct DataNode {
    key1: i32,
    key2: i32,
}

pub struct Hashmap {
    modd: i32,
    list: LinkList,
    a: Vec<DataNode>,
}

impl Hashmap {
    fn hash(&self, k1: i32, k2: i32) -> i32 {
        ((k1 + k2) * (k1 + k2 + 1) / 2 + k2) % self.modd
    }

    pub fn print(&self) {
        println!("empty check begin");
        for i in &self.a {
            if i.key1 != -1 || i.key2 != -1 {
                println!("{:?}", i);
            }
        }
        println!("empty check end");
    }

    pub fn find_index(&self, k1: i32, k2: i32) -> i32 {
        let h = self.hash(k1, k2);
        let mut p = self.list.get_first(h);
        while !self.list.is_head(p) {
            if self.a[p as usize].key1 == k1 && self.a[p as usize].key2 == k2 {
                return p;
            }
            p = self.list.next(p);
        }
        return -1;
    }

    pub fn replace(&mut self, index: i32, k1: i32, k2: i32) {
        let h = self.hash(k1, k2);
        self.list.insert_first(h, index);
        self.a[index as usize].key1 = k1;
        self.a[index as usize].key2 = k2;
    }

    pub fn remove(&mut self, index: i32) {
        self.list.del(index);
        self.a[index as usize].key1 = -1;
        self.a[index as usize].key2 = -1;
    }

    pub fn get_keys(&self, index: i32) -> (i32, i32) {
        (self.a[index as usize].key1, self.a[index as usize].key2)
    }

    pub fn new(c: i32, m: i32) -> Self {
        let mut v: Vec<DataNode> = Vec::with_capacity(c as usize);
        for _i in 0..c {
            v.push(DataNode{key1: -1, key2: -1});
        }

        Self {
            modd: m,
            a: v,
            list: LinkList::new(c, m),
        }
    }
}

