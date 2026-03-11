pub struct TailwindColors;

impl TailwindColors {
    pub fn glassmorphism_yellow_yellow_stone() -> &'static str {
        "bg-gradient-to-bl from-yellow-600 via-yellow-800 to-stone-900"
    }

    pub fn glassmorphism_sky_indigo_zinc() -> &'static str {
        "bg-gradient-to-br from-sky-400 via-indigo-900 to-zinc-900"
    }

    pub fn glassmorphism_overlay() -> &'static str {
        "absolute inset-0 bg-white/20 backdrop-blur-xl border border-white/20 shadow-2xl"
    }

    pub fn glassmorphism_hover_yellow_yellow_stone() -> &'static str {
        "hover:bg-gradient-to-bl hover:from-yellow-600 hover:via-yellow-800 hover:to-stone-900"
    }

    pub fn glassmorphism_hover_sky_indigo_zinc() -> &'static str {
        "hover:bg-gradient-to-br hover:from-sky-400 hover:via-indigo-900 hover:to-zinc-900"
    }
}
