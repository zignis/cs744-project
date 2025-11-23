use thousands::Separable;

/// formats microseconds to milliseconds
pub fn us_to_ms(us: f64) -> String {
    format!("{:.2}", us / 1000.0).separate_with_commas()
}
