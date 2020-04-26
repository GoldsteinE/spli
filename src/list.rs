use replace_with::replace_with_or_abort_and_return;
use std::fmt::{self, Write};
use std::sync::Arc;

pub type Link<T> = Option<Arc<ListNode<T>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListNode<T> {
    pub val: Arc<T>,
    pub next: Link<T>,
}

#[derive(Debug)]
pub struct List<T> {
    pub head: Link<T>,
    length: usize,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn cons(&self, val: T) -> Self {
        Self {
            head: Some(Arc::new(ListNode {
                val: Arc::new(val),
                next: self.head.clone(),
            })),
            length: self.length + 1,
        }
    }

    pub fn head(&self) -> Option<Arc<T>> {
        self.head.as_ref().map(|node| node.val.clone())
    }

    pub fn tail(&self) -> Option<Self> {
        self.head.as_ref().map(|node| List {
            head: node.next.clone(),
            length: self.length - 1,
        })
    }

    pub fn pop_node(&mut self) -> Link<T> {
        self.length = self.length.saturating_sub(1);
        replace_with_or_abort_and_return(&mut self.head, move |head| {
            (head.clone(), head.map(|node| node.next.clone()).flatten())
        })
    }

    pub fn pop(&mut self) -> Option<Arc<T>> {
        self.pop_node().map(|node| node.val.clone())
    }

    pub fn reverse_from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut result = Self::new();
        for item in iter.into_iter() {
            result = result.cons(item)
        }
        result
    }

    pub fn from_double_ended_iter<I, It>(iter: I) -> Self
    where
        It: DoubleEndedIterator<Item = T>,
        I: IntoIterator<Item = T, IntoIter = It>,
    {
        Self::reverse_from_iter(iter.into_iter().rev())
    }

    pub fn iter(&self) -> impl Iterator<Item = Arc<T>> {
        self.clone().into_iter()
    }
}

impl<T: PartialEq> List<T> {
    fn elems_eq(&self, other: &Self) -> bool {
        for (left, right) in self.iter().zip(other.iter()) {
            if left != right {
                return false;
            }
        }
        true
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_node() {}
    }
}

impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
            length: self.length,
        }
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length && self.elems_eq(other)
    }
}

impl<T: Eq> Eq for List<T> {}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = Arc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_node().map(|node| node.val.clone())
    }
}

impl<T> IntoIterator for List<T> {
    type IntoIter = IntoIter<T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T: fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut first_item = true;
        fmt.write_char('(')?;
        for item in self.iter() {
            if !first_item {
                fmt.write_char(' ')?;
            } else {
                first_item = false;
            }
            if fmt.alternate() {
                write!(fmt, "{:#}", *item)?;
            } else {
                write!(fmt, "{}", *item)?;
            }
        }
        fmt.write_char(')')
    }
}

#[macro_export]
macro_rules! list_impl {
    ([] => $($reversed:tt)*) => {
        $crate::list::List::new()$(.cons($reversed))*
    };
    ([$head:tt $($tail:tt)*] => $($reversed:tt)*) => {
        $crate::list_impl!([$($tail)*] => $head $($reversed)*)
    };
}

#[macro_export]
macro_rules! list {
    ($($values:expr),*$(,)?) => {
        $crate::list_impl!([$($values)*] =>)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let list: List<()> = List::new();
        assert_eq!(list.head, None);
    }

    #[test]
    fn test_default() {
        let list: List<()> = Default::default();
        assert_eq!(list.head, None);
    }

    fn make_123() -> List<i32> {
        List::new().cons(3).cons(2).cons(1)
    }

    #[test]
    fn test_list_macro() {
        assert_eq!(list![1, 2, 3], make_123());
        assert_eq!(list![1], List::new().cons(1));
        // trailing comma
        assert_eq!(list![1, 2, 3,], make_123());
        assert_eq!(list![1,], List::new().cons(1));
        // empty
        assert_eq!(list![], List::<()>::new());
    }

    #[test]
    fn test_head() {
        assert_eq!(make_123().head(), Some(Arc::new(1)));
    }

    #[test]
    fn test_head_empty() {
        assert_eq!(List::<()>::new().head(), None);
    }

    #[test]
    fn test_tail() {
        assert_eq!(make_123().tail(), Some(list![2, 3]));
    }

    #[test]
    fn test_tail_empty() {
        assert_eq!(List::<()>::new().tail(), None);
    }

    #[test]
    fn test_pop_node() {
        let mut list = make_123();
        for num in 1..=3 {
            let node = list.pop_node().expect("too few items");
            assert_eq!(node.val, Arc::new(num));
        }
        assert_eq!(list, List::new());
        assert_eq!(list.pop_node(), None);
    }

    #[test]
    fn test_reverse_from_iter() {
        struct OneWayIter<'a, T>(&'a [T], usize);

        impl<T: Copy> Iterator for OneWayIter<'_, T> {
            type Item = T;

            fn next(&mut self) -> Option<T> {
                self.1 += 1;
                if self.1 > self.0.len() {
                    None
                } else {
                    Some(self.0[self.1 - 1])
                }
            }
        }

        assert_eq!(
            List::reverse_from_iter(OneWayIter(&[3, 2, 1], 0)),
            make_123()
        );
    }

    #[test]
    fn test_from_double_ended_iter() {
        assert_eq!(List::from_double_ended_iter(vec![1, 2, 3]), make_123());
    }

    #[test]
    fn test_eq() {
        assert_eq!(list![1, 2, 3, 4], list![1, 2, 3, 4]);
        assert_ne!(list![1, 2, 3], list![1, 2, 3, 4]);
        assert_ne!(list![1, 2, 3, 5], list![1, 2, 3, 4]);
    }

    fn test_123_iter(mut iter: impl Iterator<Item = Arc<i32>>) {
        assert_eq!(*iter.next().unwrap(), 1);
        assert_eq!(*iter.next().unwrap(), 2);
        assert_eq!(*iter.next().unwrap(), 3);
        assert_eq!(iter.next(), None);

        // Trying to use normally
        struct A;
        fn func(_: &A) {}
        func(&list![A].into_iter().next().unwrap());
    }

    #[test]
    fn test_into_iter() {
        let iter = make_123().into_iter();
        test_123_iter(iter);
    }

    #[test]
    fn test_iter() {
        let list = make_123();
        let iter = make_123().iter();
        test_123_iter(iter);
        // Testing that list is not changed
        assert_eq!(list, make_123());
    }

    #[test]
    fn test_len() {
        let mut list = List::new();
        assert_eq!(list.len(), 0);
        list = list.cons(1);
        assert_eq!(list.len(), 1);
        list = list.cons(2);
        assert_eq!(list.len(), 2);
        list = list.cons(3);
        assert_eq!(list.len(), 3);
        list = list.tail().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_clone() {
        let mut first_list = make_123();
        let mut second_list = first_list.clone();
        first_list = first_list.cons(4);
        second_list = second_list.cons(5);
        assert_eq!(first_list, list![4, 1, 2, 3]);
        assert_eq!(second_list, list![5, 1, 2, 3]);
    }
}
