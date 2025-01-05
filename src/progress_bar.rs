use iocraft::prelude::*;
use std::time::Duration;

#[derive(Props)]
pub struct ProgressBarProps {
    pub progress_percentage: f64,
    pub title: String,
    pub target: String,
}

impl Default for ProgressBarProps {
    fn default() -> Self {
        ProgressBarProps {
            progress_percentage: 0.0,
            title: String::from("Progress"),
            target: String::from(""),
        }
    }
}

#[component]
pub fn ProgressBar(mut hooks: Hooks, props: &ProgressBarProps) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut progress = hooks.use_state::<f32, _>(|| 0.0);

    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_millis(100)).await;
            progress.set((progress.get() + 0.5).min(100.0));
        }
    });

    if progress.get() >= props.progress_percentage as f32 {
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

#[component]
pub fn StaticProgressBar(
    // mut hooks: Hooks,
    props: &ProgressBarProps,
) -> impl Into<AnyElement<'static>> {
    element! {
        View {

            View(padding: 1, width: 10) {
                Text(content: format!("{}", props.title))
            }
            View(border_style: BorderStyle::Round, border_color: Color::Blue, width: 60) {
                View(width: Percent(props.progress_percentage as f32), height: 1, background_color: Color::Green)
            }
            // View(padding: 1) {
            //     Text(content: format!("{:.1}%", props.progress_percentage))
            // }
            View(padding: 1) {
                Text(content: format!("{}", props.target))
            }
        }
    }
}
