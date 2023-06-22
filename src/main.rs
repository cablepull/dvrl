
#[macro_use] 
 extern crate rocket;
 extern crate log;
 extern crate fern;
 extern crate chrono;
 extern crate config;
 
 
use rocket::{get, post, routes, Rocket, Build};
use rocket::serde::{json::Json};
use libloading::{Library, Symbol};
use std::env;
// Define the type signature for the process_input function
type ProcessInputFn = extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_char;

// Load the shared library and retrieve the process_input function
fn load_library() -> Result<Library, &'static str> {
    unsafe{
        
        let out_dir = env::var("OUT_DIR").unwrap();
        let path = format!("{}/mypi.so",out_dir);
        info!("Attempting to load {}", path);
        let library = match { Library::new(path) } {
            Ok(lib) => lib,
            Err(_) => return Err("Failed to load the shared library"),
        };
    
    Ok(library)
    }
}

fn get_process_input_function(library: &Library) -> Result<Symbol<ProcessInputFn>, &'static str> {
    
    unsafe{
        info!("Attempting to load func");
        match { library.get(b"processInput") } {
            Ok(function) => Ok(function),
            Err(_) => Err("Failed to retrieve the process_input function"),
        }
    }
}

// Define your Rust function that wraps the process_input function from the shared library
fn process_input(input: &str) -> String {
    // Load the shared library
    let library = match load_library() {
        Ok(lib) => lib,
        Err(err) => return format!("Error: {}", err),
    }; 

    // Retrieve the process_input function from the shared library
    let  process_input_fn = match get_process_input_function(&library) {
        Ok(func) => func,
        Err(err) => return format!("Error: {}", err),
    };
 
    unsafe{
    // Convert the input string to a C-compatible format
    let c_string = std::ffi::CString::new(input).expect("CString::new failed");
    
    // Call the process_input function from the shared library
    let result =  {
        let result_ptr = process_input_fn(c_string.as_ptr());
        let result_c_str = std::ffi::CStr::from_ptr(result_ptr);
        let result_string = result_c_str.to_string_lossy().into_owned();
        //libc::free(result_ptr as *mut _); // Deallocate the memory allocated by the C function
        result_string
    };
    
    // Unload the shared library
    drop(library);

    result
    }
}

#[get("/")]
fn hello() -> Json<String> {
    let result = "{ \"Hello\":\"World\" }";
    Json(result.to_string())
}

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}


// Define a Rocket route that invokes your Rust function
#[post("/", format = "json", data = "<user_input>")]
fn process_route(user_input: Json<String>) -> Json<String> {
    trace!("Yo momma-----------");
    let result = process_input(&user_input);
    Json(result)
}



fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = setup_logger();
    let rocket = Rocket::build().mount("/", routes![hello])
                                .mount("/hello", routes![world])
                                .mount("/process", routes![process_route]);

    info!("Working!");  
    
    let result = rocket.launch().await;
    assert!(result.is_ok());

    Ok(())
}