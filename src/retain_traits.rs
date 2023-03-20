pub trait EnumeratedRetain<T> {
    fn retain_enumerate<F>(&mut self, f: F)
    where
        F: FnMut(usize, &T) -> bool;
}

impl<T> EnumeratedRetain<T> for Vec<T> {
    fn retain_enumerate<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(i, &v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}

pub trait SplitLastRetain<T> {
    fn retain_split_last<F>(&mut self, f: F)
    where
        F: FnMut(&T, &[T]) -> bool;
}

impl<T> SplitLastRetain<T> for Vec<T> {
    fn retain_split_last<F>(&mut self, mut f: F)
    where
        F: FnMut(&T, &[T]) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&v[i], &v[i + 1..]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}
