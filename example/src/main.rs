use reborrow::{IntoConst, Reborrow, ReborrowMut};

mod shared {
    use reborrow::ReborrowCopy;

    #[derive(ReborrowCopy)]
    pub struct I32Ref<'a, 'b> {
        pub i: i32,
        pub j: &'a i32,
        pub k: &'b i32,
    }

    #[derive(ReborrowCopy)]
    pub struct I32TupleRef<'a, 'b>(pub i32, pub &'a i32, pub &'b i32);

    #[derive(ReborrowCopy)]
    pub struct Ref<'a, 'b, T> {
        pub i: i32,
        pub j: &'a T,
        pub k: &'b T,
    }
}

#[derive(Reborrow)]
#[Const(shared::I32Ref)]
struct I32RefMut<'a, 'b> {
    i: i32,
    #[reborrow]
    j: &'a mut i32,
    #[reborrow]
    k: &'b mut i32,
}

#[derive(Reborrow)]
#[Const(shared::I32TupleRef)]
pub struct I32TupleRefMut<'a, 'b>(i32, #[reborrow] &'a mut i32, #[reborrow] &'b mut i32);

fn main() {
    let i = 0;
    let j = &mut 0;
    let k = &mut 0;
    {
        let mut r = I32RefMut { i, j, k };
        let _unused = r.rb_mut();
        let _unused = r.rb();
        let _unused = r.into_const();
    }

    {
        let mut r = I32TupleRefMut(i, j, k);
        let _unused = r.rb();
        let _unused = r.rb_mut();
        let _unused = r.into_const();
    }
    println!("Hello, world!");
}
