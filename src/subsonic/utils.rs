use serde::{Deserialize, Deserializer};

pub fn deserialize_vec<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany<T> {
        One(T),
        Many(Vec<T>),
    }

    match Option::<OneOrMany<T>>::deserialize(deserializer)? {
        Some(OneOrMany::One(x)) => Ok(Some(vec![x])),
        Some(OneOrMany::Many(xs)) => Ok(Some(xs)),
        None => Ok(None),
    }
}
