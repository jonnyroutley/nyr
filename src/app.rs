use chrono::Local;
use iocraft::prelude::*;
use std::time::Duration;

use crate::{progress_bar, targets};

#[derive(Default, Props)]
pub struct MainProps {
    pub target_progresses: Vec<targets::TargetProgress>,
}

#[component]
fn Main(mut hooks: Hooks, props: &MainProps) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut time = hooks.use_state(|| Local::now());
    let mut should_exit = hooks.use_state(|| false);

    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_secs(1)).await;
            time.set(Local::now());
        }
    });

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('q') => should_exit.set(true),
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(
            // subtract one in case there's a scrollbar
            width: width - 1,
            height,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        ) {
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
                margin_bottom: 2,
                padding_top: 2,
                padding_bottom: 2,
                padding_left: 8,
                padding_right: 8,
            ) {
                View(flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ) {
                    View(
                        margin_bottom: 1,
                    ) {
                        Text(content: "Resolutions 2025", weight: Weight::Bold, align: TextAlign::Center, )
                    }
                    #(props.target_progresses.iter().map(|target_progress| element! {
                        View {
                            progress_bar::StaticProgressBar(progress_percentage: target_progress.percentage, target: format!("{:.0}", target_progress.target_value), title: target_progress.name.clone())
                        }
                    }))

                }
            }
            Text(content: "Press \"q\" to quit.")
        }
    }
}

pub fn run_app(target_progresses: Vec<targets::TargetProgress>) {
    smol::block_on(element!(Main(target_progresses)).fullscreen()).unwrap();
}
