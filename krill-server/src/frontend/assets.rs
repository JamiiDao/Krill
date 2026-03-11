use dioxus::prelude::*;

pub const MAIN_CSS: Asset = asset!("/assets/main.css");
pub const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
pub const FONT_STYLES: Asset = asset!("/assets/fonts/fonts-stylesheet.css");

pub const FAVICON: Asset = asset!("/assets/favicon.png");

pub const COMMIT_MONO_FONT: Asset = asset!("/assets/fonts/commitmono_regular-webfont.woff2");

pub const BUNGEE_HAIRLINE_FONT: Asset =
    asset!("/assets/fonts/bungeehairline-regular-webfont.woff2");

pub const MARKO_ONE_FONT: Asset = asset!("/assets/fonts/markoone-regular-webfont.woff2");

pub fn extra_css_styles() -> Element {
    rsx! {
        document::Style {
            r#"
                @font-face {{
                    font-family: 'commitmonofont';
                    src: url('{COMMIT_MONO_FONT}') format('woff2');
                    font-weight: normal;
                    font-style: normal;

                }}
                
                @font-face {{
                    font-family: 'bungeehairlinefont';
                    src: url('{BUNGEE_HAIRLINE_FONT}') format('woff2');
                    font-weight: normal;
                    font-style: normal;

                }}

                @font-face {{
                    font-family: 'markoonefont';
                    src: url('{MARKO_ONE_FONT}') format('woff2');
                    font-weight: normal;
                    font-style: normal;

                }}

                body {{
                    background-color: #000;
                }}
        "#
        }
    }
}
