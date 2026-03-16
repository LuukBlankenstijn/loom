mod client;
mod map;

use std::sync::Arc;

use iced::Font;
use iced::Theme;
use iced::font;
use loom_rpc::map::v1::map_service_client::MapServiceClient;
use map::Map;
use tonic_web_wasm_client::Client;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    const FIRA_SANS_BYTES: &[u8] = include_bytes!("../fonts/FiraSans-Regular.ttf");
    let client = Arc::new(get_client());
    let app = iced::application(
        move || Map::new(client.clone(), get_map_id_from_url()),
        Map::update,
        Map::view,
    )
    .theme(Theme::Dracula)
    .font(FIRA_SANS_BYTES)
    .default_font(Font {
        family: font::Family::Name("Fira Sans"),
        ..Font::DEFAULT
    });

    let _ = app.run();
}

fn get_client() -> MapServiceClient<Client> {
    let window = web_sys::window().expect("no global `window` exists");
    let location = window.location();
    let origin = location.origin().expect("failed to get origin");
    let base_url = format!("{}/api", origin);
    let wasm_transport = Client::new(base_url);
    MapServiceClient::new(wasm_transport)
}

fn get_map_id_from_url() -> i32 {
    let window = web_sys::window().expect("no global `window` exists");
    let search = window.location().search().expect("failed to getsearch");
    let params =
        web_sys::UrlSearchParams::new_with_str(&search).expect("failed to parse query string");

    params
        .get("mapId")
        .expect("mapId not found in query string")
        .parse::<i32>()
        .expect("mapId is not a valid i32")
}
