use console_log::console_log;
use image::{ImageBuffer, Luma};
use std::io::Cursor;
use web_sys::window;
use yew::prelude::*;
use yew::services::reader::ReaderTask;
use yew::services::{reader::FileData, ReaderService};
use yew::web_sys::File;

const VERSION: &str = env!("DITHER_WEB_VERSION");
const IMAGE_PNG_MIME_TYPE: &str = "image/png";

enum Msg {
    FileSelection(Vec<File>),
    FileLoaded(FileData),
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    image_urls: Vec<(String, String, f64)>,
    tasks: Vec<ReaderTask>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            image_urls: vec![],
            tasks: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
                let all_start = performance.now();

                let load_start = performance.now();
                console_log!("finished loading image: {}", &file.name);

                let original_image = image::load_from_memory(&file.content).unwrap().to_luma8();
                console_log!(
                    "width, height",
                    original_image.width(),
                    original_image.height()
                );
                // let mut floyd_steinberg_clone = working_image.clone();
                // let mut atkinson_clone = working_image.clone();
                // let mut sierra_lite_clone = working_image.clone();
                // let mut bayer_clone = working_image.clone();
                // let mut random_threshold_clone = working_image;
                let load_end = performance.now();
                console_log!("load end: {}", load_end - load_start);

                ///////////////////////////////////////////

                console_log!("original image len", original_image.len());

                let original_bytes = encode_image_as_png_bytes(&original_image);
                console_log!("original bytes len", original_bytes.len());

                let original_url =
                    bytes_to_object_url(&original_bytes, IMAGE_PNG_MIME_TYPE).unwrap();

                self.image_urls
                    .push(("original".to_string(), original_url, 0.0));

                ///////////////////////////////////////////

                // This ImageBuffer is reused by all dither functions.
                // All dither functions must fill it with a clone of the original image
                // prior to use.
                //
                // This is an optimization to avoid allocating a separate buffer
                // for every dither.
                let mut working_image = original_image.clone();

                console_log!("floyd start");
                let floyd_start = performance.now();
                dither::dither_floyd_steinberg(&mut working_image, &image::imageops::BiLevel);

                let floyd_bytes = encode_image_as_png_bytes(&working_image);

                let floyd_url = bytes_to_object_url(&floyd_bytes, IMAGE_PNG_MIME_TYPE).unwrap();
                console_log!("floyd done");

                let floyd_end = performance.now();
                self.image_urls.push((
                    "Floyd-Steinberg".to_string(),
                    floyd_url,
                    floyd_end - floyd_start,
                ));
                console_log!("floyd end: {}", floyd_end - floyd_start);

                ///////////////////////////////////////////

                working_image = original_image.clone();

                console_log!("atkinson start");
                let atkinson_start = performance.now();
                dither::dither_atkinson(&mut working_image, &image::imageops::BiLevel);

                let atkinson_bytes = encode_image_as_png_bytes(&working_image);

                let atkinson_url =
                    bytes_to_object_url(&atkinson_bytes, IMAGE_PNG_MIME_TYPE).unwrap();
                console_log!("atkinson done");

                let atkinson_end = performance.now();
                self.image_urls.push((
                    "Atkinson".to_string(),
                    atkinson_url,
                    atkinson_end - atkinson_start,
                ));
                console_log!("atkinson end: {}", atkinson_end - atkinson_start);

                ///////////////////////////////////////////

                working_image = original_image.clone();

                console_log!("sierra lite start");
                let sierra_lite_start = performance.now();
                dither::dither_sierra_lite(&mut working_image, &image::imageops::BiLevel);

                let sierra_lite_bytes = encode_image_as_png_bytes(&working_image);

                let sierra_lite_url =
                    bytes_to_object_url(&sierra_lite_bytes, IMAGE_PNG_MIME_TYPE).unwrap();
                console_log!("sierra lite done");

                let sierra_lite_end = performance.now();
                self.image_urls.push((
                    "Sierra Lite".to_string(),
                    sierra_lite_url,
                    sierra_lite_end - sierra_lite_start,
                ));
                console_log!("sierra lite end: {}", sierra_lite_end - sierra_lite_start);

                ///////////////////////////////////////////

                working_image = original_image.clone();

                console_log!("bayer start");
                let bayer_start = performance.now();
                dither::dither_bayer(&mut working_image, &image::imageops::BiLevel);

                let bayer_bytes = encode_image_as_png_bytes(&working_image);

                let bayer_url = bytes_to_object_url(&bayer_bytes, IMAGE_PNG_MIME_TYPE).unwrap();
                console_log!("bayer done");

                let bayer_end = performance.now();
                self.image_urls
                    .push(("Bayer".to_string(), bayer_url, bayer_end - bayer_start));
                console_log!("bayer end: {}", bayer_end - bayer_start);

                ///////////////////////////////////////////

                working_image = original_image;

                console_log!("random threshold start");
                let random_threshold_start = performance.now();
                dither::dither_random_threshold(&mut working_image, &image::imageops::BiLevel);

                let random_threshold_bytes = encode_image_as_png_bytes(&working_image);

                let random_threshold_url =
                    bytes_to_object_url(&random_threshold_bytes, IMAGE_PNG_MIME_TYPE).unwrap();
                console_log!("Random threshold done");

                let random_threshold_end = performance.now();
                self.image_urls.push((
                    "random threshold".to_string(),
                    random_threshold_url,
                    random_threshold_end - random_threshold_start,
                ));
                console_log!(
                    "random threshold end: {}",
                    random_threshold_end - random_threshold_start
                );

                ///////////////////////////////////////////

                let all_end = performance.now();

                console_log!("all time", all_end - all_start);

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
                <div>
                    <a href="https://github.com/ckampfe/dither-web">{ format!("source code version {}", VERSION) }</a>
                </div>
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
                        for self.image_urls.iter().map(|(title, image_url, dither_ms)| {
                            html! {
                                <div style="flex: 1; margin-right: 10px;">
                                    <div>
                                        <a href={ image_url.to_string() } alt={ title.to_string() } download={ title.to_string() }>{ title }</a>
                                    </div>

                                    <div>{ format!("Time taken: {}ms", dither_ms) }</div>

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

fn encode_image_as_png_bytes(image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<u8> {
    let (x, y) = image.dimensions();

    let mut w = Cursor::new(Vec::new());
    let as_png = image::png::PngEncoder::new(&mut w);

    let page_as_bytes = image.as_ref();

    as_png
        .encode(page_as_bytes, x, y, image::ColorType::L8)
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
