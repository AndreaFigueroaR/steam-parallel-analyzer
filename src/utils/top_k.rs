use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// `TopK<T>` mantiene los `k` elementos máximos de tipo `T`.
/// Usa un min-heap para que el elemento más pequeño de los top k sea accesible y reemplazable por un "mejor" elemento en O(log k)
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

    /// Indica, en O(1) y sin construir el elemento, si un candidato sería incluido.
    /// `ranks_above_min` recibe el menor de los top k actuales y debe devolver si el candidato es mayor.
    /// Permite evitar copias costosas (p. ej. el texto de una review) de candidatos que serían descartados.
    pub fn admits(&self, ranks_above_min: impl FnOnce(&T) -> bool) -> bool {
        if self.capacity == 0 {
            return false;
        }
        match self.heap.peek() {
            Some(smallest) if self.heap.len() >= self.capacity => ranks_above_min(&smallest.0),
            _ => true,
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
        } else if let Some(mut smallest) = self.heap.peek_mut()
            && value > smallest.0
        {
            // Reemplazo in-place: un único sift-down en lugar de pop + push.
            smallest.0 = value;
        }
    }

    /// Extrae y devuelve todos los top k en un `Vec` **sin** ordenarlos.
    /// Consumirá el heap, pero evita el coste de ordenamiento (O(k)).
    pub fn into_unsorted(self) -> Vec<T> {
        self.heap.into_iter().map(|rev| rev.0).collect()
    }

    /// Extrae y devuelve todos los top k ordenados de mayor a menor (O(k log k)).
    pub fn into_sorted_desc(self) -> Vec<T> {
        // El orden ascendente de `Reverse<T>` es el orden descendente de `T`.
        self.heap
            .into_sorted_vec()
            .into_iter()
            .map(|rev| rev.0)
            .collect()
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
