use rand::seq::SliceRandom;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
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
    xlconfig: xl_config,
    faction_vec: Factions,
}
#[derive(Deserialize, Debug, Default, Clone)]
struct Config {
    varbool: bool,
    variant_name: String,
    xl_dir_path: String,
    ware_path: String,
    t_path: String,
    out_path: String,
    pageid: String,
    variant_tname: String,
    mod_name: String,
}
#[derive(Deserialize, Debug, Default, Clone)]
struct Factions {
    // racial factions
    argon: Vec<String>,
    teladi: Vec<String>,
    paranid: Vec<String>,
    xenon: Vec<String>,
    khaak: Vec<String>,
    pirates: Vec<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct xl_config {
    trade_purposemod: f32,
    fight_purposemod: f32,
    build_purposemod: f32,
    mine_purposemod: f32,
    auxiliary_purposemod: f32,
    // first order
    mass: Vec<i32>,
    hull: Vec<i32>,
    cargo: Vec<i32>,
    // second order
    people: Vec<i32>,
    hangarcapacity: Vec<i32>,
    unit: Vec<i32>,
    ammo: Vec<i32>,
    // idfk
    explosion: Vec<i32>,

    // movement
    i_pitch: Vec<i32>,
    i_yaw: Vec<i32>,
    i_roll: Vec<i32>,
    forward: Vec<i32>,
    reverse: Vec<i32>,
    horizontal: Vec<i32>,
    vertical: Vec<i32>,
    d_pitch: Vec<i32>,
    d_yaw: Vec<i32>,
    d_roll: Vec<i32>,
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
    hull: Hull,
    storage: Ammo,
    people: People,
    explosiondamage: Explosion,
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
struct Ammo {
    missile: String,
    unit: String,
}
#[derive(Deserialize, Debug, Default)]
struct People {
    capacity: String,
}
#[derive(Deserialize, Debug, Default)]
struct Explosion {
    value: String,
}
#[derive(Deserialize, Debug, Default)]
struct Purpose {
    primary: String,
    // <purpose primary="fight" />
}
#[derive(Deserialize, Debug, Default)]
struct Hull {
    max: String,
    // <purpose primary="fight" />
}
//
//xml ware
//

#[derive(Deserialize, Debug, Default)]
struct Ware {
    id: String,
    name: String,
    description: String,
    restriction: Restriction,
    owner: Owner,
    component: ComponentWare,
}
#[derive(Deserialize, Debug, Default)]
struct Restriction {
    licence: String,
}
#[derive(Deserialize, Debug, Default)]
struct ComponentWare {
    r#ref: String,
}

#[derive(Deserialize, Debug, Default)]
struct Owner {
    faction: String,
}

//
// xml t
//

#[derive(Deserialize, Debug, Default)]
struct t {
    id: String,
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Deserialize, Debug, Default)]
struct Storage {
    id: String,
    #[serde(rename = "$value")]
    content: String,
}
#[derive(Deserialize, Debug, Default)]
struct Shipstorage {
    id: String,
    #[serde(rename = "$value")]
    content: String,
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
    // README!!!!!!!!!
    // ok, lets talk about how this works.
    // we iterate over some path like D:/x4_extract_2.6/assets/units/size_xl/macros
    // shipstorage, index, tfiles are generated from macro templates
    // a hashmap is used to keep track of values calculated for ships so that when their
    // corresponding storage macro is read the ship stats can be applied to it and the shipstorage macro can be generated

    let mut tname = 1;
    let mut tbase = 2;
    let mut tdesc = 3;
    let mut tvar = 4;
    let mut tshort = 5;
    let mut i_string = "".to_string();
    let mut t_string = "".to_string();
    let mut ware_file_string = "".to_string();
    let ware_new = "".to_string();
    let toml_str = include_str!("Config.toml");
    let toml_parsed: Toml = toml::from_str(&toml_str).unwrap();
    let variant = &toml_parsed.config.varbool;
    let t_path = &toml_parsed.config.t_path;
    let unwrapped_tfile = fs::read_to_string(t_path).unwrap();
    let out_path = toml_parsed.config.out_path.clone();

    let t_out_path = [&out_path, "t/"].concat();
    fs::create_dir_all(&t_out_path).unwrap();
    let i_out_path = [&out_path, "index/"].concat();
    fs::create_dir_all(&i_out_path).unwrap();
    let m_out_path = [&out_path, "macros/"].concat();
    fs::create_dir_all(&m_out_path).unwrap();
    let w_out_path = [&out_path, "libraries/"].concat();
    fs::create_dir_all(&w_out_path).unwrap();

    let mut macro_relations = HashMap::new();

    // invariant!!! - this reads the ships before the storage only because its somehow alphabetized
    for entry in fs::read_dir(&toml_parsed.config.xl_dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            tbase += 100;
            tvar += 100;
            tshort += 100;
            tname += 100;
            tdesc += 100;
            let mut macro_string = fs::read_to_string(&path).unwrap();

            let macro_parsed: Macros = serde_xml_rs::from_str(&macro_string).unwrap_or_default();
            let macroname = &path.file_name().unwrap().to_str().unwrap();

            if macro_string.contains("class=\"storage\"") == true {
                let namecombo = &macroname
                    .replace(".xml", "")
                    .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "_macro"].concat());
                i_string.push_str(&i_add(namecombo.to_string(), toml_parsed.config.mod_name.to_string()));
            }

            if toml_parsed.config.varbool == true {
                let pattern = &macro_parsed.r#macro.name;
                if pattern != "" {
                    let namecombo = &macroname
                        .replace(".xml", "")
                        .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "_macro"].concat());
                    macro_string = replace_pattern(pattern, &macro_string, namecombo);
                    i_string.push_str(&i_add(namecombo.to_string(), toml_parsed.config.mod_name.to_string()));
                }
                let pattern = &macro_parsed.r#macro.properties.identification.name;
                if pattern != "" {
                    //tfile
                    let tname_line = get_tfile_value(&pattern, &unwrapped_tfile);
                    t_string.push_str(&format!("\n{}", &tfile_ware(tname, tname_line, &toml_parsed)));
                    //macro
                    macro_string = replace_pattern(&pattern, &macro_string, &format!("{{{},{}}}", &toml_parsed.config.pageid, tname.to_string()));
                }
                let pattern = &macro_parsed.r#macro.properties.identification.basename;
                if pattern != "" {
                    //tfile
                    let tname_line = get_tfile_value(&pattern, &unwrapped_tfile);
                    t_string.push_str(&format!("\n{}", &tfile_ware(tbase, tname_line, &toml_parsed)));
                    //macro
                    macro_string = replace_pattern(&pattern, &macro_string, &format!("{{{},{}}}", &toml_parsed.config.pageid, tbase.to_string()));
                }
                let pattern = &macro_parsed.r#macro.properties.identification.description;
                if pattern != "" {
                    //tfile
                    let tname_line = get_tfile_value(&pattern, &unwrapped_tfile);
                    t_string.push_str(&format!("\n{}", &tfile_ware(tdesc, tname_line, &toml_parsed)));
                    //macro
                    macro_string = replace_pattern(&pattern, &macro_string, &format!("{{{},{}}}", &toml_parsed.config.pageid, tdesc.to_string()));
                }
                let pattern = &macro_parsed.r#macro.properties.identification.variation;
                if pattern != "" {
                    //tfile
                    let tname_line = get_tfile_value(&pattern, &unwrapped_tfile);
                    t_string.push_str(&format!("\n{}", &tfile_ware(tvar, tname_line, &toml_parsed)));
                    //macro
                    macro_string = replace_pattern(&pattern, &macro_string, &format!("{{{},{}}}", &toml_parsed.config.pageid, tvar.to_string()));
                }
                let pattern = &macro_parsed.r#macro.properties.identification.shortvariation;
                if pattern != "" {
                    //tfile
                    let tname_line = get_tfile_value(&pattern, &unwrapped_tfile);
                    t_string.push_str(&format!("\n{}", &tfile_ware(tshort, tname_line, &toml_parsed)));
                    //macro
                    macro_string = replace_pattern(&pattern, &macro_string, &format!("{{{},{}}}", &toml_parsed.config.pageid, tshort.to_string()));
                }
            }
            // common macro stuff
            // first order
            let mut cargo = 0;
            let mut mass = 0;
            let mut hull = 0;
            let mut rarity = 0;
            // second order
            let mut ammo = 0;
            let mut unit = 0;
            let hangarcapacity = ""; // not used since we do small and medium separately
            let mut people = 0; // TODO not currently affected by purpose_mod, should we add?

            //
            let mut i_pitch = 0;
            let mut i_yaw = 0;
            let mut i_roll = 0;
            let mut forward = 0;
            let mut reverse = 0;
            let mut horizontal = 0;
            let mut vertical = 0;
            let mut d_pitch = 0;
            let mut d_yaw = 0;
            let mut d_roll = 0;

            // let cargo = "";
            // let cargo = "";
            // let cargo = "";
            // let cargo = "";
            // let cargo = "";
            // let cargo = "";
            let purpose = &macro_parsed.r#macro.properties.purpose.primary;
            let mut purpose_mod = 0.6;
            // println!("purpose {}", purpose);
            /*

            might.. want to check the odd assumption.... but hey 50/50... right?
            these ifs determine the ordered values and should eventually contain some unique logic beyond order
            a few points about the ordering:
            1. the order function should not be applied second order values as it loses its influence in longer chains of values.
                a. cargo can roll high to enforce a high mass which in turn can roll an essentially random hull
                b. a + or - values could be propagated through the chain by changing the average calculation to consider the min's
                    position relative to its range in min_and_value
            2. it would be fairly simple to add look ahead or look behind as some form of deterministic mechanism.
            3. rarity is derivable from the first order values.
                a. rarity can be split by tpwar faction type: major, minor, landed, aux, explore
                b. not sure how good it would be to split rarity by faction category: pirate, mercenary, trader, zealot, scavenger
            4.  the second order values are: ammo, people, hangarcapacity. the method described in 1.b might be the best method
            5. see toml comment on ordering for details

            purpose, mass, hull, ammo
            */

            // return_min_and_value
            // if value <= average
            // then min = average
            //
            let mut greater_than_average = false;
            if purpose == "build" {
                purpose_mod = toml_parsed.xlconfig.build_purposemod;
                //mass // done
                let min = &toml_parsed.xlconfig.mass[0];
                let max = &toml_parsed.xlconfig.mass[1];
                mass = return_min_and_value(*min, *max);
                let mass_rarity = get_rarity_float(mass as f32, *min as f32, *max as f32);
                let average = (min + max) / 2;
                if mass >= average {
                    greater_than_average = true;
                }
                // cargo // done
                let mut min = &toml_parsed.xlconfig.cargo[0];
                let mut max = &toml_parsed.xlconfig.cargo[1];
                let average = (min + max) / 2;
                if greater_than_average == true {
                    min = &average;
                } else {
                    max = &average
                }
                cargo = return_min_and_value(*min, *max);
                let cargo_rarity = get_rarity_float(cargo as f32, *min as f32, *max as f32);
                if cargo >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }
                // hull
                let mut min = &toml_parsed.xlconfig.hull[0];
                let mut max = &toml_parsed.xlconfig.hull[1];
                let average = (min + max) / 2;
                if greater_than_average == true {
                    max = &average;
                } else {
                    min = &average
                }
                hull = return_min_and_value(*min, *max);
                let hull_rarity = get_rarity_float(hull as f32, *min as f32, *max as f32);
                if hull >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }
                rarity = set_rarity(cargo_rarity, hull_rarity, mass_rarity);
            }
            if purpose == "fight" {
                purpose_mod = toml_parsed.xlconfig.fight_purposemod;
                //mass // done
                let min = &toml_parsed.xlconfig.mass[0];
                let max = &toml_parsed.xlconfig.mass[1];
                mass = return_min_and_value(*min, *max);
                let mass_rarity = get_rarity_float(mass as f32, *min as f32, *max as f32);
                if mass >= min + max / 2 {
                    greater_than_average = true;
                }
                // cargo // done
                let mut min = &toml_parsed.xlconfig.cargo[0];
                let mut max = &toml_parsed.xlconfig.cargo[1];
                let average = min + max / 2;
                if greater_than_average == true {
                    min = &average;
                } else {
                    max = &average
                }
                cargo = return_min_and_value(*min, *max);
                let cargo_rarity = get_rarity_float(cargo as f32, *min as f32, *max as f32);
                if cargo >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }

                // hull
                let mut min = &toml_parsed.xlconfig.hull[0];
                let mut max = &toml_parsed.xlconfig.hull[1];
                let average = min + max / 2;
                if greater_than_average == true {
                    max = &average;
                } else {
                    min = &average
                }
                hull = return_min_and_value(*min, *max);
                let hull_rarity = get_rarity_float(hull as f32, *min as f32, *max as f32);
                if hull >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }
                rarity = set_rarity(cargo_rarity, hull_rarity, mass_rarity);
            }
            if purpose == "trade" {
                purpose_mod = toml_parsed.xlconfig.trade_purposemod;
                //mass // done
                let min = &toml_parsed.xlconfig.mass[0];
                let max = &toml_parsed.xlconfig.mass[1];
                mass = return_min_and_value(*min, *max);
                let mass_rarity = get_rarity_float(mass as f32, *min as f32, *max as f32);
                if mass >= min + max / 2 {
                    greater_than_average = true;
                }
                // cargo // done
                let mut min = &toml_parsed.xlconfig.cargo[0];
                let mut max = &toml_parsed.xlconfig.cargo[1];
                let average = min + max / 2;
                if greater_than_average == true {
                    min = &average;
                } else {
                    max = &average
                }
                cargo = return_min_and_value(*min, *max);
                let cargo_rarity = get_rarity_float(cargo as f32, *min as f32, *max as f32);
                if cargo >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }

                // hull
                let mut min = &toml_parsed.xlconfig.hull[0];
                let mut max = &toml_parsed.xlconfig.hull[1];
                let average = min + max / 2;
                if greater_than_average == true {
                    max = &average;
                } else {
                    min = &average
                }
                hull = return_min_and_value(*min, *max);
                let hull_rarity = get_rarity_float(hull as f32, *min as f32, *max as f32);
                if hull >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }
                rarity = set_rarity(cargo_rarity, hull_rarity, mass_rarity);
            }
            if purpose == "auxiliary" {
                purpose_mod = toml_parsed.xlconfig.auxiliary_purposemod;
                //mass // done
                let min = &toml_parsed.xlconfig.mass[0];
                let max = &toml_parsed.xlconfig.mass[1];
                mass = return_min_and_value(*min, *max);
                let mass_rarity = get_rarity_float(mass as f32, *min as f32, *max as f32);
                if mass >= min + max / 2 {
                    greater_than_average = true;
                }
                // cargo // done
                let mut min = &toml_parsed.xlconfig.cargo[0];
                let mut max = &toml_parsed.xlconfig.cargo[1];
                let average = min + max / 2;
                if greater_than_average == true {
                    min = &average;
                } else {
                    max = &average
                }
                cargo = return_min_and_value(*min, *max);
                let cargo_rarity = get_rarity_float(cargo as f32, *min as f32, *max as f32);
                if cargo >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }

                // hull
                let mut min = &toml_parsed.xlconfig.hull[0];
                let mut max = &toml_parsed.xlconfig.hull[1];
                let average = min + max / 2;
                if greater_than_average == true {
                    max = &average;
                } else {
                    min = &average
                }
                hull = return_min_and_value(*min, *max);
                let hull_rarity = get_rarity_float(hull as f32, *min as f32, *max as f32);
                if hull >= average {
                    greater_than_average = true;
                } else {
                    greater_than_average = false;
                }
                rarity = set_rarity(cargo_rarity, hull_rarity, mass_rarity);
            }
            // apply purpose modifier to first order values
            cargo = (cargo as f32 * purpose_mod) as i32;
            hull = (hull as f32 * purpose_mod) as i32;
            mass = (mass as f32 * purpose_mod) as i32;

            // physics
            let min = &toml_parsed.xlconfig.i_pitch[0];
            let max = &toml_parsed.xlconfig.i_pitch[1];
            i_pitch = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.i_yaw[0];
            let max = &toml_parsed.xlconfig.i_yaw[1];
            i_yaw = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.i_roll[0];
            let max = &toml_parsed.xlconfig.i_roll[1];
            i_roll = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.forward[0];
            let max = &toml_parsed.xlconfig.forward[1];
            forward = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.reverse[0];
            let max = &toml_parsed.xlconfig.reverse[1];
            reverse = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.horizontal[0];
            let max = &toml_parsed.xlconfig.horizontal[1];
            horizontal = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.vertical[0];
            let max = &toml_parsed.xlconfig.vertical[1];
            vertical = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.d_pitch[0];
            let max = &toml_parsed.xlconfig.d_pitch[1];
            d_pitch = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.d_yaw[0];
            let max = &toml_parsed.xlconfig.d_yaw[1];
            d_yaw = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let min = &toml_parsed.xlconfig.d_roll[0];
            let max = &toml_parsed.xlconfig.d_roll[1];
            d_roll = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            // sometimes this borks out - d_values were outputting values 10x what was calculated above
            let physics = format!(
                "<physics mass=\"{}\">
        <inertia pitch=\"{}\" yaw=\"{}\" roll=\"{}\"/>
        <drag forward=\"{}\" reverse=\"{}\" horizontal=\"{}\" vertical=\"{}\" pitch=\"{}\" yaw=\"{}\" roll=\"{}\"/>
      </physics>",
                mass, i_pitch, i_yaw, i_roll, forward, reverse, horizontal, vertical, d_pitch, d_yaw, d_roll
            );
            let re = Regex::new("((?s)<physics.*</physics>)").unwrap();
            macro_string = re.replace(&macro_string, physics.as_str()).into_owned();
            // storage and shipstorage
            // hull replace
            let pattern = &macro_parsed.r#macro.properties.hull.max;
            if pattern != "" {
                macro_string = macro_string.replace(pattern, &hull.to_string());
            }
            // ammo choose, modify and replace
            let min = &toml_parsed.xlconfig.ammo[0];
            let max = &toml_parsed.xlconfig.ammo[1];
            ammo = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let pattern = &macro_parsed.r#macro.properties.storage.missile;
            if pattern != "" {
                macro_string = macro_string.replace(pattern, &ammo.to_string());
            }
            let min = &toml_parsed.xlconfig.unit[0];
            let max = &toml_parsed.xlconfig.unit[1];
            unit = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let pattern = &macro_parsed.r#macro.properties.storage.unit;
            if pattern != "" {
                macro_string = macro_string.replace(pattern, &unit.to_string());
            }
            // people choose, modify and replace
            let min = &toml_parsed.xlconfig.people[0];
            let max = &toml_parsed.xlconfig.people[1];
            people = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let pattern = &macro_parsed.r#macro.properties.people.capacity;
            if pattern != "" {
                macro_string = macro_string.replace(pattern, &people.to_string());
            }
            // explosion choose, modify and replace
            let min = &toml_parsed.xlconfig.explosion[0];
            let max = &toml_parsed.xlconfig.explosion[1];
            let explosion = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
            let pattern = &macro_parsed.r#macro.properties.explosiondamage.value;
            if pattern != "" {
                macro_string = macro_string.replace(pattern, &explosion.to_string());
            }
            let mut small = 0;
            if macro_string.contains("shipstorage_gen_s_01_macro") == true {
                let min = &toml_parsed.xlconfig.hangarcapacity[0];
                let max = &toml_parsed.xlconfig.hangarcapacity[1];
                small = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
                // replace name
                let namecombo = &macroname
                    .replace(".xml", "")
                    .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "size_s", "_macro"].concat())
                    .replace("ship", "shipstorage");
                macro_string = macro_string.replace("shipstorage_gen_s_01_macro", namecombo);
                i_string.push_str(&i_add(namecombo.to_string(), toml_parsed.config.mod_name.to_string()));
            }
            let mut medium = 0;
            if macro_string.contains("shipstorage_gen_m_01_macro") == true {
                let min = &toml_parsed.xlconfig.hangarcapacity[2];
                let max = &toml_parsed.xlconfig.hangarcapacity[3];
                medium = (return_min_and_value(*min, *max) as f32 * purpose_mod) as i32;
                //  replace name
                let namecombo = &macroname
                    .replace(".xml", "")
                    .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "size_m", "_macro"].concat())
                    .replace("ship", "shipstorage");
                macro_string = macro_string.replace("shipstorage_gen_m_01_macro", namecombo);
                i_string.push_str(&i_add(namecombo.to_string(), toml_parsed.config.mod_name.to_string()));
            }
            
            // storage name
                if macro_string.contains("storage_arg_xl_builder_01_a_macro") {println!("{:?}", macroname)}
                let pattern = &macroname.replace(".xml", "").replace("ship", "storage");
                macro_string = replace_pattern(
                    &pattern,
                    &macro_string,
                    &macroname
                        .replace(".xml", "")
                        .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "_macro"].concat())
                        .replace("ship", "storage"),
                );
                // // storage name
                // let pattern = &macroname.replace(".xml", "").replace("ship", "storage");
                // macro_string = replace_pattern(
                //     &pattern,
                //     &macro_string,
                //     &macroname
                //         .replace(".xml", "")
                //         .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "_macro"].concat())
                //         .replace("ship", "storage"),
                // );
            // table!
            macro_relations.insert(macroname.to_string(), (cargo.to_string(), small, medium));
        
            if macro_relations.contains_key(&macroname.to_owned().to_string().replace("ship", "storage")) {
                // oh this is either really smart or really dumb
                //cargo value
                let re = Regex::new("<cargo max=\".*\" ta").unwrap();
                macro_string = re
                    .replace(
                        &macro_string,
                        format!(
                            "<cargo max=\"{}\" ta",
                            macro_relations.get(&macroname.replace("storage", "ship")).unwrap().0.as_str()
                        )
                        .as_str(),
                    )
                    .to_string();
                // storage name
                let pattern = &macroname.replace(".xml", "").replace("ship", "storage");
                macro_string = replace_pattern(
                    &pattern,
                    &macro_string,
                    &macroname
                        .replace(".xml", "")
                        .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), "_macro"].concat())
                        .replace("ship", "storage"),
                );

                let medium = &macro_relations.get(&macroname.replace("storage", "ship").to_owned()).unwrap().2;
                if medium > &0 {
                    let size = "size_m";
                    makeshipstorage(&toml_parsed, &m_out_path, &macroname.to_string(), &size.to_string(), &medium.to_string());
                }
                let small = &macro_relations.get(&macroname.replace("storage", "ship").to_owned()).unwrap().1;
                if small > &0 {
                    let size = "size_s";
                    makeshipstorage(&toml_parsed, &m_out_path, &macroname.to_string(), &size.to_string(), &small.to_string());
                }
            }

            // ware

            let ware_string = fs::read_to_string(&toml_parsed.config.ware_path).unwrap();
            for ware in ware_string.split_terminator("</ware>") {
                if ware.contains(&macroname.replace(".xml", "")) == true {
                    let mut ware_new = "".to_string();

                    ware_new.push_str(&ware);

                    ware_new.push_str("\n</ware>");
                    let ware_parsed: Ware = serde_xml_rs::from_str(&ware_new).unwrap_or_default();
                    let mut prng = rand::thread_rng();
                    let ware_price = format!(
                        "<price min=\"{}\" average=\"{}\" max=\"{}\" />",
                        randomize(prng.gen_range(0.5, 5.0), 25000),
                        randomize(prng.gen_range(0.5, 5.0), 25000),
                        randomize(prng.gen_range(0.5, 5.0), 25000)
                    );
                    let re = Regex::new("<price.* />").unwrap();
                    let mut ware_new = re.replace(&ware_new, ware_price.as_str()).into_owned();

                    //##############################################
                    // VARAINT!
                    //##############################################
                    if toml_parsed.config.varbool == true {
                        let pattern = &ware_parsed.id;
                        if pattern != "" {
                            ware_new = replace_pattern(
                                &pattern,
                                &ware_new,
                                &macroname.replace(".xml", "").replace("_macro", &toml_parsed.config.variant_name.as_str()),
                            );
                        }

                        let rarity = rarity as usize + 2;
                        let pattern = &ware_parsed.owner.faction;

                        if pattern != "" {
                            for faction in toml_parsed.faction_vec.argon.iter() {
                                if pattern == faction {
                                    let owner_string = &ownership(toml_parsed.faction_vec.argon.choose_multiple(&mut rand::thread_rng(), rarity)).to_owned();
                                    let re = Regex::new("<owner.* />").unwrap();
                                    if owner_string != "" {
                                        ware_new = re.replace(&ware_new, owner_string.as_str()).into_owned();
                                    }
                                }
                            }
                            for faction in toml_parsed.faction_vec.teladi.iter() {
                                if pattern == faction {
                                    let owner_string = &ownership(toml_parsed.faction_vec.teladi.choose_multiple(&mut rand::thread_rng(), rarity)).to_owned();
                                    let re = Regex::new("<owner.* />").unwrap();
                                    if owner_string != "" {
                                        ware_new = re.replace(&ware_new, owner_string.as_str()).into_owned();
                                    }
                                }
                            }
                            for faction in toml_parsed.faction_vec.paranid.iter() {
                                if pattern == faction {
                                    let owner_string = &ownership(toml_parsed.faction_vec.paranid.choose_multiple(&mut rand::thread_rng(), rarity)).to_owned();
                                    let re = Regex::new("<owner.* />").unwrap();
                                    if owner_string != "" {
                                        ware_new = re.replace(&ware_new, owner_string.as_str()).into_owned();
                                    }
                                }
                            }
                            for faction in toml_parsed.faction_vec.xenon.iter() {
                                if pattern == faction {
                                    let owner_string = &ownership(toml_parsed.faction_vec.xenon.choose_multiple(&mut rand::thread_rng(), rarity)).to_owned();
                                    let re = Regex::new("<owner.* />").unwrap();
                                    if owner_string != "" {
                                        ware_new = re.replace(&ware_new, owner_string.as_str()).into_owned();
                                    }
                                }
                            }
                            for faction in toml_parsed.faction_vec.khaak.iter() {
                                if pattern == faction {
                                    let owner_string = &ownership(toml_parsed.faction_vec.khaak.choose_multiple(&mut rand::thread_rng(), rarity)).to_owned();
                                    let re = Regex::new("<owner.* />").unwrap();
                                    if owner_string != "" {
                                        ware_new = re.replace(&ware_new, owner_string.as_str()).into_owned();
                                    }
                                }
                            }
                            for faction in toml_parsed.faction_vec.pirates.iter() {
                                if pattern == faction {
                                    let owner_string = &ownership(toml_parsed.faction_vec.pirates.choose_multiple(&mut rand::thread_rng(), rarity)).to_owned();
                                    let re = Regex::new("<owner.* />").unwrap();
                                    if owner_string != "" {
                                        ware_new = re.replace(&ware_new, owner_string.as_str()).into_owned();
                                    }
                                }
                            }
                        }
                    }
                    let pattern = &ware_parsed.name;
                    if pattern != "" {
                        ware_new = replace_pattern(&pattern, &ware_new, &format!("{{{},{}}}", &toml_parsed.config.pageid, tname.to_string()));
                    }
                    let pattern = &ware_parsed.description;
                    if pattern != "" {
                        ware_new = replace_pattern(&pattern, &ware_new, &format!("{{{},{}}}", &toml_parsed.config.pageid, tbase.to_string()));
                    }

                    ware_file_string.push_str("\n");
                    if variant == &true {
                        ware_file_string.push_str(&ware_new);
                    } else {
                        ware_file_string.push_str(&ware_new);
                    }
                }
            }

            output(&m_out_path, &path, &toml_parsed.config.variant_name, &macro_string);
        }
    }

    let mut outputfile = File::create(format!("{}{}", &w_out_path, "wares.xml")).unwrap();
    outputfile.write_all(ware_file_string.as_bytes()).unwrap();
    let mut outputfile = File::create(format!("{}{}", &t_out_path, "tfiles.xml")).unwrap();
    outputfile.write_all(t_string.as_bytes()).unwrap();
    let mut outputfile = File::create(format!("{}{}", &i_out_path, "index.xml")).unwrap();
    outputfile.write_all(i_string.as_bytes()).unwrap();
}

fn ownership(owners: rand::seq::SliceChooseIter<'_, [std::string::String], std::string::String>) -> (String) {
    let mut owner_string = "".to_string();
    let mut owners_vec: Vec<String> = vec![];
    for owner in owners {
        if owners_vec.contains(owner) == false {
            owner_string.push_str(&format!("\n    <owner faction=\"{}\"/>", owner));
            // println!("{}", owner_string);
            owners_vec.push(owner.to_string())
        }
    }
    owners_vec.clear();
    owner_string
}

fn makeshipstorage(toml_parsed: &Toml, m_out_path: &String, macroname: &String, size: &String, count: &String) -> () {
    let shipstorage_string = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
 <!--Exported by: Michael (192.168.3.150) at 09.11.2017_11-30-00-->
 <macros>
   <macro name=\"{}\" class=\"dockingbay\">
     <component ref=\"generic_dockingbay\" />
     <properties>
       <identification unique=\"0\" />
       <dock capacity=\"{}\" external=\"0\" storage=\"1\" />
       <room walkable=\"0\" />
       <docksize tags=\"{}\" />
     </properties>
   </macro>
 </macros>",
        &macroname
            .replace(".xml", "")
            .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), size.as_str(), "_macro"].concat())
            .replace("storage", "shipstorage"),
        count,
        size
    );

    let mut outputfile = File::create(format!(
        "{}{}",
        &m_out_path,
        &macroname
            .replace("storage", "shipstorage")
            .replace("_macro", &[&toml_parsed.config.variant_name.as_str(), size.as_str(), "_macro"].concat())
    ))
    .unwrap();
    outputfile.write_all(shipstorage_string.as_bytes()).unwrap();
}

// input min and max of expected range -> min or average of the range, and value of the range result.
fn return_min_and_value(min: i32, max: i32) -> (i32) {
    let mut prng = rand::thread_rng();
    let mut returnmin = 0;
    let value = prng.gen_range(min, max);
    value
}

fn replace_pattern(pattern: &String, text: &String, replace: &str) -> String {
    if pattern != "" {
        let text = &text.replace(pattern.as_str(), &replace);
        text.to_string()
    } else {
        text.to_string()
    }
}
// randomize(prng.gen_range(0.5, 5.0), 25000);
fn randomize(multi: f32, input: i32) -> String {
    let result = multi * input as f32;
    (result as i32).to_string()
}

fn tfile_ware(tnum: i32, tname_line: String, toml_parsed: &Toml) -> String {
    let mut tname_line = tname_line;
    let t_line_parsed: t = serde_xml_rs::from_str(&tname_line).unwrap_or_default();
    tname_line = tname_line.replace(&t_line_parsed.id, &tnum.to_string());
    tname_line = tname_line.replace(
        &t_line_parsed.content,
        &format!("{} {}", &t_line_parsed.content, &toml_parsed.config.variant_tname),
    );
    tname_line
}

fn output(path: &String, pathbuf: &std::path::PathBuf, variant: &String, macro_string: &String) {
    let mut outputfile = File::create(
        format!("{}{}", path, pathbuf.file_name().unwrap().to_str().unwrap().to_string()).replace("_macro", &[&variant.as_str(), "_macro"].concat()),
    )
    .unwrap();
    outputfile.write_all(macro_string.as_bytes()).unwrap();
}

// Alby's tfile stuff
// use t_path from Config toml and id string from parsed macro/ware
fn get_tfile_value(id_tfile: &String, unwrapped_tfile: &str) -> String {
    let re = Regex::new(r"\d+").unwrap();
    let mut tfile_vec = vec![];
    for caps in re.captures_iter(&id_tfile) {
        let num = caps.get(0).unwrap().as_str();
        tfile_vec.push(num)
    }

    let mut tfile_value = "".to_string();
    let mut flag = false;
    for line in unwrapped_tfile.lines() {
        if flag == false {
            if line.contains(format!("<page id=\"{}", tfile_vec[0]).as_str()) {
                flag = true
            };
        } else {
            if line.contains(tfile_vec[1]) {
                tfile_value.push_str(line);
                break;
            };
        }
    }

    tfile_value
}

fn i_add(macroname: String, folderpath: String) -> String {
    let i_add_value = format!("<entry name=\"{}\" value=\"{}\\{}\" />\n", macroname, folderpath, macroname);
    i_add_value.to_string()
}

fn get_rarity_float(value: f32, min: f32, max: f32) -> f32 {
    let average = (min + max) / 2.0;
    let mut rarity_float: f32 = 0.0;
    if value > average {
        rarity_float = (value - average) / (max - average)
    } else if value < average {
        rarity_float = (value - average) / (average - min)
    }
    // println!("value: {}, min: {}, average {}, max {}, float: {}", value, min, average, max, rarity_float);
    rarity_float
}

fn set_rarity(cargo: f32, hull: f32, mass: f32) -> i32 {
    let mut rarity: i32;
    let average = (cargo + hull - mass) / 3.0;
    if average < -0.6 {
        rarity = 5
    } else if average < -0.2 {
        rarity = 4
    } else if average < 0.2 {
        rarity = 3
    } else if average < 0.4 {
        rarity = 2
    } else {
        rarity = 1
    }
    rarity
}
