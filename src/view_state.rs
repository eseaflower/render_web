#[derive(Debug)]
pub struct ViewState {
    pub zoom: Zoom,
    pub pos: (f32, f32),
    pub anchor: Option<(f32, f32)>,
}

#[derive(Debug)]
pub enum Zoom {
    Fit(f32),
    Pixel(f32),
}

impl ViewState {
    pub fn new() -> Self {
        ViewState {
            zoom: Zoom::Fit(1.0),
            pos: (0.0, 0.0),
            anchor: None,
        }
    }

    pub fn set_zoom_mode(&mut self, z: Zoom) {
        self.zoom = z;
    }

    pub fn update_magnification(&mut self, mag: f32) {
        self.zoom = match self.zoom {
            Zoom::Fit(current) => Zoom::Fit(current * mag),
            Zoom::Pixel(current) => Zoom::Pixel(current * mag),
        }
    }

    pub fn set_position(&mut self, pos: (f32, f32)) {
        if self.anchor == None {
            self.set_anchor(pos);
        }
        let disp = (
            pos.0 - self.anchor.unwrap().0,
            pos.1 - self.anchor.unwrap().1,
        );
        self.pos = (self.pos.0 + disp.0, self.pos.1 + disp.1);
        self.set_anchor(pos);
    }

    pub fn set_zoom(&mut self, pos: (f32, f32)) {
        if self.anchor == None {
            self.set_anchor(pos);
        }
        let disp = (
            pos.0 - self.anchor.unwrap().0,
            pos.1 - self.anchor.unwrap().1,
        );

        // Add to the zoom factor
        let factor = 1.0_f32 + (-disp.1 / 256.0_f32).max(-0.5).min(0.5);

        self.update_magnification(factor);

        self.set_anchor(pos);
    }

    pub fn get_displacement(&self) -> (f32, f32) {
        self.pos
    }

    pub fn set_anchor(&mut self, a: (f32, f32)) {
        self.anchor = Some(a);
    }
    pub fn clear_anchor(&mut self) {
        self.anchor = None;
    }
}
