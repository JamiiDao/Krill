use dioxus::prelude::*;

pub const MAIN_CSS: Asset = asset!("/assets/main.css");
pub const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
pub const FONT_STYLES: Asset = asset!("/assets/fonts/fonts-stylesheet.css");

pub const FAVICON: Asset = asset!("/assets/favicon.png");

pub const COMMIT_MONO_FONT: Asset = asset!("/assets/fonts/commitmono_regular-webfont.woff2");

pub const BUNGEE_HAIRLINE_FONT: Asset =
    asset!("/assets/fonts/bungeehairline-regular-webfont.woff2");

pub const MARKO_ONE_FONT: Asset = asset!("/assets/fonts/markoone-regular-webfont.woff2");

const _: Asset = asset!("/assets/icons", AssetOptions::folder());

pub fn extra_css_styles() -> Element {
    rsx! {
        document::Style {
            r#"
                @font-face {{
                    font-family: 'monospacefont';
                    src: url('{COMMIT_MONO_FONT}') format('woff2');
                    font-weight: normal;
                    font-style: normal;

                }}
                
                @font-face {{
                    font-family: 'headingfont';
                    src: url('{BUNGEE_HAIRLINE_FONT}') format('woff2');
                    font-weight: normal;
                    font-style: normal;

                }}

                @font-face {{
                    font-family: 'subheadingfont';
                    src: url('{MARKO_ONE_FONT}') format('woff2');
                    font-weight: normal;
                    font-style: normal;

                }}

                .bg-surface-container {{
                    background-color: rgba(var(--backgroundColor-surface-container, 0,0,0,0), 0.5);
                    border-radius: 50px;
                    border: 1px solid var(--borderColor-secondary, rgba(255, 255, 255, 0.2));
                    overflow: hidden;
                    position: relative;
                }}

                /* Backdrop Blur Effect */
                .backdrop-blur-glass {{
                    --tw-backdrop-blur: blur(40px);
                    -webkit-backdrop-filter: var(--tw-backdrop-blur);
                    backdrop-filter: var(--tw-backdrop-blur);
                }}

                /* Glass Shadow */
                .shadow-glass {{
                    --tw-shadow: 0 0 15px 0 rgba(0, 0, 0, 0.25);
                    box-shadow: var(--tw-shadow);
                }}

                /* Optional: Flex Layout Helpers */
                .flex-col {{
                    display: flex;
                    flex-direction: column;
                }}

                .flex-1 {{
                    flex: 1 1 0%;
                }}

                .w-full {{
                    width: 100%;
                }}

                .min-h-0 {{
                    min-height: 0;
                }}

                .pb-3 {{
                    padding-bottom: 0.75rem;
                }}

                .overflow-hidden {{
                    overflow: hidden;
                }}

                .rounded-2xl {{
                    border-radius: 1rem;
                }}

                .border-secondary {{
                    border-color: var(--borderColor-secondary, rgba(255, 255, 255, 0.2));
                }}

                body {{
                    background-color: #000;
                }}
            "#
        }
    }
}
