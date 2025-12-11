pub fn truncate_with_ellipsis(mut s: String, max_len: usize) -> String {
    let upto = s.char_indices().map(|(i, _)| i).nth(max_len);
    if let Some(upto) = upto {
        s.truncate(upto);
        s.push_str("...");
    }
    s
}
