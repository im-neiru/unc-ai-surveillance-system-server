
pub trait WithSeperator where Self: Iterator + Sized {
    fn with_seperator<F>(self, new_separator: F) -> Separated<Self, F>
        where F: Fn() -> Self::Item;
}

impl<I: Iterator> WithSeperator for I {
    fn with_seperator<F>(self, new_separator: F) -> Separated<Self, F>
        where F: Fn() -> Self::Item {
            Separated {
            inner: self,
            new_separator,
            in_between: false
        }
    }
}

pub struct Separated<I, F>
    where I: Iterator, F: Fn() -> I::Item {
    inner: I,
    new_separator: F,
    in_between: bool
}

impl<I, F> Iterator for Separated<I, F>
    where I: Iterator, F: Fn() -> I::Item {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.in_between {
            self.in_between = false;
            return Some((self.new_separator)());
        }

        self.in_between = true;
        self.inner.next()
    }
}
