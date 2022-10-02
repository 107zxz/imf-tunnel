use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Tunnel;

impl Tunnel {
    fn new(_base: &Node) -> Self {
        Tunnel
    }
}

#[methods]
impl Tunnel {

    #[method]
    fn _ready(&self, #[base] base: &Node) {
        godot_print!("Sussy BAKA!");
    }

}