use reborrow::{IntoConst, Reborrow, ReborrowMut};

mod a {
    #[derive(Clone, Copy)]
    pub struct I32Ref<'a, 'b> {
        pub i: i32,
        pub j: &'a i32,
        pub k: &'b i32,
    }

    #[derive(Clone, Copy)]
    pub struct I32TupleRef<'a, 'b>(pub i32, pub &'a i32, pub &'b i32);
}

#[derive(Reborrow)]
#[Const(a::I32Ref)]
struct I32RefMut<'a, 'b> {
    #[copy]
    i: i32,
    j: &'a mut i32,
    k: &'b mut i32,
}

#[derive(Reborrow)]
#[Const(a::I32TupleRef)]
pub struct I32TupleRefMut<'a, 'b>(#[copy] i32, &'a mut i32, &'b mut i32);

fn main() {
    let i = 0;
    let j = &mut 0;
    let k = &mut 0;
    {
        let mut r = I32RefMut { i, j, k };
        r.rb();
        r.rb_mut();
        r.into_const();
    }

    {
        let mut r = I32TupleRefMut(i, j, k);
        r.rb();
        r.rb_mut();
        r.into_const();
    }
    println!("Hello, world!");
}
