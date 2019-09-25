use rand::Rng;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::Write;
#[macro_use]
extern crate serde;
extern crate serde_xml_rs;

//
// toml
//
#[derive(Deserialize, Debug, Default, Clone)]
struct Toml {
    config: Config,
}
#[derive(Deserialize, Debug, Default, Clone)]
struct Config {
    varbool: bool,
    variant_name: String,
    xl_dir_path: String,
    ware_path: String,
    t_path: String,
    out_path: String,
}
//
// xml ship
//
#[derive(Deserialize, Debug, Default)]
struct Macros {
    r#macro: NameMacro,
}

#[derive(Deserialize, Debug, Default)]
struct NameMacro {
    name: String,
    class: String,
    component: Component,
    properties: Properties,
}
#[derive(Deserialize, Debug, Default)]
struct Component {
    r#ref: String,
}
#[derive(Deserialize, Debug, Default)]
struct Properties {
    identification: Identification,
    purpose: Purpose,
}
#[derive(Deserialize, Debug, Default)]
struct Identification {
    name: String,
    basename: String,
    description: String,
    variation: String,
    shortvariation: String,
    icon: String,
}
#[derive(Deserialize, Debug, Default)]
struct Purpose {
    primary: String,
    // <purpose primary="fight" />
}

//
//xml ware
//

// #[derive(Deserialize, Debug, Default)]
// struct Wares {
//     ware: Ware,
// }
#[derive(Deserialize, Debug, Default)]
struct Ware {
    id: String,
    name: String,
    description: String,
    restriction: Restriction,
    owner: Owner,
}
#[derive(Deserialize, Debug, Default)]
struct Restriction {
    licence: String,
}
#[derive(Deserialize, Debug, Default)]
struct Owner {
    faction: String,
}
/*
<ware id="ship_xen_xl_destroyer_01_a" name="{20101,70501}" description="{20101,70511}" group="ships_xenon" transport="ship" volume="1" tags="noplayerblueprint ship">
    <price min="1033787" average="1216220" max="1398653" />
    <production time="526" amount="1" method="default" name="{20206,601}">
      <primary>
        <ware ware="energycells" amount="2908" />
        <ware ware="ore" amount="2437" />
        <ware ware="silicon" amount="2447" />
      </primary>
    </production>
    <component ref="ship_xen_xl_destroyer_01_a_macro" />
    <restriction licence="capitalship" />
    <owner faction="xenon" />
  </ware>
*/

fn main() {
    let mut ware_file_string = "".to_string();
    let mut ware_new = "".to_string();
    let toml_str = include_str!("Config.toml");
    let toml_parsed: Toml = toml::from_str(&toml_str).unwrap();
    let variant = &toml_parsed.config.varbool;

    for entry in fs::read_dir(&toml_parsed.config.xl_dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            let mut macro_string = fs::read_to_string(&path).unwrap();
            let macro_parsed: Macros = serde_xml_rs::from_str(&macro_string).unwrap_or_default();
            let macroname = &path.file_name().unwrap().to_str().unwrap();
            let ware_string = fs::read_to_string(&toml_parsed.config.ware_path).unwrap();
            for ware in ware_string.split_terminator("</ware>") {
                if ware.contains(&macroname.replace(".xml", "")) {
                    ware_new.push_str(&ware);

                    ware_new.push_str(
                        "
                    </ware>",
                    );

                    let ware_parsed: Ware = serde_xml_rs::from_str(&ware_new).unwrap_or_default();
                    //  println!("{:?}", &ware);
                    // println!("{:#?}", ware_parsed);
                    if ware_new.contains("</ware>") {
                        println!("{:#?}", ware_parsed);
                    }
                    // println!("{:#?}", macro_parsed);
                    let mut prng = rand::thread_rng();
                    let ware_price = format!(
                        "<price min=\"{}\" average=\"{}\" max=\"{}\" />",
                        randomize(prng.gen_range(0.5, 5.0), 25000),
                        randomize(prng.gen_range(0.5, 5.0), 25000),
                        randomize(prng.gen_range(0.5, 5.0), 25000)
                    );
                    let re = Regex::new("<price.* />").unwrap();
                    let ware_new = re.replace(&ware_new, ware_price.as_str());
                    if variant == &true {
                        // let ware = ware_replace(ware: String, macroname: String, VARIANT_NAME: String, tname: i32, pageid: String, tdesc: i32)

                        ware_file_string.push_str(
                            "
                            ",
                        );
                        ware_file_string.push_str(&ware_new);
                    } else {
                        ware_file_string.push_str(
                            "
                            ",
                        );
                        ware_file_string.push_str(&ware_new);
                    }
                }
            }
            let macro_string = replace_pattern(
                macro_parsed.r#macro.properties.identification.description,
                macro_string,
                "pasta",
            );

            output(
                &toml_parsed.config.out_path,
                &path,
                &toml_parsed.config.variant_name,
                &ware_file_string,
            );
            // output(
            //     &toml_parsed.config.out_path,
            //     &path,
            //     &toml_parsed.config.variant_name,
            //     &macro_string,
            // );
        }
    }
}
fn replace_pattern(pattern: String, text: String, replace: &str) -> String {
    if pattern != "" {
        let mut text = text;
        text = text.replace(pattern.as_str(), &replace);
        text
    } else {
        text
    }
}
// randomize(prng.gen_range(0.5, 5.0), 25000);
fn randomize(multi: f32, input: i32) -> String {
    let result = multi * input as f32;
    (result as i32).to_string()
}

fn output(path: &String, pathbuf: &std::path::PathBuf, variant: &String, macro_string: &String) {
    let mut outputfile = File::create(
        format!(
            "{}{}",
            path,
            pathbuf.file_name().unwrap().to_str().unwrap().to_string()
        )
        .replace("_macro", &[&variant.as_str(), "_macro"].concat()),
    )
    .unwrap();
    outputfile.write_all(macro_string.as_bytes()).unwrap();
}

fn ware_replace(
    ware: String,
    macroname: String,
    VARIANT_NAME: String,
    tname: i32,
    pageid: String,
    tdesc: i32,
) -> String {
    // name="{20101,70501}" description="{20101,70511}"
    let re = Regex::new("ref=.* a").unwrap();
    let ware = re.replace(&ware, "ref=\"pasta\" a");
    let ware = ware.replace(
        "pasta",
        &macroname.replace(
            "_macro.xml",
            &[VARIANT_NAME.to_string(), "_macro".to_string()].concat(),
        ),
    );
    // print!("{:?}", ware);
    let re = Regex::new("<ware id=.* name=").unwrap();
    let ware = re.replace(&ware, "<ware id=\"pasta\" name=");
    let ware = ware.replace(
        "pasta",
        &macroname.replace(
            "_macro.xml",
            &[VARIANT_NAME.to_string(), "_ware".to_string()].concat(),
        ),
    );
    // <identification name="{20101,11002}"
    let re = Regex::new("name=.* des").unwrap();
    let ware = re.replace(&ware, "name=\"pasta}\" des");
    let tword = pageid.to_string() + &tname.to_string();
    let ware = ware.replace("pasta", &tword);

    // description="{20101,11012}"
    let re = Regex::new("description=.* gr").unwrap();
    let ware = re.replace(&ware, "description=\"pasta}\" gr");
    let tword = pageid.to_string() + &tdesc.to_string();
    let ware = ware.replace("pasta", &tword);
    ware
}
