pub fn map_into<T, U>(iter: impl IntoIterator<Item = T>) -> Vec<U>
where
    T: Into<U>,
{
    iter.into_iter().map(Into::into).collect()
}
