// Helper function to convert sRGB to linear color space
fn srgb_to_linear(srgb: [f32; 4]) -> [f32; 4] {
    fn component(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    [
        component(srgb[0]),
        component(srgb[1]),
        component(srgb[2]),
        srgb[3], // Alpha is linear
    ]
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: Colors,
    pub font: FontTheme,
    pub metrics: Metrics,
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub background: [f32; 4],
    pub surface: [f32; 4],
    pub widget_bg: [f32; 4], // Background for input widgets like checkbox, slider track
    pub primary: [f32; 4],
    pub text: [f32; 4],
    pub text_dim: [f32; 4],
    pub border: [f32; 4],
    pub error: [f32; 4],
    pub success: [f32; 4],
    pub hover_overlay: [f32; 4],
    pub press_overlay: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct FontTheme {
    pub size_small: f32,
    pub size_body: f32,
    pub size_heading: f32,
}

#[derive(Debug, Clone)]
pub struct Metrics {
    pub spacing: f32,
    pub padding: f32,
    pub radius: f32,
    pub border_width: f32,
    pub scrollbar_width: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            colors: Colors {
                background: srgb_to_linear([0.1, 0.1, 0.1, 1.0]),
                surface: srgb_to_linear([0.2, 0.2, 0.2, 1.0]),
                widget_bg: srgb_to_linear([0.15, 0.15, 0.15, 1.0]), // Darker for contrast
                primary: srgb_to_linear([0.2, 0.6, 1.0, 1.0]),
                text: srgb_to_linear([0.9, 0.9, 0.9, 1.0]),
                text_dim: srgb_to_linear([0.6, 0.6, 0.6, 1.0]),
                border: srgb_to_linear([0.3, 0.3, 0.3, 1.0]),
                error: srgb_to_linear([0.8, 0.2, 0.2, 1.0]),
                success: srgb_to_linear([0.2, 0.8, 0.2, 1.0]),
                hover_overlay: srgb_to_linear([1.0, 1.0, 1.0, 0.1]),
                press_overlay: srgb_to_linear([0.0, 0.0, 0.0, 0.2]),
            },
            font: FontTheme {
                size_small: 14.0,
                size_body: 18.0,
                size_heading: 24.0,
            },
            metrics: Metrics {
                spacing: 8.0,
                padding: 12.0,
                radius: 4.0,
                border_width: 1.0,
                scrollbar_width: 10.0,
            },
        }
    }

    pub fn light() -> Self {
        Self {
            colors: Colors {
                background: srgb_to_linear([0.95, 0.95, 0.95, 1.0]),
                surface: srgb_to_linear([1.0, 1.0, 1.0, 1.0]),
                widget_bg: srgb_to_linear([0.92, 0.92, 0.92, 1.0]), // Slightly darker for contrast
                primary: srgb_to_linear([0.2, 0.6, 1.0, 1.0]),
                text: srgb_to_linear([0.1, 0.1, 0.1, 1.0]),
                text_dim: srgb_to_linear([0.4, 0.4, 0.4, 1.0]),
                border: srgb_to_linear([0.8, 0.8, 0.8, 1.0]),
                error: srgb_to_linear([0.8, 0.2, 0.2, 1.0]),
                success: srgb_to_linear([0.2, 0.8, 0.2, 1.0]),
                hover_overlay: srgb_to_linear([0.0, 0.0, 0.0, 0.05]),
                press_overlay: srgb_to_linear([0.0, 0.0, 0.0, 0.1]),
            },
            font: FontTheme {
                size_small: 14.0,
                size_body: 18.0,
                size_heading: 24.0,
            },
            metrics: Metrics {
                spacing: 8.0,
                padding: 12.0,
                radius: 4.0,
                border_width: 1.0,
                scrollbar_width: 10.0,
            },
        }
    }
}
