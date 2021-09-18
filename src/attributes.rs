use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AttributeKeyValue {
    pub(crate) key: String,
    pub(crate) value: String,
}

pub trait ParsePathToAttributes {
    fn parse(self, pointer: &str) -> Vec<AttributeKeyValue>;
}

impl ParsePathToAttributes for serde_yaml::Value {
    fn parse(self, pointer: &str) -> Vec<AttributeKeyValue> {
        Vec::deserialize(
            yaml_pointer(&self, pointer)
                .expect("Bad JSON Pointer")
                .clone(),
        )
        .expect("Invalid attribute array")
    }
}

impl ParsePathToAttributes for serde_json::Value {
    fn parse(self, pointer: &str) -> Vec<AttributeKeyValue> {
        Vec::deserialize(self.pointer(pointer).expect("Bad JSON Pointer").clone())
            .expect("Invalid attribute array")
    }
}

// Copied from `serde_json`
fn yaml_pointer<'a>(value: &'a serde_yaml::Value, pointer: &str) -> Option<&'a serde_yaml::Value> {
    if pointer.is_empty() {
        return Some(value);
    }
    if !pointer.starts_with('/') {
        return None;
    }
    pointer
        .split('/')
        .skip(1)
        .map(|x| x.replace("~1", "/").replace("~0", "~"))
        .try_fold(value, |target, token| match target {
            serde_yaml::Value::Mapping(map) => map.get(&serde_yaml::Value::String(token)),
            serde_yaml::Value::Sequence(list) => parse_index(&token).and_then(|x| list.get(x)),
            _ => None,
        })
}

// Copied from `serde_json`
fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}
