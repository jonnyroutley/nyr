use iocraft::prelude::*;
use std::time::Duration;

#[derive(Props)]
pub struct ProgressBarProps<'a> {
    pub progress_percentage: &'a f64,
}

impl<'a> Default for ProgressBarProps<'a> {
    fn default() -> Self {
        ProgressBarProps {
            progress_percentage: &0.0,
        }
    }
}

#[component]
pub fn ProgressBar<'a>(
    mut hooks: Hooks,
    props: &ProgressBarProps<'a>,
) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut progress = hooks.use_state::<f32, _>(|| 0.0);

    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_millis(100)).await;
            progress.set((progress.get() + 0.5).min(100.0));
        }
    });

    if progress.get() >= *props.progress_percentage as f32 {
        system.exit();
    }

    element! {
        View {
            View(border_style: BorderStyle::Round, border_color: Color::Blue, width: 100) {
                View(width: Percent(progress.get()), height: 1, background_color: Color::Green)
            }
            View(padding: 1) {
                Text(content: format!("{:.1}%", progress))
            }
        }
    }
}
