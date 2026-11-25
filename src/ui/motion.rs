use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use std::f32::consts::PI;
use std::time::Instant;

/// Smooth cubic-out deceleration curve (macOS / Linear standard for entering surfaces).
pub fn ease_out_cubic(t: f32) -> f32 {
    let c = t.clamp(0.0, 1.0);
    1.0 - (1.0 - c).powi(3)
}

/// Manages restrained, OS-aware UI transitions with zero idle frame overhead.
#[derive(Debug, Clone)]
pub struct MotionState {
    pub reduced_motion: bool,
    tab_started: Option<Instant>,
    pub tab_progress: f32,
    modal_started: Option<Instant>,
    pub modal_progress: f32,
    spinner_origin: Instant,
    pub spinner_phase: f32,
}

impl MotionState {
    pub fn new(reduced_motion: bool) -> Self {
        Self {
            reduced_motion,
            tab_started: None,
            tab_progress: 1.0,
            modal_started: None,
            modal_progress: 1.0,
            spinner_origin: Instant::now(),
            spinner_phase: 0.0,
        }
    }

    pub fn trigger_tab_transition(&mut self) {
        if self.reduced_motion {
            self.tab_started = None;
            self.tab_progress = 1.0;
        } else {
            self.tab_started = Some(Instant::now());
            self.tab_progress = 0.0;
        }
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
        self.tab_started.is_some() || self.modal_started.is_some()
    }

    pub fn tick(&mut self, now: Instant) {
        if let Some(start) = self.tab_started {
            let t = (now.saturating_duration_since(start).as_secs_f32() / 0.15).min(1.0);
            self.tab_progress = ease_out_cubic(t);
            if t >= 1.0 {
                self.tab_started = None;
            }
        }
        if let Some(start) = self.modal_started {
            let t = (now.saturating_duration_since(start).as_secs_f32() / 0.17).min(1.0);
            self.modal_progress = ease_out_cubic(t);
            if t >= 1.0 {
                self.modal_started = None;
            }
        }
        let secs = now.saturating_duration_since(self.spinner_origin).as_secs_f32();
        self.spinner_phase = (secs * 1.45) % 1.0;
    }

    /// Subtle top vertical settle (in px) for tab content entrance (0 under reduced motion).
    pub fn tab_offset_y(&self) -> f32 {
        if self.reduced_motion {
            0.0
        } else {
            (1.0 - self.tab_progress) * 5.0
        }
    }

    /// Modal backdrop opacity factor (0..=1).
    pub fn modal_alpha(&self) -> f32 {
        if self.reduced_motion {
            1.0
        } else {
            self.modal_progress
        }
    }
}

/// Minimal macOS-style vector spinner arc rendered on GPU/tiny-skia Canvas.
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

        let track_circle = Path::circle(center, radius);
        frame.stroke(
            &track_circle,
            Stroke::default().with_width(1.8).with_color(self.track),
        );

        let start = self.phase * 2.0 * PI;
        let sweep = 1.45 * PI;
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
                .with_width(1.8)
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

/// macOS Storage / CleanMyMac proportional segmented horizontal breakdown bar.
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

        // Background track
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
    .height(Length::Fixed(8.0))
    .into()
}
