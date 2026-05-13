use async_channel::Sender;
use wasm_bindgen::prelude::*;
use web_sys::{Event, VisualViewport};

use crate::{WasmToolkitError, WasmToolkitResult, WasmWindow};

impl WasmWindow {
    pub fn get_visual_viewport(&self) -> WasmToolkitResult<VisualViewport> {
        self.inner()
            .visual_viewport()
            .ok_or(WasmToolkitError::UnableToGetVisulaViewPort)
    }

    pub fn get_inner_height(&self) -> WasmToolkitResult<f64> {
        self.inner()
            .inner_height()
            .map(|value| value.as_f64())
            .transpose()
            .ok_or(WasmToolkitError::JsErrorStatic {
                name: "window.inner_height",
                message: "`window.innerHeight` value is not an f64",
            })?
            .or(Err(WasmToolkitError::UnableToWindowInnerHeight))
    }

    pub fn get_inner_width(&self) -> WasmToolkitResult<f64> {
        self.inner()
            .inner_width()
            .map(|value| value.as_f64())
            .transpose()
            .ok_or(WasmToolkitError::JsErrorStatic {
                name: "window.inner_widtheight",
                message: "`window.innerWidth` value is not an f64",
            })?
            .or(Err(WasmToolkitError::UnableToWindowInnerWidth))
    }

    pub fn get_outer_height(&self) -> WasmToolkitResult<f64> {
        self.inner()
            .outer_height()
            .map(|value| value.as_f64())
            .transpose()
            .ok_or(WasmToolkitError::JsErrorStatic {
                name: "window.outer_height",
                message: "`window.outerHeight` value is not an f64",
            })?
            .or(Err(WasmToolkitError::UnableToWindowOuterHeight))
    }

    pub fn get_outer_width(&self) -> WasmToolkitResult<f64> {
        self.inner()
            .outer_width()
            .map(|value| value.as_f64())
            .transpose()
            .ok_or(WasmToolkitError::JsErrorStatic {
                name: "window.outer_width",
                message: "`window.outerWidth` value is not an f64",
            })?
            .or(Err(WasmToolkitError::UnableToWindowOuterWidth))
    }

    pub fn get_browser_measurements(&self) -> WasmToolkitResult<BrowserMeasurements> {
        let visual_viewport = BrowserVisualViewport::new(self.get_visual_viewport()?);
        let inner_height = self.get_inner_height()?;
        let inner_width = self.get_inner_width()?;
        let outer_height = self.get_outer_height()?;
        let outer_width = self.get_outer_width()?;

        Ok(BrowserMeasurements {
            visual_viewport,
            inner_height,
            inner_width,
            outer_height,
            outer_width,
        })
    }

    pub fn browser_measurements_listener(
        &self,
        sender: Sender<Result<BrowserMeasurements, Vec<WasmToolkitError>>>,
    ) -> WasmToolkitResult<()> {
        let visual_viewport = self.get_visual_viewport()?;

        let handler = Closure::wrap(Box::new(move |_event: Event| {
            let mut errors = Vec::<WasmToolkitError>::default();

            let new_visual_viewport = self
                .get_visual_viewport()
                .map(BrowserVisualViewport::new)
                .map_err(|error| {
                    errors.push(error);
                });

            let inner_height = self.get_inner_height().map_err(|error| {
                errors.push(error);
            });
            let inner_width = self.get_inner_width().map_err(|error| {
                errors.push(error);
            });
            let outer_height = self.get_outer_height().map_err(|error| {
                errors.push(error);
            });
            let outer_width = self.get_outer_width().map_err(|error| {
                errors.push(error);
            });

            let cloned_sender = sender.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let outcome = if errors.is_empty() {
                    Ok(BrowserMeasurements {
                        visual_viewport: new_visual_viewport.unwrap(),
                        inner_height: inner_height.unwrap(),
                        inner_width: inner_width.unwrap(),
                        outer_height: outer_height.unwrap(),
                        outer_width: outer_width.unwrap(),
                    })
                } else {
                    Err(errors)
                };

                if cloned_sender.send(outcome).await.is_err() {
                    tracing::error!("Send VisualViewport via channel error. Channel closed!",)
                }
            });
        }) as Box<dyn FnMut(_)>);

        if let Err(error) = visual_viewport
            .add_event_listener_with_callback("resize", handler.as_ref().unchecked_ref())
        {
            Err(WasmToolkitError::parse_js_error(
                error,
                "Unable to add event listner for `visualViewport.resize()`",
            ))
        } else {
            handler.forget();

            Ok(())
        }
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Copy)]
pub struct BrowserMeasurements {
    pub visual_viewport: BrowserVisualViewport,
    pub inner_height: f64,
    pub inner_width: f64,
    pub outer_height: f64,
    pub outer_width: f64,
}

#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Copy)]
pub struct BrowserVisualViewport {
    pub height: f64,
    pub width: f64,
    pub offset_top: f64,
    pub offset_left: f64,
    pub page_left: f64,
    pub page_top: f64,
    pub scale: f64,
}

impl BrowserVisualViewport {
    pub fn new(value: VisualViewport) -> Self {
        Self {
            height: value.height(),
            width: value.width(),
            offset_top: value.offset_top(),
            offset_left: value.offset_left(),
            page_left: value.page_left(),
            page_top: value.page_top(),
            scale: value.scale(),
        }
    }

    pub fn breakpoints(&self) -> Breakpoints {
        Breakpoints::parse(self)
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Breakpoints {
    #[default]
    Small = 640,
    Medium = 768,
    Large = 1024,
    ExtraLarge = 1280,
    ExtraExtraLarge = 1536,
}

impl Breakpoints {
    #[inline]
    pub const fn parse(visual_viewport: &BrowserVisualViewport) -> Self {
        let width = visual_viewport.width;

        if width <= Self::Small.pixels() {
            Self::Small
        } else if width <= Self::Medium.pixels() {
            Self::Medium
        } else if width <= Self::Large.pixels() {
            Self::Large
        } else if width <= Self::ExtraLarge.pixels() {
            Self::ExtraLarge
        } else {
            Self::ExtraExtraLarge
        }
    }

    #[inline]
    pub const fn prefix(&self) -> &str {
        match self {
            Self::Small => "sm",
            Self::Medium => "md",
            Self::Large => "lg",
            Self::ExtraLarge => "xl",
            Self::ExtraExtraLarge => "2xl",
        }
    }

    #[inline]
    pub const fn pixels(&self) -> f64 {
        (*self as u16) as f64
    }

    #[inline]
    pub const fn rem(&self) -> u16 {
        match self {
            Self::Small => 40,
            Self::Medium => 48,
            Self::Large => 64,
            Self::ExtraLarge => 80,
            Self::ExtraExtraLarge => 96,
        }
    }

    #[inline]
    pub const fn device(&self) -> &str {
        match self {
            Self::Small => "mobile",
            Self::Medium => "tablet",
            Self::Large => "laptop",
            Self::ExtraLarge => "desktop",
            Self::ExtraExtraLarge => "wide desktop",
        }
    }
}
