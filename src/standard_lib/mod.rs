pub mod crypto;
pub mod csv;
pub mod dns;
pub mod fs;
pub mod hcl;
pub mod http;
pub mod json;
pub mod llm;
pub mod math;
pub mod os;
pub mod sys;
pub mod xml;
pub mod yaml;

use crate::runtime::RuntimeState;
use crate::type_system::Value;
use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;

pub fn call(
    name: &str,
    args: &[Value],
    named_args: &HashMap<String, Value>,
    _state: &RuntimeState,
) -> CorvoResult<Value> {
    match name {
        "sys.echo" => sys::echo(args, named_args),
        "sys.read_line" => sys::read_line(args, named_args),
        "sys.sleep" => sys::sleep(args, named_args),
        "sys.panic" => sys::panic(args, named_args),
        "sys.exec" => sys::exec(args, named_args),

        "os.get_env" => os::get_env(args, named_args),
        "os.set_env" => os::set_env(args, named_args),
        "os.exec" => os::exec(args, named_args),
        "os.info" => os::info(args, named_args),

        "math.add" => math::add(args, named_args),
        "math.sub" => math::sub(args, named_args),
        "math.mul" => math::mul(args, named_args),
        "math.div" => math::div(args, named_args),
        "math.mod" => math::modulo(args, named_args),

        "fs.read" => fs::read(args, named_args),
        "fs.write" => fs::write(args, named_args),
        "fs.append" => fs::append(args, named_args),
        "fs.delete" => fs::delete(args, named_args),
        "fs.exists" => fs::exists(args, named_args),
        "fs.mkdir" => fs::mkdir(args, named_args),
        "fs.list_dir" => fs::list_dir(args, named_args),
        "fs.copy" => fs::copy(args, named_args),
        "fs.move" => fs::move_file(args, named_args),
        "fs.stat" => fs::stat(args, named_args),

        "http.get" => http::get(args, named_args),
        "http.post" => http::post(args, named_args),
        "http.put" => http::put(args, named_args),
        "http.delete" => http::delete(args, named_args),

        "dns.resolve" => dns::resolve(args, named_args),
        "dns.lookup" => dns::lookup(args, named_args),

        "crypto.hash" => crypto::hash(args, named_args),
        "crypto.encrypt" => crypto::encrypt(args, named_args),
        "crypto.decrypt" => crypto::decrypt(args, named_args),
        "crypto.uuid" => crypto::uuid(args, named_args),

        "json.parse" => json::parse_value(args, named_args),
        "json.stringify" => json::stringify(args, named_args),

        "yaml.parse" => yaml::parse_value(args, named_args),
        "yaml.stringify" => yaml::stringify(args, named_args),

        "hcl.parse" => hcl::parse_value(args, named_args),
        "hcl.stringify" => hcl::stringify(args, named_args),

        "csv.parse" => csv::parse_value(args, named_args),

        "xml.parse" => xml::parse_value(args, named_args),

        "llm.model" => llm::model(args, named_args),
        "llm.prompt" => llm::prompt(args, named_args),
        "llm.embed" => llm::embed(args, named_args),
        "llm.chat" => llm::chat(args, named_args),

        // Type methods
        s if s.starts_with("string.") => {
            crate::type_system::type_methods::call_string_method(s, args)
        }
        s if s.starts_with("number.") => {
            crate::type_system::type_methods::call_number_method(s, args)
        }
        s if s.starts_with("list.") => crate::type_system::type_methods::call_list_method(s, args),
        s if s.starts_with("map.") => crate::type_system::type_methods::call_map_method(s, args),

        _ => Err(CorvoError::unknown_function(name)),
    }
}
