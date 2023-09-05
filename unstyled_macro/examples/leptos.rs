use leptos::ssr::render_to_string;
use leptos::*;

#[component]
pub fn MyUnstyledComponent(cx: Scope) -> impl IntoView {
    let class_name = unstyled::style! {"
        @keyframes rainbow-text {
            0% {
                color: hsl(0deg 100% 50%);
            }

            16% {
                color: hsl(60deg 100% 50%);
            }

            33% {
                color: hsl(120deg 100% 50%);
            }

            50% {
                color: hsl(180deg 100% 50%);
            }

            66% {
                color: hsl(240deg 100% 50%);
            }

            82% {
                color: hsl(300deg 100% 50%);
            }

            100% {
                color: hsl(360deg 100% 50%);
            }
        }

        .title {
            animation: rainbow-text infinite 2s;
        }
    "};

    view! {cx, class = class_name,
        <header>
            <h1 class="title">"Style the, cruel, Unstyled world!"</h1>
            <nav>
                <ul>
                    <li><a href="https://github.com/YannikSc">My GitHub Profile</a></li>
                    <li><a href="https://github.com/YannikSc/Unstyled">Unstyled</a></li>
                </ul>
            </nav>
        </header>
    }
}

pub fn main() {
    let output = render_to_string(|cx| {
        view! {cx,
            <html lang="en">
            <head>
                <title>"My Unstyled test"</title>
                <link rel="stylesheet" href="/unstyled.css" />
            </head>
            <body>
                <MyUnstyledComponent />
            </body>
            </html>
        }
    });

    write_to_target(&output);

    println!("{output}");
}

fn write_to_target(html: &str) {
    let output = std::env::current_dir()
        .unwrap()
        .join("../../target")
        .join("index.html");
    std::fs::write(output, html).expect("Could not write index.html");
}
