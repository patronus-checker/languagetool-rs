extern crate languagetool;

use languagetool::{LanguageTool, Request};

fn main() {
    let text = String::from("borken");
    let lang = String::from("en-GB");
    let instance_url = "http://localhost:8081/";

    match LanguageTool::new(instance_url) {
        Ok(lt) => match lt.list_languages() {
            Ok(languages) => {
                for language in languages {
                    println!("{:?}", language);
                }

                let req = Request {
                    mother_tongue: Some(String::from("sk")),
                    ..Request::new(text, lang)
                };

                match lt.check(req) {
                    Ok(check) => {
                        println!("{:?}", check);
                    }
                    Err(msg) => println!("Cannot check the text: {:?}", msg),
                }
            }
            Err(msg) => println!("Cannot obtain list of languages: {:?}", msg),
        },
        Err(msg) => println!("Cannot initialise LanguageTool: {:?}", msg),
    }
}
