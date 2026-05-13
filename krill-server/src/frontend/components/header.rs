use dioxus::prelude::*;
use krill_common::OrganizationInfo;
use wasm_toolkit::Breakpoints;

use crate::{
    OrgCacheOps, Translations, TranslationsMemInfo, BROWSER_MEASUREMENTS, NOTIFICATION_MANAGER,
};

#[component]
pub fn Header() -> Element {
    let translations_info = consume_context::<Signal<TranslationsMemInfo>>();
    let mut org_info = use_signal(|| OrganizationInfo::default());
    let mut active = use_signal(|| ActiveItem::default());

    use_effect(move || {
        spawn(async move {
            let fetched_org_info = match OrgCacheOps::get_org_info() {
                Err(error) => {
                    NOTIFICATION_MANAGER.send_final_error(error).await;

                    return;
                }
                Ok(value) => value,
            };

            org_info.set(fetched_org_info);
        });
    });

    let show_logo = if BROWSER_MEASUREMENTS.read().visual_viewport.breakpoints()
        > Breakpoints::Medium
    {
        rsx! {
            div {
                class: "krill-bg-surface-container krill-backdrop-blur-glass krill-shadow-glass",
                class: "flex p-1",
                img {
                    class: "flex w-[clamp(25px,3vw,30px)]",
                    src: org_info.read().logo_to_css_base64().to_string(),
                }
            }
        }
    } else {
        rsx! {}
    };

    let height;
    let content_min_height;
    let position;
    let justify_class;
    let container_flex;
    let content_margin;

    if BROWSER_MEASUREMENTS.read().visual_viewport.breakpoints() <= Breakpoints::Medium {
        justify_class = "justify-end";
        position = "fixed bottom-0 dark:bg-[#000] krill-bg-dots light:bg-[#FFF]";
        container_flex = "flex-col-reverse";

        height = "h-[".to_string()
            + BROWSER_MEASUREMENTS
                .read()
                .visual_viewport
                .height
                .to_string()
                .as_str()
            + "px]";

        content_min_height = "h-[".to_string()
            + (BROWSER_MEASUREMENTS.read().visual_viewport.height * 0.9)
                .to_string()
                .as_str()
            + "px]";

        content_margin = "mb-[60px]";
    } else {
        justify_class = "justify-start";
        position = "sticky top-0";
        container_flex = "flex-col";
        height = "min-h-screen".to_string();
        content_min_height = "min-h-[90dvh]".to_string();
        content_margin = ""
    };

    rsx! {
        div { class: "{justify_class} {container_flex} flex {height} items-center w-full px-2",
            nav { class: "{position} flex w-full z-20 start-0 min-h-[60px] items-center justify-between md:justify-center lg:justify-between
            gap-1 transition-all duration-300 ease-in flex dark:text-white light:text-black",
                {show_logo}

                div { class: "flex w-full items-center justify-between lg:max-w-[90dvh] p-2",
                    {
                        nav_op_item(
                            ActiveItem::Inbox,
                            active,
                            move || { active.set(ActiveItem::Inbox) },
                            &translations_info.read().translations,
                            notifications_bell_icon,
                        )
                    }
                    {
                        nav_op_item(
                            ActiveItem::Home,
                            active,
                            move || {
                                active.set(ActiveItem::Home);
                            },
                            &translations_info.read().translations,
                            home_icon,
                        )
                    }
                    {
                        nav_op_item(
                            ActiveItem::Events,
                            active,
                            move || { active.set(ActiveItem::Events) },
                            &translations_info.read().translations,
                            calendar_icon,
                        )
                    }
                    {
                        nav_op_item(
                            ActiveItem::Members,
                            active,
                            move || { active.set(ActiveItem::Members) },
                            &translations_info.read().translations,
                            members_icon,
                        )
                    }
                    {
                        nav_op_item(
                            ActiveItem::Settings,
                            active,
                            move || { active.set(ActiveItem::Settings) },
                            &translations_info.read().translations,
                            settings_icon,
                        )
                    }
                }

            }

            div { class: "flex flex-col {content_min_height} overflow-y-auto",
                {lorem()}
                div { class: "{content_margin}" }
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ActiveItem {
    Home,
    #[default]
    Inbox,
    Events,
    Members,
    Settings,
}

impl ActiveItem {
    fn to_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Inbox => "inbox",
            Self::Events => "events",
            Self::Members => "members",
            Self::Settings => "settings",
        }
    }
}

fn nav_op_item<F>(
    variant: ActiveItem,
    active: Signal<ActiveItem>,
    mut callback: F,
    translations: &Translations,
    icon: fn() -> Result<VNode, RenderError>,
) -> Element
where
    F: FnMut() + 'static,
{
    let is_mobile =
        BROWSER_MEASUREMENTS.read().visual_viewport.breakpoints() <= Breakpoints::Medium;

    rsx! {
        div {
            onclick: move |_| { callback() },
            class: if *active.read() == variant { "krill-bg-surface-container krill-backdrop-blur-glass krill-shadow-glass  dark:text-[var(--primary-color)] light:text-black " } else { "dark:text-white light:text-black hover:text-[var(--primary-color)] " },
            class: "items-center justify-center gap-1 cursor-pointer hover:text-[var(--primary-color)] ",
            class: if is_mobile { "flex-col flex-1 w-full " } else { "flex min-w-[100px] px-2 py-1 " },
            span {
                class: "flex items-center justify-center",
                class: if is_mobile { " w-[25px] " } else { " w-[15px] " },
                {icon()}
            }

            a {
                class: "flex items-center justify-center text-center  font-[subheadingfont]",
                class: if is_mobile { "text-[0.6rem]" } else { "text-sm" },
                {translations.translate(variant.to_str())}
            }
        }
    }
}

fn home_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 100 100", xmlns: "http://www.w3.org/2000/svg",
            path {
                d: "m50 294.07-44.99 44.99h11.247v37.491c0 4.154 3.3442 7.4983 7.4983 7.4983h16.871v-26.244h18.746v26.244h16.871c4.154 0 7.4983-3.3442 7.4983-7.4983v-37.491h11.247l-11.247-11.247v-11.247c0-2.077-1.6721-3.7491-3.7491-3.7491s-3.7491 1.6721-3.7491 3.7491v3.7491z",
                fill: "currentColor",
                transform: "translate(0 -289.06)",
            }
        }
    }
}

fn notifications_bell_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 3 3", xmlns: "http://www.w3.org/2000/svg",
            path {
                d: "m1.26 2.46h.48a.24.24 0 0 1 -.48 0zm.96-.48v-.72a.72.72 0 0 0 -.6-.7092v-.1308a.12.12 0 0 0 -.24 0v.1308a.72.72 0 0 0 -.6.7092v.72l-.24.24h1.92z",
                fill: "currentColor",
                stroke_width: ".12",
            }
        }
    }
}

fn calendar_icon() -> Element {
    rsx! {
        svg {
            fill: "none",
            view_box: "0 0 3 3",
            xmlns: "http://www.w3.org/2000/svg",
            g {
                fill: "currentColor",
                transform: "matrix(.11566 0 0 .11566 .11203 .15542)",
                path { d: "m16.75 3.56v-1.56c0-.41-.34-.75-.75-.75s-.75.34-.75.75v1.5h-6.5v-1.5c0-.41-.34-.75-.75-.75s-.75.34-.75.75v1.56c-2.7.25-4.01 1.86-4.21 4.25-.02.29.22.53.5.53h16.92c.29 0 .53-.25.5-.53-.2-2.39-1.51-4-4.21-4.25z" }
                path {
                    d: "m20 9.8398h-16c-.55 0-1 .44996-1 .99996v6.16c0 3 1.5 5 5 5h8c3.5 0 5-2 5-5v-6.16c0-.55-.45-.99996-1-.99996zm-10.79 8.37c-.05.04-.1.09-.15.12-.06.04-.12.07-.18.09-.06.03-.12.05-.18.06-.07.01-.13.02-.2.02-.13 0-.26-.03-.38-.08-.13-.05-.23-.12-.33-.21-.18-.19-.29-.45-.29-.71s.11-.52.29-.71c.1-.09.2-.16.33-.21.18-.08.38-.1.58-.06.06.01.12.03.18.06.06.02.12.05.18.09l.15.12c.18.19.29.45.29.71s-.11.52-.29.71zm0-3.5c-.19.18-.45.29-.71.29s-.52-.11-.71-.29c-.18-.19-.29-.45-.29-.71s.11-.52.29-.71c.28-.28.72-.37 1.09-.21.13.05.24.12.33.21.18.19.29.45.29.71s-.11.52-.29.71zm3.5 3.5c-.19.18-.45.29-.71.29s-.52-.11-.71-.29c-.18-.19-.29-.45-.29-.71s.11-.52.29-.71c.37-.37 1.05-.37 1.42 0 .18.19.29.45.29.71s-.11.52-.29.71zm0-3.5-.15.12c-.06.04-.12.07-.18.09-.06.03-.12.05-.18.06-.07.01-.13.02-.2.02-.26 0-.52-.11-.71-.29-.18-.19-.29-.45-.29-.71s.11-.52.29-.71c.09-.09.2-.16.33-.21.37-.16.81-.07 1.09.21.18.19.29.45.29.71s-.11.52-.29.71zm3.5 3.5c-.19.18-.45.29-.71.29s-.52-.11-.71-.29c-.18-.19-.29-.45-.29-.71s.11-.52.29-.71c.37-.37 1.05-.37 1.42 0 .18.19.29.45.29.71s-.11.52-.29.71zm0-3.5-.15.12c-.06.04-.12.07-.18.09-.06.03-.12.05-.18.06-.07.01-.14.02-.2.02-.26 0-.52-.11-.71-.29-.18-.19-.29-.45-.29-.71s.11-.52.29-.71c.1-.09.2-.16.33-.21.18-.08.38-.1.58-.06.06.01.12.03.18.06.06.02.12.05.18.09l.15.12c.18.19.29.45.29.71s-.11.52-.29.71z",
                }
            }
        }
    }
}

fn members_icon() -> Element {
    rsx! {
        svg {
            fill: "none",
            view_box: "0 0 3 3",
            xmlns: "http://www.w3.org/2000/svg",
            path {
                d: "m2.0001 2.4111c.066554.034432.15225.056155.26135.056155.21732 0 .34198-.085722.40759-.1865.031079-.047738.046366-.095007.053975-.13007.00383-.017667.00581-.032662.00685-.043705.0005173-.00555.000776-.010141.0009238-.013634.0000821-.00175.000129-.00322.0001527-.00442l.0000232-.00156v-.11836c0-.023318-.00926-.045686-.025746-.062169-.016493-.016495-.038851-.025756-.062181-.025756h-.57666c.012908.027222.019789.057222.019789.087926v.17727l-.0000115.00156-.0000469.00217c-.0000351.00204-.0001057.00438-.0002228.00703-.0002343.0053-.0006283.011829-.00133.019414-.0014.015088-.00404.034819-.009.057703-.00981.045182-.029578.10662-.070294.16918-.0017.0026-.00344.00519-.00517.00776zm-1.0665-.53002c-.012661.026647-.019745.05646-.019745.087926l.0000011.1768.00000444.00108.0000388.00251.00001183.0005987c.00004213.00203.00011382.00438.00022875.00702.00022838.00529.00062823.011806.00133.019379.001404.015076.004017.034784.008965.057632.009763.045135.029453.10653.070037.16906.0017.00264.00344.00526.00523.00788-.066758.034514-.15275.056284-.26228.056284-.21685 0-.34122-.085816-.40663-.1866-.030979-.047738-.046213-.094983-.053793-.13002-.003817-.017644-.005791-.032638-.006814-.04367-.00051737-.00555-.00077605-.010129-.00096082-.013623-.00009867-.00227-.0001748-.00686-.0001748-.00686v-.11747c0-.048559.039366-.087926.087926-.087926zm.097491.087926c0-.048558.039366-.087926.087926-.087926h.76202c.023318 0 .045686.00926.062169.025756.016497.016483.025757.038852.025757.062169v.17585.0005041l-.0000118.0006208-.0000233.00156c-.0000236.0012-.0000706.00267-.0001408.00442-.0001526.00349-.0004434.00809-.0009608.013634-.00103.011043-.00299.026038-.00684.043705-.00761.035065-.022897.082334-.053975.13007-.065592.10078-.19026.1865-.40758.1865-.21685 0-.34122-.085816-.40663-.1866-.030979-.047738-.046213-.094983-.053793-.13002-.00382-.017644-.00579-.032638-.00681-.04367-.0005173-.00555-.000776-.010129-.0009608-.013623-.000075-.00175-.0001186-.00322-.0001441-.00441l-.0000248-.00157-.0000048-.0006097-.0000011-.0002697zm.93788-.5469c0-.16188.13122-.29309.29309-.29309s.29309.13121.29309.29309c0 .16187-.13122.29309-.29309.29309s-.29309-.13122-.29309-.29309zm-.38101-.65481c0-.16187.13122-.29309.29309-.29309s.29309.13122.29309.29309-.13122.29309-.29309.29309-.29309-.13122-.29309-.29309zm-.76202 0c0-.16187.13122-.29309.29309-.29309s.29309.13122.29309.29309-.13122.29309-.29309.29309-.29309-.13122-.29309-.29309zm-.38101.65481c0-.16188.13122-.29309.29309-.29309s.29309.13121.29309.29309c0 .16187-.13122.29309-.29309.29309s-.29309-.13122-.29309-.29309zm.76202-.00996c0-.16187.13122-.29309.29309-.29309s.29309.13122.29309.29309-.13122.29309-.29309.29309-.29309-.13122-.29309-.29309z",
                fill: "currentColor",
                stroke_width: ".11723",
            }
        }
    }
}

fn settings_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 100 100", xmlns: "http://www.w3.org/2000/svg",
            g {
                fill: "currentColor",
                transform: "matrix(.20718 0 0 .20718 11.639 11.639)",
                path { d: "m154.66 134.16h-67.667c-8.284 0-15 6.716-15 15s6.716 15 15 15h67.667c8.284 0 15-6.716 15-15s-6.716-15-15-15z" }
                path { d: "m154.66 224.16h-67.667c-8.284 0-15 6.716-15 15s6.716 15 15 15h67.667c8.284 0 15-6.716 15-15s-6.716-15-15-15z" }
                path { d: "m271.99 152.88c-13.035 0-23.636 10.602-23.636 23.637v15.635h47.272v-15.635c0-13.035-10.601-23.637-23.636-23.637z" }
                path {
                    d: "m343.36 177.65v-1.131c0-31.924-21.069-59.017-50.031-68.109v-34.703c0-3.934-1.56-7.706-4.344-10.484l-58.876-58.88c-2.784-2.778-6.55-4.339-10.484-4.339h-177.18c-16.053 0-29.112 13.06-29.112 29.112v312.1c0 16.054 13.059 29.113 29.111 29.113h221.78c16.052 0 29.111-13.06 29.111-29.113v-11.508h22.758c22.557 0 40.908-18.352 40.908-40.91v-80.727c.001-11.914-5.237-22.854-13.634-30.416zm-81.644 160.26h-216.77v-305.49h154.09v43.577c0 10.554 8.554 19.106 19.105 19.106h43.58v10.8c-34.496 5-61.086 34.757-61.086 70.619v1.131c-8.395 7.563-13.635 18.502-13.635 30.416v80.727c0 22.558 18.352 40.91 40.908 40.91h33.813v8.206zm70.279-118.18h-27.044c-6.404 0-11.592 5.191-11.592 11.59 0 6.4 5.188 11.59 11.592 11.59h27.044v16.561h-27.044c-6.404 0-11.592 5.189-11.592 11.594 0 6.402 5.188 11.592 11.592 11.592h27.044v6.133c0 8.789-7.121 15.91-15.908 15.91h-88.184c-8.787 0-15.908-7.121-15.908-15.91v-80.727c0-8.016 5.925-14.643 13.635-15.748v-15.799c0-25.57 20.801-46.369 46.365-46.369s46.365 20.799 46.365 46.369v15.799c7.71 1.105 13.635 7.732 13.635 15.748z",
                }
            }
        }
    }
}

fn lorem() -> &'static str {
    r#""" Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla a ex lacus. Integer ultricies, enim vitae sollicitudin aliquam, metus arcu faucibus neque, eget gravida nisl lacus ultrices nibh. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Quisque cursus odio at aliquam pretium. Vivamus vel nulla auctor, laoreet diam in, porta justo. Ut cursus augue at velit porttitor pharetra. Nullam eleifend massa et urna ultrices, a venenatis augue rutrum. Phasellus tempus lacinia ex, sit amet facilisis massa imperdiet eu. Proin tristique pellentesque rutrum. Sed pellentesque ligula vitae lobortis accumsan. Morbi fringilla, ex eu ornare dignissim, sapien arcu aliquet velit, vitae efficitur libero eros pharetra nunc. Praesent id dui in lectus molestie ultrices. Integer imperdiet, velit at fermentum aliquet, nunc augue consequat est, in tincidunt augue neque et massa. Phasellus venenatis nulla ex, non maximus neque ultricies a.

Nam sagittis, tellus id accumsan tincidunt, dolor tortor dignissim magna, id pulvinar mauris mauris sed massa. Sed congue tincidunt sapien sed sollicitudin. Nunc ut sapien tincidunt, pretium ex sed, pretium ante. Ut sollicitudin felis lorem. Integer vel ante nec libero euismod tempus vitae et nunc. Nunc facilisis, tortor sit amet sollicitudin elementum, diam dolor ultricies arcu, eu fermentum nisl lectus quis purus. Donec sed venenatis odio. Aliquam nunc quam, cursus malesuada lectus eu, tincidunt interdum nulla.

Nam sit amet leo magna. Nunc non eros faucibus, mattis sem eget, rutrum elit. Nulla facilisi. Cras tincidunt, risus eu consectetur vestibulum, turpis tortor rhoncus ante, eu gravida nunc lectus et justo. Nullam laoreet enim eget iaculis tempor. In dui enim, gravida vitae urna convallis, dictum vulputate ipsum. Suspendisse potenti. Aliquam erat volutpat. Vestibulum et enim et orci sodales tempor. Nulla id orci sed tortor viverra porttitor. Etiam vel eleifend justo. Nullam nisi felis, consectetur volutpat velit ac, tincidunt dictum dolor. Sed ullamcorper lorem ac lorem luctus faucibus. Sed sem tortor, euismod et consequat gravida, dapibus ut erat.

Phasellus aliquam sapien sapien, ac pulvinar tellus egestas tincidunt. Curabitur vulputate tempor nisl eu lacinia. Nunc suscipit et augue sit amet interdum. Sed mi nisl, porttitor eget leo non, dapibus ultrices quam. Nullam sit amet est ut velit iaculis interdum at a turpis. Quisque pretium accumsan augue et lobortis. In a magna orci.

Nam tristique semper erat, at hendrerit mauris sollicitudin eget. Donec venenatis tincidunt tempus. Integer pharetra odio eget orci tempus faucibus. Phasellus at quam a dui tristique cursus. Nam finibus sapien ac bibendum eleifend. Cras commodo sagittis posuere. Vivamus faucibus diam sit amet efficitur feugiat. Cras ac dolor eget diam lacinia aliquam non a eros. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Aliquam dignissim congue mattis. Curabitur tincidunt eros vitae tortor tincidunt fringilla. Integer ut neque leo. Phasellus libero ligula, ultrices a magna id, laoreet euismod justo. Morbi tristique diam nec libero dapibus tincidunt. Praesent quis eros eget sapien aliquam luctus. Nunc convallis nulla sem, non congue leo feugiat non.     
    """#
}
