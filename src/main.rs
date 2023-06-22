
#[macro_use] extern crate rocket;
use rocket::{get, post, routes, Rocket, Build};
use rocket::serde::{Serialize, Deserialize, json::Json};
use libloading::{Library, Symbol};

// Define the type signature for the process_input function
type ProcessInputFn = extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_char;

// Load the shared library and retrieve the process_input function
fn load_library() -> Result<Library, &'static str> {
    unsafe{
        let library = match { Library::new("./process_input.so") } {
            Ok(lib) => lib,
            Err(_) => return Err("Failed to load the shared library"),
        };
    
    Ok(library)
    }
}

fn get_process_input_function(library: &Library) -> Result<Symbol<ProcessInputFn>, &'static str> {
    
    unsafe{

        match { library.get(b"process_input\0") } {
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
    let process_input_fn = match get_process_input_function(&library) {
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
#[post("/process/<input>")]
fn process_route(input: String) -> Json<String> {
    let result = process_input(&input);
    Json(result)
}

// Start the Rocket web server
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![hello])
                   .mount("/hello", routes![world])
                   .mount("/process", routes![process_route])
}

fn main() {
    rocket().launch();
}