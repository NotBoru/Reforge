use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Instant;

pub fn ease_out_cubic(t: f32) -> f32 {
    let c = t.clamp(0.0, 1.0);
    1.0 - (1.0 - c).powi(3)
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let f = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * f,
        g: a.g + (b.g - a.g) * f,
        b: a.b + (b.b - a.b) * f,
        a: a.a + (b.a - a.a) * f,
    }
}

#[derive(Debug, Clone)]
pub struct MotionState {
    pub reduced_motion: bool,
    modal_started: Option<Instant>,
    pub modal_progress: f32,
    spinner_origin: Instant,
    pub spinner_phase: f32,
    last_tick: Instant,
    hovers: HashMap<String, (f32, bool)>,
}

impl MotionState {
    pub fn new(reduced_motion: bool) -> Self {
        let now = Instant::now();
        Self {
            reduced_motion,
            modal_started: None,
            modal_progress: 1.0,
            spinner_origin: now,
            spinner_phase: 0.0,
            last_tick: now,
            hovers: HashMap::new(),
        }
    }

    pub fn set_hover(&mut self, id: String, hovered: bool) {
        if self.reduced_motion {
            if hovered {
                self.hovers.insert(id, (1.0, true));
            } else {
                self.hovers.remove(&id);
            }
            return;
        }
        let now = Instant::now();
        if !self.has_active_hovers() {
            self.last_tick = now;
        }
        let entry = self.hovers.entry(id).or_insert((0.0, hovered));
        entry.1 = hovered;
    }

    pub fn hover_level(&self, id: &str) -> f32 {
        self.hovers.get(id).map(|(v, _)| *v).unwrap_or(0.0)
    }

    pub fn clear_hovers(&mut self) {
        self.hovers.clear();
    }

    fn has_active_hovers(&self) -> bool {
        self.hovers
            .values()
            .any(|&(val, target)| (target && val < 0.995) || (!target && val > 0.005))
    }

    pub fn trigger_tab_transition(&mut self) {
        self.hovers.clear();
    }

    pub fn trigger_modal_enter(&mut self) {
        if self.reduced_motion {
            self.modal_started = None;
            self.modal_progress = 1.0;
        } else {
            self.modal_started = Some(Instant::now());
            self.modal_progress = 0.0;
        }
    }

    pub fn is_animating(&self, has_live_spinner: bool) -> bool {
        if has_live_spinner {
            return true;
        }
        if self.reduced_motion {
            return false;
        }
        self.modal_started.is_some() || self.has_active_hovers()
    }

    pub fn tick(&mut self, now: Instant) {
        let dt = now
            .saturating_duration_since(self.last_tick)
            .as_secs_f32()
            .clamp(0.001, 0.05);
        self.last_tick = now;

        if let Some(start) = self.modal_started {
            let t = (now.saturating_duration_since(start).as_secs_f32() / 0.11).min(1.0);
            self.modal_progress = ease_out_cubic(t);
            if t >= 1.0 {
                self.modal_started = None;
            }
        }

        // Exponential damping gives crisp native desktop response without linear drift.
        let alpha_in = 1.0 - (-dt / 0.022).exp();
        let alpha_out = 1.0 - (-dt / 0.042).exp();
        for (val, target) in self.hovers.values_mut() {
            if *target {
                *val += (1.0 - *val) * alpha_in;
                if *val > 0.995 {
                    *val = 1.0;
                }
            } else {
                *val *= 1.0 - alpha_out;
                if *val < 0.005 {
                    *val = 0.0;
                }
            }
        }
        self.hovers.retain(|_, &mut (val, target)| target || val > 0.005);

        let secs = now.saturating_duration_since(self.spinner_origin).as_secs_f32();
        self.spinner_phase = (secs * 1.25) % 1.0;
    }

    pub fn tab_offset_y(&self) -> f32 {
        0.0
    }

    pub fn modal_alpha(&self) -> f32 {
        if self.reduced_motion {
            1.0
        } else {
            self.modal_progress
        }
    }
}

struct SpinnerProgram {
    phase: f32,
    color: Color,
    track: Color,
}

impl<M> canvas::Program<M> for SpinnerProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width.min(bounds.height) / 2.0 - 1.5).max(2.0);

        frame.stroke(
            &Path::circle(center, radius),
            Stroke::default().with_width(1.6).with_color(self.track),
        );

        let start = self.phase * 2.0 * PI;
        let sweep = 1.25 * PI;
        let arc = Path::new(|b| {
            let steps = 24;
            for i in 0..=steps {
                let a = start + sweep * (i as f32 / steps as f32);
                let pt = Point::new(center.x + radius * a.cos(), center.y + radius * a.sin());
                if i == 0 {
                    b.move_to(pt);
                } else {
                    b.line_to(pt);
                }
            }
        });
        frame.stroke(
            &arc,
            Stroke::default()
                .with_width(1.6)
                .with_color(self.color)
                .with_line_cap(canvas::LineCap::Round),
        );

        vec![frame.into_geometry()]
    }
}

pub fn spinner<'a, M: 'a>(phase: f32, size: f32, color: Color, track: Color) -> Element<'a, M> {
    Canvas::new(SpinnerProgram {
        phase,
        color,
        track,
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

#[derive(Debug, Clone)]
pub struct StorageSegment {
    pub bytes: u64,
    pub color: Color,
}

struct StorageBarProgram {
    segments: Vec<StorageSegment>,
    total: u64,
    track: Color,
}

impl<M> canvas::Program<M> for StorageBarProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let h = bounds.height;
        let w = bounds.width;
        let r = h / 2.0;

        frame.fill(
            &Path::rounded_rectangle(Point::ORIGIN, Size::new(w, h), r.into()),
            self.track,
        );

        if self.total == 0 || w <= 2.0 {
            return vec![frame.into_geometry()];
        }

        let gap = 2.0_f32;
        let mut x = 0.0_f32;
        let non_zero: Vec<&StorageSegment> = self.segments.iter().filter(|s| s.bytes > 0).collect();
        for (i, seg) in non_zero.iter().enumerate() {
            let ratio = seg.bytes as f32 / self.total as f32;
            let raw_w = (ratio * w).max(4.0);
            let seg_w = if i + 1 == non_zero.len() {
                (w - x).max(0.0)
            } else {
                (raw_w - gap).max(3.0).min((w - x).max(0.0))
            };
            if seg_w <= 0.5 {
                continue;
            }
            let rect = Path::rounded_rectangle(
                Point::new(x, 0.0),
                Size::new(seg_w, h),
                if non_zero.len() == 1 { r } else { 2.5 }.into(),
            );
            frame.fill(&rect, seg.color);
            x += seg_w + gap;
            if x >= w {
                break;
            }
        }

        vec![frame.into_geometry()]
    }
}

pub fn storage_breakdown_bar<'a, M: 'a>(
    segments: Vec<StorageSegment>,
    total: u64,
    track: Color,
) -> Element<'a, M> {
    Canvas::new(StorageBarProgram {
        segments,
        total,
        track,
    })
    .width(Length::Fill)
    .height(Length::Fixed(7.0))
    .into()
}
