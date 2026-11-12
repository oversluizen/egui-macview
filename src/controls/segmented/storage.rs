//! Safe stack-inline buffer (`N = 8`) eliminating per-frame `Vec` heap allocations for segmented controls.

/// Maximum number of segments stored inline on the stack before spilling to a heap `Vec`.
pub(crate) const INLINE_CAP: usize = 8;

/// Safe inline-first collection storing up to [`INLINE_CAP`] elements on the stack with zero `unsafe`.
#[derive(Clone)]
pub(crate) enum InlineBuf<T> {
    Inline {
        slots: [Option<T>; INLINE_CAP],
        len: usize,
    },
    Heap(Vec<T>),
}

impl<T> InlineBuf<T> {
    pub(crate) const fn new() -> Self {
        Self::Inline {
            slots: [const { None }; INLINE_CAP],
            len: 0,
        }
    }

    #[inline]
    pub(crate) const fn is_inline(&self) -> bool {
        matches!(self, Self::Inline { .. })
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Inline { len, .. } => *len,
            Self::Heap(v) => v.len(),
        }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        match self {
            Self::Inline { slots, len } => {
                if index < *len {
                    slots[index].as_ref()
                } else {
                    None
                }
            }
            Self::Heap(v) => v.get(index),
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        match self {
            Self::Inline { slots, len } => {
                if *len < INLINE_CAP {
                    slots[*len] = Some(item);
                    *len += 1;
                } else {
                    let mut vec = Vec::with_capacity(INLINE_CAP + 4);
                    for slot in slots.iter_mut() {
                        if let Some(existing) = slot.take() {
                            vec.push(existing);
                        }
                    }
                    vec.push(item);
                    *self = Self::Heap(vec);
                }
            }
            Self::Heap(v) => v.push(item),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        let (inline_slice, heap_slice): (&[Option<T>], &[T]) = match self {
            Self::Inline { slots, len } => (&slots[..*len], &[]),
            Self::Heap(v) => (&[], v.as_slice()),
        };
        inline_slice.iter().filter_map(Option::as_ref).chain(heap_slice.iter())
    }
}

impl<T> FromIterator<T> for InlineBuf<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let it = iter.into_iter();
        let (lower, _) = it.size_hint();
        if lower > INLINE_CAP {
            return Self::Heap(it.collect());
        }
        let mut buf = Self::new();
        for item in it {
            buf.push(item);
        }
        buf
    }
}

pub(crate) enum InlineBufIter<T> {
    Inline {
        slots: [Option<T>; INLINE_CAP],
        idx: usize,
        len: usize,
    },
    Heap(std::vec::IntoIter<T>),
}

impl<T> Iterator for InlineBufIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Inline { slots, idx, len } => {
                while *idx < *len {
                    let cur = *idx;
                    *idx += 1;
                    if let Some(val) = slots[cur].take() {
                        return Some(val);
                    }
                }
                None
            }
            Self::Heap(it) => it.next(),
        }
    }
}

impl<T> IntoIterator for InlineBuf<T> {
    type Item = T;
    type IntoIter = InlineBufIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Inline { slots, len } => InlineBufIter::Inline { slots, idx: 0, len },
            Self::Heap(v) => InlineBufIter::Heap(v.into_iter()),
        }
    }
}
