pub fn difficulty_label(d: i32) -> &'static str {
    match d {
        1 => "Very easy",
        2 => "Easy",
        3 => "Medium",
        4 => "Hard",
        5 => "Ambitious",
        _ => "—",
    }
}
