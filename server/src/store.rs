use dashmap::DashMap;

const HASHMAP_SIZE: usize = 256;

pub struct Store<'a>(DashMap<usize, &'a str>);

impl<'a> Store<'a> {
    pub fn init() -> Self {
        Self(DashMap::with_capacity(HASHMAP_SIZE))
    }

    pub fn insert<K, V>(&self, key: K, value: V) -> Option<&'a str>
    where
        K: Into<usize>,
        V: Into<&'a str>,
    {
        // @todo impl replacement
        self.0.insert(key.into(), value.into())
    }

    pub fn get<K>(&self, key: K) -> Option<&'a str>
    where
        K: Into<usize>,
    {
        self.0.get(&key.into()).map(|x| *x)
    }

    pub fn remove<K>(&self, key: K) -> Option<(usize, &'a str)>
    where
        K: Into<usize>,
    {
        self.0.remove(&key.into())
    }
    
    
}
