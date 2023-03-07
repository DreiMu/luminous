use std::{env, sync::Arc};

use reqwest::Client;
use serde::Deserialize;
use serenity::{prelude::Context, model::prelude::GuildId};

#[derive(Debug, Deserialize)]
struct JsonResponse {
    completions: Vec<Completion>,
}

#[derive(Debug, Deserialize)]
struct Completion {
    completion: String,
}


// Idee: Zusammenfassen der Unterhaltung in einen String, der als Kontext an die KI übergeben wird.
pub(crate) async fn request(context: Vec<(u64, String)>, ctx: Context) -> String {
    let mut string = "Eine Konversation zwischen Luminous, einer KI und mehreren Menschen.\\n".to_string();
    for (id, content) in context {
        let user = ctx.http.get_user(id).await.expect("User not found");
        let name = if ctx.cache.current_user().id.0 == id {
            "Luminous".to_string()
        } else {
            user.nick_in(&ctx.http, GuildId(env::var("GUILD_ID").expect("Guild ID").parse::<u64>().expect("Guild ID Not a number"))).await.unwrap_or(user.name)
        };
        string = format!("{string}{name}: {content}\\n");
    }

    string = format!("{string}Luminous: ");

    println!("{string:?}");

    let body = format!(r#"{}"model": "luminous-supreme","prompt": "{string}","maximum_tokens": 256, "temperature": 1.0, "stop_sequences": ["\n"]{}"#, "{", "}");

    let token = env::var("LUMINOUS_API_KEY").expect("token");
    println!("{body}");

    let response = Client::new()
        .post("https://api.aleph-alpha.com/complete")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(body.to_string())
        .send().await;

    println!("{response:?}");

    if response.is_err() {
        panic!("{:?}",response.err().unwrap());
    }

    let response2 = response.expect("Error in Request").json::<JsonResponse>().await;
    response2.expect("Error in Response").completions[0].completion.clone()

}