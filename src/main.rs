use console_log::console_log;
use image::imageops::ColorMap;
use image::{ImageBuffer, Pixel};
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

                ///////////////////////////////////////////

                self.tasks.clear();

                for (_title, url, _processing_time_ms) in &self.image_urls {
                    let _ = web_sys::Url::revoke_object_url(url);
                }

                self.image_urls.clear();

                ///////////////////////////////////////////

                let load_start = performance.now();
                console_log!("finished loading image: {}", &file.name);

                let original_image = image::load_from_memory(&file.content).unwrap().to_luma8();
                console_log!(
                    "width, height",
                    original_image.width(),
                    original_image.height()
                );

                let load_end = performance.now();
                console_log!("load end: {}", load_end - load_start);

                ///////////////////////////////////////////

                console_log!("original image len", original_image.len());

                let original_bytes = encode_image_as_png_bytes(&original_image);
                console_log!("original bytes len", original_bytes.len());

                let original_url =
                    bytes_to_object_url(&original_bytes, IMAGE_PNG_MIME_TYPE).unwrap();

                self.image_urls
                    .push(("Original image".to_string(), original_url, 0.0));

                ///////////////////////////////////////////

                let dithers = all_dithers(&original_image, &image::imageops::BiLevel);

                self.image_urls.extend_from_slice(&dithers);

                let all_end = performance.now();

                console_log!("all time", all_end - all_start);

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

pub fn all_dithers<Pix, Map>(
    image: &ImageBuffer<Pix, Vec<u8>>,
    color_map: &Map,
) -> Vec<(String, String, f64)>
where
    Map: ColorMap<Color = Pix> + ?Sized,
    Pix: Pixel<Subpixel = u8> + 'static,
{
    let performance = window().unwrap().performance().unwrap();
    let mut working_image = image.clone();

    [
        (
            "Floyd-Steinberg",
            Box::new(dither::dither_floyd_steinberg)
                as Box<dyn FnMut(&mut ImageBuffer<Pix, Vec<u8>>, &Map)>,
        ),
        (
            "Atkinson",
            Box::new(dither::dither_atkinson)
                as Box<dyn FnMut(&mut ImageBuffer<Pix, Vec<u8>>, &Map)>,
        ),
        (
            "Sierra Lite",
            Box::new(dither::dither_sierra_lite)
                as Box<dyn FnMut(&mut ImageBuffer<Pix, Vec<u8>>, &Map)>,
        ),
        (
            "Bayer",
            Box::new(dither::dither_bayer) as Box<dyn FnMut(&mut ImageBuffer<Pix, Vec<u8>>, &Map)>,
        ),
        (
            "Random threshold",
            Box::new(dither::dither_random_threshold)
                as Box<dyn FnMut(&mut ImageBuffer<Pix, Vec<u8>>, &Map)>,
        ),
    ]
    .iter_mut()
    .map(|(title, f)| {
        let start = performance.now();

        working_image = image.clone();

        f(&mut working_image, color_map);

        let png_bytes = encode_image_as_png_bytes(&working_image);

        let dithered_image_url = bytes_to_object_url(&png_bytes, IMAGE_PNG_MIME_TYPE).unwrap();

        let end = performance.now();

        console_log!((*title), "time", end - start);

        (title.to_string(), dithered_image_url, end - start)
    })
    .collect()
}

fn encode_image_as_png_bytes<Pix>(image: &ImageBuffer<Pix, Vec<u8>>) -> Vec<u8>
where
    Pix: Pixel<Subpixel = u8> + 'static,
{
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
