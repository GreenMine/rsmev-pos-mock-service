use std::{collections::VecDeque, time::Instant};

pub trait KeyGenerator {
    type Key: Clone + PartialEq<Self::Key>;

    fn generate() -> Self::Key;
}

pub struct UuidKey;
impl KeyGenerator for UuidKey {
    type Key = uuid::Uuid;

    fn generate() -> Self::Key {
        uuid::Uuid::new_v4()
    }
}

pub struct QueueItem<K, V> {
    key: K,
    value: V,
    taken: Option<Instant>,
}

impl<K, V> QueueItem<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            taken: None,
        }
    }
}

// TODO: two VecDeque(one for not taken, one for taken)
pub struct ConfirmQueue<T, const TTL: u64, KG: KeyGenerator = UuidKey> {
    container: VecDeque<QueueItem<KG::Key, T>>,
}

impl<T, const TTL: u64, KG: KeyGenerator> ConfirmQueue<T, TTL, KG> {
    pub const fn new() -> Self {
        Self {
            container: VecDeque::new(),
        }
    }

    pub fn add_with_key(&mut self, key: KG::Key, value: T) {
        self.container.push_back(QueueItem::new(key, value));
    }

    #[allow(unused)]
    pub fn add(&mut self, value: T) -> KG::Key {
        let key = KG::generate();
        self.add_with_key(key.clone(), value);
        key
    }

    pub fn take(&mut self) -> Option<(&KG::Key, &T)> {
        let v = self.container.back()?;
        let mut qi = match v.taken {
            None => self.container.pop_back(),
            Some(ttl) if ttl.elapsed() >= std::time::Duration::from_millis(TTL) => {
                self.container.pop_back()
            }
            _ => None,
        }?;

        qi.taken = Some(Instant::now());
        self.container.push_front(qi);

        let new_qi = self.container.front().unwrap();
        Some((&new_qi.key, &new_qi.value))
    }

    pub fn confirm(&mut self, key: &KG::Key) {
        let idx = self.container.iter().position(|q| q.key == *key).unwrap();
        self.container.remove(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::ConfirmQueue;

    #[test]
    pub fn test_fifo() {
        let mut queue = ConfirmQueue::<String, 10>::new();

        let _ = queue.add("random".to_string());
        let _ = queue.add("string".to_string());

        assert_eq!("string", queue.take().unwrap().1);
    }

    #[test]
    pub fn test_not_confirm() {
        let mut queue = ConfirmQueue::<String, 10>::new();

        let _ = queue.add("random".to_string());
        let _ = queue.add("string".to_string());

        let _ = queue.take().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (_, v2) = queue.take().unwrap();
        assert_eq!("random", v2);

        assert_eq!("string", queue.take().unwrap().1);
    }

    #[test]
    pub fn test_confirm() {
        let mut queue = ConfirmQueue::<String, 10>::new();

        let _ = queue.add("random".to_string());
        let _ = queue.add("string".to_string());

        let k1 = {
            let (k1, _) = queue.take().unwrap();
            k1.clone()
        };
        queue.confirm(&k1);
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (_, v2) = queue.take().unwrap();
        assert_eq!("random", v2);
    }
}
