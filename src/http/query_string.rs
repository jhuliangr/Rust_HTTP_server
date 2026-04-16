use std::collections::HashMap;

/// Parsed representation of a URL query string (e.g., `?color=red&size=large`).
///
/// Borrows from the original request buffer (`'buf`) to avoid allocating new strings.
/// Supports duplicate keys: `?tag=rust&tag=http` produces `Multiple(["rust", "http"])`.
#[derive(Debug)]
pub struct QueryString<'buf> {
    data: HashMap<&'buf str, Value<'buf>>,
}

/// A query parameter value that can be a single string or a list.
///
/// Most query params appear once (`Single`), but HTML forms with checkboxes
/// or multi-selects can repeat the same key, which we store as `Multiple`.
#[derive(Debug)]
pub enum Value<'buf> {
    Single(&'buf str),
    Multiple(Vec<&'buf str>),
}

impl<'buf> QueryString<'buf> {
    pub fn get(&self, key: &str) -> Option<&Value<'buf>> {
        self.data.get(key)
    }
}

/// Parses a raw query string like `name=jhulian&lang=rust&lang=elixir`.
///
/// The `entry` API is key here: it lets us handle the "first occurrence vs. repeated key"
/// logic in a single pass without checking if the key already exists.
/// - First time we see a key → `or_insert` stores it as `Single`
/// - Second time → `and_modify` promotes it from `Single` to `Multiple`
/// - Third+ time → `and_modify` just pushes onto the existing `Vec`
impl<'buf> From<&'buf str> for QueryString<'buf> {
    fn from(raw_query: &'buf str) -> Self {
        let mut data = HashMap::new();

        for key_value_pair in raw_query.split('&') {
            let (key, value) = match key_value_pair.find('=') {
                Some(separator_index) => (
                    &key_value_pair[..separator_index],
                    &key_value_pair[separator_index + 1..],
                ),
                // Keys without values (e.g., "?debug") get an empty string
                None => (key_value_pair, ""),
            };

            data.entry(key)
                .and_modify(|existing: &mut Value| match existing {
                    Value::Single(previous_value) => {
                        *existing = Value::Multiple(vec![previous_value, value]);
                    }
                    Value::Multiple(values) => values.push(value),
                })
                .or_insert(Value::Single(value));
        }

        QueryString { data }
    }
}
