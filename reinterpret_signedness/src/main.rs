use std::slice::{from_raw_parts, from_raw_parts_mut};

trait CanBeCast: Sized {
    type Out: Sized;
}

impl CanBeCast for i8 {
    type Out = u8;
}
impl CanBeCast for i16 {
    type Out = u16;
}
impl CanBeCast for i32 {
    type Out = u32;
}
impl CanBeCast for i64 {
    type Out = u64;
}

impl CanBeCast for u8 {
    type Out = i8;
}
impl CanBeCast for u16 {
    type Out = i16;
}
impl CanBeCast for u32 {
    type Out = i32;
}
impl CanBeCast for u64 {
    type Out = i64;
}

trait ReinterpretSignedness<Out>
where
    Out: ?Sized,
{
    fn with_other_signedness(&self) -> &Out;

    fn with_other_signedness_mut(&mut self) -> &mut Out;
}

impl<In, Out> ReinterpretSignedness<[Out]> for [In]
where
    In: CanBeCast<Out = Out>,
    Out: Sized,
{
    fn with_other_signedness(&self) -> &[Out] {
        // SAFETY: Only safe where bits can be reinterpreted.
        unsafe { from_raw_parts(self.as_ptr() as *mut () as *mut Out, self.len()) }
    }

    fn with_other_signedness_mut(&mut self) -> &mut [Out] {
        // SAFETY: Only safe where bits can be reinterpreted.
        unsafe { from_raw_parts_mut(self.as_ptr() as *mut () as *mut Out, self.len()) }
    }
}

fn main() {
    let s1: &[i8] = &[1, 2, 3, 4];
    let _s1_as_u8: &[u8] = s1.with_other_signedness();

    let s2: &mut [u32] = &mut [10, 20, 30, 40, 50];
    let _s2_as_i32: &[i32] = s2.with_other_signedness_mut();
}
