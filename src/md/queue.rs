use std::fmt;

pub struct Queue<T> {
    buf: Vec<T>,
    start: usize,
}

impl<T: Clone> Queue<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            start: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len() - self.start
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.buf.push(value)
    }

    #[inline]
    pub fn pop_first(&mut self) -> Option<T> {
        let value = self.buf.get(self.start)?;
        self.start += 1;

        Some(value.clone())
    }

    #[inline]
    pub fn peek_first(&self) -> Option<&T> {
        let value = self.buf.get(self.start)?;

        Some(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Queue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&self.buf[self.start..]).finish()
    }
}

type Coords = (usize, usize);
pub fn pop_min2<T: Ord + Clone>(files: &mut [[Queue<T>; 3]]) -> Option<((T, T), Coords)> {
    let (_, x, y) = files
        .iter()
        .enumerate()
        .filter_map(|(y, queues)| {
            queues
                .iter()
                .enumerate()
                .filter_map(|(x, q)| {
                    if q.len() >= 2 {
                        Some((q.peek_first()?, x, y))
                    } else {
                        None
                    }
                })
                .min_by_key(|(min, _, _)| *min)
        })
        .min_by_key(|(min, _, _)| *min)?;

    let min = (
        files[y][x].pop_first().unwrap(),
        files[y][x].pop_first().unwrap(),
    );
    Some((min, (x, y)))
}
