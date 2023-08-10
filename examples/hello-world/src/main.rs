use std::rc::Rc;

use rudi::{Context, Singleton};

trait Service {
    fn hello(&self) -> &str;
}

#[derive(Clone)]
#[Singleton(name = "hello", binds = [Self::into_service])]
struct ServiceImpl;

impl ServiceImpl {
    fn into_service(self) -> Rc<dyn Service> {
        Rc::new(self)
    }
}

impl Service for ServiceImpl {
    fn hello(&self) -> &str {
        "Hello World!"
    }
}

#[derive(Clone)]
#[Singleton(name = "controller")]
struct Controller {
    #[di("hello")]
    s: Rc<dyn Service>,
}

impl Controller {
    fn hello(&self) -> &str {
        self.s.hello()
    }
}

#[derive(Clone)]
struct Hello;

#[Singleton]
impl Hello {
    fn new() -> Hello {
        println!("Hello::new");
        Hello
    }
}

#[Singleton]
fn Run(#[di("controller")] controller: Controller, num: i32, success: bool, _: Hello) {
    println!("{}", controller.hello());

    println!("num: {}", num);

    println!("success: {}", success);
}

fn main() {
    let mut cx = Context::options()
        .instance(42)
        .instance(true)
        .auto_register();

    // cx.resolve::<()>();
    cx.resolve()
}
