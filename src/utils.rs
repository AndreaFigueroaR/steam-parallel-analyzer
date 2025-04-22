use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// `TopK<T>` mantiene los `k` elementos máximos de tipo `T`.
/// Usa un min-heap (almacenado como BinaryHeap<Reverse<T>>) para que
/// el elemento más pequeño de los top k esté siempre en la raíz.
pub struct TopK<T: Ord> {
    heap: BinaryHeap<Reverse<T>>,
    capacity: usize,
}

impl<T: Ord> TopK<T> {
    /// Crea un nuevo TopK con capacidad para `k` elementos.
    /// Si `k == 0`, el heap nunca almacenará elementos.
    pub fn new(k: usize) -> Self {
        TopK {
            heap: BinaryHeap::with_capacity(k),
            capacity: k,
        }
    }

    /// Intenta insertar `value` en los top k.
    /// - Si aún no hay k elementos, se inserta directamente.
    /// - Si ya hay k elementos y `value` es mayor que el más pequeño
    ///   de los actuales (es decir, el tope del min-heap), reemplaza
    ///   ese tope; en caso contrario, se descarta.
    pub fn consider_include(&mut self, value: T) {
        if self.capacity == 0 {
            return;
        }

        if self.heap.len() < self.capacity {
            self.heap.push(Reverse(value));
        } else if let Some(smallest) = self.heap.peek() {
            if value > (smallest.0) {
                self.heap.pop();
                self.heap.push(Reverse(value));
            }
        }
    }

    /// Extrae y devuelve todos los top k en un `Vec` **sin** ordenarlos.
    /// Consumirá el heap, pero evita el coste de ordenamiento (O(k)).
    pub fn into_unsorted(self) -> Vec<T> {
        self.heap.into_iter().map(|rev| rev.0).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TopK;

    #[test]
    fn basic_topk_unsorted() {
        let mut top3 = TopK::new(3);
        for x in &[5, 1, 7, 3, 9, 2] {
            top3.consider_include(*x);
        }
        let mut result = top3.into_unsorted();
        result.sort_unstable();
        assert_eq!(result, vec![5, 7, 9]);
    }
}
