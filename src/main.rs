use actix_web::{
    get,
    web,
    App, HttpResponse, HttpServer, Responder,
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize};

#[derive(Deserialize)]
struct StatefulOsuAuthorizeInfo {
    client_id: String,
    redirect_uri: String,
    source_service: String
}

#[derive(Deserialize)]
struct EnvironmentConfig {
    client_id: String,
    allowed_origins: Vec<String>
}

#[derive(Deserialize)]
struct StatefulOAuthCode {
    code: String,
    state: String
}
 
struct AppState {
    environment: EnvironmentConfig,
}
 
#[get("/authorize")]
async fn authorize_proxy(state: web::Data<AppState>, query: web::Query<StatefulOsuAuthorizeInfo>) -> impl Responder {
    
    if state.environment.client_id != query.client_id && !state.environment.allowed_origins.contains(&query.source_service)  {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let base64_source = general_purpose::STANDARD.encode(query.source_service.as_bytes());   

    let name = format!(
        "https://osu.ppy.sh/oauth/authorize?client_id={}&redirect_uri={}&response_type={}&scope={}&state={}",
        query.client_id,
        query.redirect_uri,
        "code".to_owned(),
        "identify".to_owned(),
        base64_source 
    );

    HttpResponse::Found().append_header(("Location", name)).finish()
}

#[get("/callback")]
async fn authorize_callback(
    code: web::Query<StatefulOAuthCode>,
) -> impl Responder {

    let base64_dst = general_purpose::STANDARD.decode(&code.state).unwrap();
    let slice = String::from_utf8(base64_dst);

    match slice {
        Ok(val) => {
            let url = format!("{}?code={}", val, code.code);

            HttpResponse::Found().append_header(("Location", url)).finish()
        },
        Err(_) => HttpResponse::BadRequest().body("Bad Request. This likely means the administrator has misconfigured the application.")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let env = envy::from_env::<EnvironmentConfig>().expect("osu! env vars not set");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                environment: EnvironmentConfig {
                    client_id: env.client_id.to_owned(),
                    allowed_origins: env.allowed_origins.clone()
                },
            }))
            .service(authorize_proxy)
            .service(authorize_callback)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
