use std::rc::Rc;

enum Fruit {
    Apple,
    Orange
}

struct Bar{
    value: u32
}

struct Foo {
    bar1: Bar,
    bar2: Bar,
    tasty: Fruit
}

impl Foo {
    fn do_something(self: &Rc<Self>) {
        // let bar = match self.tasty {
        //     Fruit::Apple => &mut self.bar1,
        //     Fruit::Orange => &mut self.bar2
        // };
        let mut bar = Rc::clone(&self);

        self.do_something_else();

        bar.bar1.value = 10;
    }

    fn do_something_else(&mut self) {
        // something else that mutates Foo
    }
}