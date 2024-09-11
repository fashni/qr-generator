use std::collections::HashMap;
use std::io::Cursor;
use qrcode::QrCode;
use qrcode::render::{svg, unicode};
use image::{Luma, ImageFormat};
use serde::{Deserialize, Serialize};
use worker::*;


#[derive(Debug, Deserialize, Serialize)]
struct JsonInputs {
  data: String,
  format: Option<String>,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let router: Router<()> = Router::new();
  router
    .get_async("/generate", generate_qr_get)
    .post_async("/generate", generate_qr_post)
    .run(req, env)
    .await
}

fn generate_qr(data: String, format: String) -> Result<Response> {
  fn set_content_type(mut resp: Response, content_type: &str) -> Response {
    resp.headers_mut().set("Content-Type", content_type).unwrap();
    resp
  }

  let qr: QrCode = QrCode::new(data).unwrap();
  match format.as_str() {
    "png" => {
      let img: image::ImageBuffer<Luma<u8>, Vec<u8>> = qr.render::<Luma<u8>>().build();
      let mut buf: Vec<u8> = Vec::new();
      let mut cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buf);
      img.write_to(&mut cursor, ImageFormat::Png).unwrap();
      Response::from_bytes(buf)
        .map(|resp: Response| set_content_type(resp, "image/png"))
    },
    "svg" => {
      let svg_str: String = format!("{}\n", qr.render::<svg::Color>().build());
      Response::ok(svg_str)
        .map(|resp: Response| set_content_type(resp, "image/svg+xml"))
    },
    "unicode" => {
      let unicode_str: String = format!("{}\n", qr.render::<unicode::Dense1x2>()
        .light_color(unicode::Dense1x2::Dark)
        .dark_color(unicode::Dense1x2::Light)
        .build());
      Response::ok(unicode_str)
        .map(|resp: Response| set_content_type(resp, "text/plain; charset=UTF-8"))
    },
    "unicode_inverted" => {
      let unicode_str = format!("{}\n", qr.render::<unicode::Dense1x2>().build());
      Response::ok(unicode_str)
        .map(|resp: Response| set_content_type(resp, "text/plain; charset=UTF-8"))
    },
    _ => Response::error("Unsupported format\n", 400)
  }
}

pub async fn generate_qr_get(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
  let url: Url = req.url().unwrap();
  let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  let data: String = match params.get("data") {
    Some(d) => d.clone(),
    None => return Response::error("Missing `data` parameter\n", 400)
  };
  let format: String = params.get("format").cloned().unwrap_or_else(|| "png".to_string());
  generate_qr(data, format)
}

pub async fn generate_qr_post(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
  let req_data: JsonInputs = match req.json().await {
    Ok(inputs) => inputs,
    Err(_) => return Response::error("Invalid JSON body\n", 400)
  };
  let data: String = req_data.data;
  let format: String = req_data.format.unwrap_or_else(|| "unicode".to_string());
  generate_qr(data, format)
}
