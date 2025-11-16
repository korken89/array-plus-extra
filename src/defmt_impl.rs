use crate::ArrayPlusExtra;
use defmt::Format;

// Forward Format to slice implementation.
impl<T, const N: usize, const EXTRA: usize> Format for ArrayPlusExtra<T, N, EXTRA>
where
    T: Format,
{
    fn format(&self, fmt: defmt::Formatter) {
        self.as_slice().format(fmt)
    }
}
