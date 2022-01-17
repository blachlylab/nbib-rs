
/// GroupBy Iterator Object
pub struct GroupBy<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item, &I::Item) -> bool
{
    /// Input Iterator
    range: I, 

    /// Predicate function to groupby on
    pred: P,  

    /// Buffer to hold last item
    /// I would just store an Option<I::Item> here
    /// but the next function takes a mutable reference of the iterator
    /// and that prevents me from taking the last item
    /// from GroupBy and updating it without cloning. 
    /// 
    /// So we use a Vec and pop out the last item and 
    /// push in the new last item.
    buf: Vec<I::Item>, 
}

/// implement Iterator trait
impl<I, P> Iterator for GroupBy<I, P> 
where
    I: Iterator,
    P: FnMut(&I::Item, &I::Item) -> bool
{
    type Item = std::vec::IntoIter::<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut arr: Vec<I::Item> = Vec::new();
        if self.buf.len() == 0 {
            return None
        }
        arr.push(self.buf.pop().unwrap());
        while let Some(b) = self.range.next() {
            if (self.pred)(&arr[0], &b) {
                arr.push(b);
            } else {
                self.buf.push(b);
                break;
            }
        } 
        Some(arr.into_iter())
    }
}

/// Create new trait for any iterator that provides function group_by
/// i.e range.group_by(|a, b| a == b)
pub trait GroupByItr: Iterator {
    fn group_by<'a, P>(self, pred: P) -> GroupBy<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item, &Self::Item) -> bool
    {
        // preload the iterator
        // without cloning
        let mut itr = self.into_iter();
        let front = itr.next();
        let mut arr: Vec<Self::Item> = Vec::new();
        if !front.is_none() {
            arr.push(front.unwrap());
        }
        GroupBy {
            range: itr,
            pred: pred,
            buf: arr,
        }
    }
}

/// Apparently this is need to implement the Trait
impl<I: Iterator> GroupByItr for I {}


#[cfg(test)]
mod tests {

    #[test]
    fn test_groupby() {
        use crate::groupby::GroupByItr;
        let range = vec![
            String::from("test"),
            String::from("test"),
            String::from("test2"),
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("c"),
            String::from("d"),
            String::from("e"),
        ];

        let val = range.clone().into_iter().group_by(|a, b| a == b).map(|x| x.collect::<Vec<String>>()).collect::<Vec<Vec<String>>>();
        let correct = vec![
            vec![String::from("test"),
            String::from("test")],
            vec![String::from("test2")],
            vec![String::from("a")],
            vec![String::from("b")],
            vec![String::from("c"),
            String::from("c")],
            vec![String::from("d")],
            vec![String::from("e")],
        ];
        assert_eq!(val, correct);

        let val = range.into_iter().group_by(|a, b| a == b).map(|x| x.into_iter().nth(0).unwrap()).collect::<Vec<String>>();
        let correct = vec![
            String::from("test"),
            String::from("test2"),
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("d"),
            String::from("e"),
        ];
        assert_eq!(val, correct);

    }
}
