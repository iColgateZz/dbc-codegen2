pub fn map_into<T, U>(iter: impl IntoIterator<Item = T>) -> Vec<U>
where
    T: Into<U>,
{
    iter.into_iter().map(Into::into).collect()
}

pub trait ToUpperCamelCase {
    fn to_upper_camelcase(&self) -> String;
}

impl ToUpperCamelCase for str {
    fn to_upper_camelcase(&self) -> String {
        let mut result = String::with_capacity(self.len());
        let mut capitalize_next = true;

        for c in self.chars() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        }

        result
    }
}
