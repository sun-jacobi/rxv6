use core::ptr::NonNull;

#[derive(Default, Clone, Copy)]
pub struct Node {
    next: Option<NonNull<Node>>,
    prev: Option<NonNull<Node>>,
}

#[derive(Clone, Copy)]
pub struct BareList {
    head: Option<NonNull<Node>>,
}

impl BareList {

    pub const fn new() -> Self {
        Self { head: None }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn push(&mut self, mut node: NonNull<Node>) {
        if let Some(mut head) = self.head {
            unsafe {
                node.as_mut().next = Some(head);
                head.as_mut().prev = Some(node);
            }
        }

        self.head = Some(node);
    }

    #[allow(dead_code)]
    pub fn head(&self) -> Option<NonNull<Node>> {
        self.head
    }

    pub fn pop(&mut self) -> Option<NonNull<Node>> {
        if let Some(mut head) = self.head {
            unsafe {
                if let Some(mut next) = head.as_mut().next {
                    next.as_mut().prev = head.as_mut().prev;
                }
            }

            self.head = unsafe { head.as_mut().next };

            return Some(head);
        }
        None
    }

    pub fn remove(&mut self, addr: u64) -> bool {
        let mut head = self.head;
        while let Some(mut node) = head {
            if node.addr().get() as u64 == addr {
                unsafe {
                    if node.as_mut().prev.is_none() && node.as_mut().next.is_none() {
                        self.head = None;
                    }

                    if let Some(mut prev) = node.as_mut().prev {
                        prev.as_mut().next = node.as_mut().next;
                    } else {
                        self.head = node.as_mut().next;
                    }

                    if let Some(mut next) = node.as_mut().next {
                        next.as_mut().prev = node.as_mut().prev;
                    }
                }

                return true;
            }

            head = unsafe { node.as_mut().next };
        }

        false
    }

    #[allow(dead_code)]
    pub fn contains(&self, addr: u64) -> bool {
        let mut head = self.head;
        while let Some(node) = head {
            if node.addr().get() as u64 == addr {
                return true;
            }

            head = unsafe { node.as_ref().next };
        }

        false
    }
}

impl Node {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            next: None,
            prev: None,
        }
    }

    pub fn from_addr(addr: u64) -> NonNull<Node> {
        let ptr = addr as *mut Node;
        unsafe { NonNull::new_unchecked(ptr) }
    }
}
