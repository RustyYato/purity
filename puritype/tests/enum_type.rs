use puritype::puritype;

puritype! {
    enum List {
        Cons(Type, List),
        #[must_use]
        Nil
    }
}

// struct Cons<T, L>(T, L);
// struct Nil;

// pub trait List {}

// impl<T, L: List> List for Cons<T, L> {}
// impl List for Nil {}