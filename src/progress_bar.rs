use iocraft::prelude::*;
// use sqlx::any::AnyValue;
use std::time::Duration;

#[derive(Props)]
pub struct ProgressBarProps<'a> {
    pub target_id: &'a i64,
}

impl<'a> Default for ProgressBarProps<'a> {
    fn default() -> Self {
        ProgressBarProps {
            target_id: &0,
        }
    }
}

// fn fetch_progress (target_id: &i64) -> f64 {
//   let target = 
// }

#[component]
pub fn ProgressBar<'a>(
    mut hooks: Hooks,
    // props: &ProgressBarProps<'a>
) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut progress = hooks.use_state::<f32, _>(|| 0.0);

    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_millis(100)).await;
            progress.set((progress.get() + 2.0).min(100.0));
        }
    });

    if progress >= 100.0 {
        system.exit();
    }

    element! {
        View {
            View(border_style: BorderStyle::Round, border_color: Color::Blue, width: 100) {
                View(width: Percent(progress.get()), height: 1, background_color: Color::Green)
            }
            View(padding: 1) {
                Text(content: format!("{:.0}%", progress))
            }
        }
    }
}
