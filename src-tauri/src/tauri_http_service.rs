use rocket::{get, post, routes, Build, Rocket};
use rocket::serde::{json::Json, Serialize, Deserialize};
use rocket::http::Status;
use rocket::data::{Data, ToByteUnit};

use jsonwebtoken::{encode, Header, EncodingKey};
use jwt_compact::UntrustedToken;

use rand::rngs::SmallRng;
use rand::{SeedableRng, RngCore};

use rhai::Engine as RhaiEngine;
use wasmtime::Engine as WasmEngine;

use std::fs::File;

use nix::unistd::{chown, Gid, Uid};

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[get("/jwt/decode?<token>")]
fn refresh_token(
    // CWE 347
    //SOURCE
    token: String
) -> Json<ApiResponse> {
    let untrusted = match UntrustedToken::new(&token) {
        Ok(t) => t,
        Err(_) => return Json(ApiResponse {
            status: "error".to_string(),
            message: "Invalid token".to_string(),
        }),
    };

    // CWE 347
    //SINK
    let header = untrusted.header();

    let new_token = generate_new_token();

    Json(ApiResponse {
        status: "ok".to_string(),
        message: format!("generated new token: {}, header: {:?}", new_token, header),
    })
}

fn generate_new_token() -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE};

    let claims = Claims {
        sub: "user@example.com".to_string(),
        exp: 2000000000,
    };

    // CWE 330
    //SOURCE
    let mut rng = SmallRng::from_os_rng();

    let mut secret_bytes = [0u8; 32];

    // CWE 330
    //TAINT_TRANSFORMER
    rng.fill_bytes(&mut secret_bytes);
    let secret_b64 = URL_SAFE.encode(&secret_bytes);

    // CWE 330
    //SINK
    let key = EncodingKey::from_base64_secret(&secret_b64).unwrap();

    let token = encode(&Header::default(), &claims, &key).unwrap();

    token
}

#[post("/code_eval", data = "<code>")]
fn code_eval(
    // CWE 94
    //SOURCE
    code: String
) -> Json<ApiResponse> {
    let engine = RhaiEngine::new();

    // CWE 94
    //SINK
    match engine.eval::<i64>(&code) {
        Ok(result) => Json(ApiResponse {
            status: "ok".to_string(),
            message: result.to_string(),
        }),
        Err(err) => Json(ApiResponse {
            status: "error".to_string(),
            message: err.to_string(),
        }),
    }
}

#[get("/string_manipulation?<n>")]
pub fn string_manipulation(
    // CWE 606
    //SOURCE
    n: usize
) -> Json<ApiResponse> {
    // CWE 606
    //SINK
    if let Some(ch) = std::iter::repeat('A').nth(n) {
        Json(ApiResponse {
            status: "ok".to_string(),
            message: format!("Character at position {}: {}", n, ch),
        })
    } else {
        Json(ApiResponse {
            status: "error".to_string(),
            message: "Index out of bounds".to_string(),
        })
    }
}

#[post("/wasmtime/deserialize_open_file/unsafe", data = "<file_path>")]
pub async fn load_wasm_file(
    // CWE 502
    //SOURCE
    file_path: Data<'_>
) -> Result<Json<ApiResponse>, Status> {

    let user_input: String = file_path
        .open(1.mebibytes())
        .into_string()
        .await
        .map_err(|_| Status::BadRequest)?
        .into_inner();

    let engine = WasmEngine::default();

    let file = File::open(user_input.trim())
        .map_err(|_| Status::BadRequest)?;

    // CWE 502
    //SINK
    let module = unsafe { wasmtime::Module::deserialize_open_file(&engine, file) }
        .map_err(|_| Status::BadRequest)?;

    Ok(Json(ApiResponse {
        status: "ok".to_string(),
        message: format!("WASM module loaded from module: {:?}", module),
    }))
}

#[get("/calculate_remainder?<div>")]
fn calculate_remainder(
    // CWE 369
    //SOURCE
    div: i32
) -> Json<ApiResponse> {
    let mut dividend = 100;

    // CWE 369
    //SINK
    dividend %= div;

    Json(ApiResponse {
        status: "ok".to_string(),
        message: format!("Remainder: {}", dividend),
    })
}

#[get("/allocate_resources?<size>")]
fn allocate_resources(
    // CWE 789
    //SOURCE
    size: usize
) -> Json<ApiResponse> {
    let mut v: Vec<u8> = Vec::new();

    // CWE 789
    //SINK
    v.resize(size, 0u8);

    if v.len() != size {
        return Json(ApiResponse {
            status: "error".to_string(),
            message: "Failed to allocate resources".to_string(),
        });
    }

    let result = calculate_array();

    if result < 0 {
        return Json(ApiResponse {
            status: "error".to_string(),
            message: "Computation error".to_string(),
        });
    }

    Json(ApiResponse {
        status: "ok".to_string(),
        message: format!("Allocated {} bytes", v.len()),
    })
}

fn calculate_array() -> i32 {
    let arr = [1, 2, 3, 4, 5];
    let mut sum = 0;
    for &num in arr.iter() {
        sum += num;
    }
    sum
}

#[get("/permission_update?<path>")]
pub fn permission_update(
    // CWE 732
    //SOURCE
    path: String
) -> Json<ApiResponse> {
    let uid = Some(Uid::from_raw(1000));
    let gid = Some(Gid::from_raw(1000));

    // CWE 732
    //SINK
    let _ = chown(path.as_str(), uid, gid);

    Json(ApiResponse {
        status: "ok".to_string(),
        message: format!("Changed ownership of {} to UID: {}, GID: {}", path, uid.unwrap(), gid.unwrap()),
    })
}

#[get("/verify_ssl")]
pub fn verify_ssl() -> Json<ApiResponse> {
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();

    // CWE 295
    //SINK
    builder.set_verify(SslVerifyMode::NONE);

    Json(ApiResponse {
        status: "ok".to_string(),
        message: "SSL verified".to_string(),
    })
}

pub fn create_rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![refresh_token, code_eval, load_wasm_file, string_manipulation, calculate_remainder, allocate_resources, permission_update, verify_ssl])
}
