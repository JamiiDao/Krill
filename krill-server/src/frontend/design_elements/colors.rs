use std::borrow::Cow;

pub struct TailwindColors;

impl TailwindColors {
    pub fn background(color: &str) -> Cow<str> {
        Cow::Borrowed("h-full w-full bg-[")
            + color
            + "] rounded-md bg-clip-padding backdrop-filter backdrop-blur-3xl "
            + "bg-opacity-0 border border-gray-100"
    }
}
