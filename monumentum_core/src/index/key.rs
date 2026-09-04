use monumentum_handler::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexKey {
    Integer(i64),
    Float(u64),
    Text(String),
    Blob(Vec<u8>),
    Boolean(bool),
}

impl IndexKey {
    #[must_use]
    pub fn from_value(v: &Value) -> Option<Self> {
        match v {
            Value::Null => None,
            Value::Integer(i) => Some(Self::Integer(i.as_i64())),
            Value::Float(f) => {
                let bits = f.as_f64().to_bits();
                let bits = if f.as_f64() == 0.0 {
                    0.0_f64.to_bits()
                } else {
                    bits
                };
                Some(Self::Float(bits))
            }
            Value::Text(t) => Some(Self::Text(t.as_str().to_string())),
            Value::Blob(b) => Some(Self::Blob(b.as_slice().to_vec())),
            Value::Boolean(b) => Some(Self::Boolean(*b)),
            _ => None,
        }
    }
}
