//! Parameter construction for requests: URL query params and POST form bodies.

use serde::ser::SerializeMap;
use std::collections::HashMap;

/// Helper for building URL query parameters.
pub struct Params {
    items: Vec<(String, String)>,
}

/// Form data wrapper that preserves insertion order for POST bodies.
pub struct FormData(Vec<(String, String)>);

impl serde::Serialize for FormData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// Helper for building form data (POST body).
pub struct FormBuilder {
    inner: Vec<(String, String)>,
}

impl FormBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    #[must_use]
    pub fn push<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.push((key.into(), value.into()));
        self
    }

    /// Push a key-value pair where the value implements `Display` (e.g. `i64`).
    #[must_use]
    pub fn push_val<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.inner.push((key.into(), value.to_string()));
        self
    }

    #[must_use]
    pub fn push_opt<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        if let Some(v) = value {
            self.inner.push((key.into(), v.into()));
        }
        self
    }

    /// Push an optional key-value pair where the value implements `Display`.
    #[must_use]
    pub fn push_opt_val<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        if let Some(v) = value {
            self.inner.push((key.into(), v.to_string()));
        }
        self
    }

    #[must_use]
    pub fn csrf(self, token: String) -> Self {
        self.push("csrf", token)
    }

    #[must_use]
    pub fn build(self) -> FormData {
        FormData(self.inner)
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Params {
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    #[must_use]
    pub fn push<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.items.push((key.into(), value.into()));
        self
    }

    /// Push a key-value pair where the value implements `Display` (e.g. `i64`).
    #[must_use]
    pub fn push_val<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.items.push((key.into(), value.to_string()));
        self
    }

    #[must_use]
    pub fn push_opt<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        if let Some(v) = value {
            self.items.push((key.into(), v.into()));
        }
        self
    }

    /// Push an optional key-value pair where the value implements `Display`.
    #[must_use]
    pub fn push_opt_val<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        if let Some(v) = value {
            self.items.push((key.into(), v.to_string()));
        }
        self
    }

    #[must_use]
    pub fn as_query(&self) -> Vec<(&str, &str)> {
        self.items
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    #[must_use]
    pub fn into_hashmap(self) -> HashMap<String, String> {
        self.items.into_iter().collect()
    }

    pub(crate) fn insert(&mut self, key: String, value: String) {
        self.items.push((key, value));
    }

    pub(crate) fn sort_by_key(&mut self) {
        self.items.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.items.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for Params {
    fn from(map: HashMap<String, String>) -> Self {
        Self {
            items: map.into_iter().collect(),
        }
    }
}

impl From<Vec<(String, String)>> for Params {
    fn from(vec: Vec<(String, String)>) -> Self {
        Self { items: vec }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_new_is_empty() {
        let p = Params::new();
        assert!(p.iter().next().is_none());
    }

    #[test]
    fn params_push_chain() {
        let p = Params::new().push("a", "1").push("b", "2");
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("a", "1"));
        assert_eq!(items[1], ("b", "2"));
    }

    #[test]
    fn params_as_query() {
        let p = Params::new().push("key", "val");
        let q = p.as_query();
        assert_eq!(q, vec![("key", "val")]);
    }

    #[test]
    fn params_into_hashmap() {
        let p = Params::new().push("x", "1").push("y", "2");
        let map = p.into_hashmap();
        assert_eq!(map.get("x").unwrap(), "1");
        assert_eq!(map.get("y").unwrap(), "2");
    }

    #[test]
    fn params_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("k".to_string(), "v".to_string());
        let p = Params::from(map);
        assert_eq!(p.iter().count(), 1);
    }

    #[test]
    fn params_from_vec() {
        let v = vec![("a".to_string(), "1".to_string())];
        let p = Params::from(v);
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items, vec![("a", "1")]);
    }

    #[test]
    fn params_push_val() {
        let p = Params::new().push("a", "str").push_val("b", 42i64);
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items, vec![("a", "str"), ("b", "42")]);
    }

    #[test]
    fn params_push_opt_val_none() {
        let p = Params::new().push("a", "1").push_opt_val("b", None::<i64>);
        assert_eq!(p.iter().count(), 1);
    }

    #[test]
    fn params_push_opt_val_some() {
        let p = Params::new().push_opt_val("b", Some(42i64));
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items, vec![("b", "42")]);
    }

    #[test]
    fn form_builder_push() {
        let f = FormBuilder::new().push("type", "1").push("oid", "123");
        let data = f.build();
        assert_eq!(data.0.len(), 2);
        assert_eq!(data.0[0], ("type".to_string(), "1".to_string()));
        assert_eq!(data.0[1], ("oid".to_string(), "123".to_string()));
    }

    #[test]
    fn form_builder_push_opt_none() {
        let f = FormBuilder::new()
            .push("a", "1")
            .push_opt("b", None::<String>);
        let data = f.build();
        assert_eq!(data.0.len(), 1);
    }

    #[test]
    fn form_builder_push_opt_some() {
        let f = FormBuilder::new().push_opt("b", Some("val"));
        let data = f.build();
        assert_eq!(data.0, vec![("b".to_string(), "val".to_string())]);
    }

    #[test]
    fn form_builder_csrf() {
        let f = FormBuilder::new()
            .push("aid", "1")
            .csrf("token123".to_string());
        let data = f.build();
        assert_eq!(data.0.last().unwrap().0, "csrf");
        assert_eq!(data.0.last().unwrap().1, "token123");
    }

    #[test]
    fn form_data_serializes_as_map() {
        let f = FormBuilder::new().push("k", "v");
        let data = f.build();
        let json = serde_json::to_value(&data).unwrap();
        assert!(json.is_object());
        assert_eq!(json["k"], "v");
    }
}
