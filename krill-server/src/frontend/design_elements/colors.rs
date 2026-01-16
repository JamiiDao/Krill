pub struct TailwindColors;

impl TailwindColors {
    pub fn plaid() -> &'static str {
        "bg-gradient-to-r from-fuchsia-600 to-purple-600"
    }

    pub fn glassmorphism() -> &'static str {
        "bg-black/20 backdrop-blur-xl border border-white/20 shadow-3xl"
    }
}
