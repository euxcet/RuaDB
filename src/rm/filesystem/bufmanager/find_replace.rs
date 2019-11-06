use super::super::utils::linklist::LinkList;

pub struct FindReplace {
    pub list: LinkList,
    cap: i32,
}

impl FindReplace {
    pub fn free(&mut self, index: i32) {
        self.list.insert_first(0, index);
    }

    pub fn access(&mut self, index: i32) {
        self.list.insert(0, index);
    }

    pub fn find(&mut self) -> i32 {
        let index = self.list.get_first(0);
        self.list.del(index);
        self.list.insert(0, index);
        index
    }

    pub fn new(c: i32) -> Self {
        let mut f = Self {
            cap: c,
            list: LinkList::new(c, 1),
        };

        for i in 0..c {
            f.list.insert(0, i);
        }

        f
    }
}