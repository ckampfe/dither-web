use console_log::console_log;
use image::{ImageBuffer, Luma};
use std::io::Cursor;
use web_sys::window;
use yew::prelude::*;
use yew::services::reader::ReaderTask;
use yew::services::{reader::FileData, ReaderService};
use yew::web_sys::File;

enum Msg {
    AddOne,
    FileSelection(Vec<File>),
    FileLoaded(FileData),
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    value: i64,
    image_urls: Vec<(String, String)>,
    tasks: Vec<ReaderTask>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
            image_urls: vec![],
            tasks: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
            Msg::FileSelection(files) => {
                for file in files {
                    let performance = window().unwrap().performance().unwrap();
                    let start = performance.now();
                    let callback = self.link.callback(Msg::FileLoaded);
                    let task = ReaderService::read_file(file, callback).unwrap();
                    self.tasks.push(task);
                    let end = performance.now();
                    console_log!("file selection time: {}", end - start);
                }

                true
            }
            Msg::FileLoaded(file) => {
                let performance = window().unwrap().performance().unwrap();

                let load_start = performance.now();
                console_log!("finished loading image: {}", &file.name);

                let original_image = image::load_from_memory(&file.content).unwrap();
                let luma_clone = original_image.to_luma8();
                let mut floyd_steinberg_clone = luma_clone.clone();
                let mut atkinson_clone = luma_clone.clone();
                let mut sierra_lite_clone = luma_clone.clone();
                let mut bayer_clone = luma_clone.clone();
                let mut random_threshold_clone = luma_clone;
                let load_end = performance.now();
                console_log!("load end: {}", load_end - load_start);

                ///////////////////////////////////////////

                let original_image = original_image.to_luma8();

                let original_bytes = encode_image_as_png_bytes(original_image);

                let original_url = bytes_to_object_url(&original_bytes, "image/png").unwrap();

                self.image_urls.push(("original".to_string(), original_url));

                ///////////////////////////////////////////
                // Floyd-Steinberg dithering
                console_log!("floyd start");
                let floyd_start = performance.now();
                dither::dither_floyd_steinberg(
                    &mut floyd_steinberg_clone,
                    &image::imageops::BiLevel,
                );

                let floyd_bytes = encode_image_as_png_bytes(floyd_steinberg_clone);

                let floyd_url = bytes_to_object_url(&floyd_bytes, "image/png").unwrap();
                console_log!("floyd done");

                self.image_urls.push(("floyd".to_string(), floyd_url));
                let floyd_end = performance.now();
                console_log!("floyd end: {}", floyd_end - floyd_start);
                ///////////////////////////////////////////
                console_log!("atkinson start");
                let atkinson_start = performance.now();
                dither::dither_atkinson(&mut atkinson_clone, &image::imageops::BiLevel);

                let atkinson_bytes = encode_image_as_png_bytes(atkinson_clone);

                let atkinson_url = bytes_to_object_url(&atkinson_bytes, "image/png").unwrap();
                console_log!("atkinson done");

                self.image_urls.push(("atkinson".to_string(), atkinson_url));
                let atkinson_end = performance.now();
                console_log!("atkinson end: {}", atkinson_end - atkinson_start);
                ///////////////////////////////////////////
                console_log!("sierra lite start");
                let sierra_lite_start = performance.now();
                dither::dither_sierra_lite(&mut sierra_lite_clone, &image::imageops::BiLevel);

                let sierra_lite_bytes = encode_image_as_png_bytes(sierra_lite_clone);

                let sierra_lite_url = bytes_to_object_url(&sierra_lite_bytes, "image/png").unwrap();
                console_log!("sierra lite done");

                self.image_urls
                    .push(("sierra lite".to_string(), sierra_lite_url));
                let sierra_lite_end = performance.now();
                console_log!("sierra lite end: {}", sierra_lite_end - sierra_lite_start);
                ///////////////////////////////////////////
                ///////////////////////////////////////////
                console_log!("bayer start");
                let bayer_start = performance.now();
                dither::dither_bayer(&mut bayer_clone, &image::imageops::BiLevel);

                let bayer_bytes = encode_image_as_png_bytes(bayer_clone);

                let bayer_url = bytes_to_object_url(&bayer_bytes, "image/png").unwrap();
                console_log!("bayer done");

                self.image_urls.push(("bayer".to_string(), bayer_url));
                let bayer_end = performance.now();
                console_log!("bayer end: {}", bayer_end - bayer_start);
                ///////////////////////////////////////////
                console_log!("random threshold start");
                let random_threshold_start = performance.now();
                dither::dither_random_threshold(
                    &mut random_threshold_clone,
                    &image::imageops::BiLevel,
                );

                let random_threshold_bytes = encode_image_as_png_bytes(random_threshold_clone);

                let random_threshold_url =
                    bytes_to_object_url(&random_threshold_bytes, "image/png").unwrap();
                console_log!("random threshold done");

                self.image_urls
                    .push(("random threshold".to_string(), random_threshold_url));
                let random_threshold_end = performance.now();
                console_log!(
                    "random threshold end: {}",
                    random_threshold_end - random_threshold_start
                );
                ///////////////////////////////////////////

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
                <input type="file" id="input" onchange=self.link.callback(move |v: ChangeData| {
                                let mut res = vec![];

                                if let ChangeData::Files(files) = v {
                                    if let Some(file) = files.get(0) {
                                        res.push(file);
                                    }
                                }

                                Msg::FileSelection(res)
                            }) />
                <div style="padding: 0; margin: 0; display: flex; flex-wrap: wrap;">
                    {
                        for self.image_urls.iter().map(|(title, image_url)| {
                            html! {
                                <div style="flex: 1; margin-right: 10px;">
                                    <div>
                                        <a href={ image_url.to_string() } alt={ title.to_string() } download={ title.to_string() }>{ title }</a>
                                    </div>

                                    <img src={ image_url.to_string() } alt={"meh"} />
                                </div>
                            }
                        })
                    }
                </div>
            </div>
        }
    }
}

fn encode_image_as_png_bytes(image: ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<u8> {
    let (x, y) = image.dimensions();

    let mut w = Cursor::new(Vec::new());
    let as_png = image::png::PngEncoder::new(&mut w);

    let page_as_bytes = image.into_raw();

    as_png
        .encode(&page_as_bytes, x, y, image::ColorType::L8)
        .unwrap();

    w.into_inner()
}

fn bytes_to_object_url(slice: &[u8], mime_type: &str) -> Result<String, wasm_bindgen::JsValue> {
    let mut blob_properties = web_sys::BlobPropertyBag::new();

    blob_properties.type_(mime_type);

    let bytearray = js_sys::Uint8Array::from(slice);

    let blob = web_sys::Blob::new_with_blob_sequence_and_options(
        &js_sys::Array::of1(&bytearray),
        &blob_properties,
    )?;

    web_sys::Url::create_object_url_with_blob(&blob)
}

fn main() {
    yew::start_app::<Model>();
}
