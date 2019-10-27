struct SomethingThatIsActivated{}
impl OnActivated for SomethingThatIsActivated {
    fn on_activated(){
    println!("I have been activated")
}
}


trait OnActivated {
    fn on_activated(&self);
}

struct Activatable<T: HasOnActivatedMethod> {
    bool activated;
}

impl 