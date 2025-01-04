use sqlx::FromRow;
use iocraft::prelude::*;

#[derive(Clone, FromRow, Debug)]
pub struct Target {
    id: i64,
    name: String,
    target_date: chrono::NaiveDate,
    status: String,
    start_value: f64,
    target_value: f64,
}

#[derive(Default, Props)]
pub struct TargetsTableProps<'a> {
    pub targets: Option<&'a Vec<Target>>,
    pub title: &'a str,
}

#[component]
pub fn TargetsTable<'a>(props: &TargetsTableProps<'a>) -> impl Into<AnyElement<'a>> {
    element! {
        View(
            margin_top: 1,
            margin_bottom: 1,
            flex_direction: FlexDirection::Column,
            width: 100,
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(width: 100pct, justify_content: JustifyContent::Center, margin_bottom:1, ) {
                Text(content: props.title, weight: Weight::Bold )
            }

            View(border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {
                View(width: 10pct, justify_content: JustifyContent::Center) {
                    Text(content: "id", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 40pct, justify_content: JustifyContent::Center) {
                    Text(content: "name", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "target date", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "status", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "start", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
                View(width: 12.5pct, justify_content: JustifyContent::Center) {
                    Text(content: "target", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
            }

            #(props.targets.map(|targets| targets.iter().enumerate().map(|(i, target)| element! {
                View(background_color: if i % 2 == 0 { None } else { Some(Color::DarkGrey) }) {
                    View(width: 10pct, justify_content: JustifyContent::Center) {
                        Text(content: target.id.to_string())
                    }

                    View(width: 40pct, justify_content: JustifyContent::Center) {
                        Text(content: target.name.clone())
                    }

                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.target_date.to_string())
                    }
                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.status.to_string())
                    }
                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.start_value.to_string())
                    }
                    View(width: 12.5pct, justify_content: JustifyContent::Center) {
                        Text(content: target.target_value.to_string())
                    }
                }
            })).into_iter().flatten())
        }
    }
}
