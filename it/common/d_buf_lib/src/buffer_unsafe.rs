use std::mem;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug,Copy,Clone)]
struct MyStruct { // note inverting a and b make the app to crash as b need to be 8bytes aligned
    a: f64,
    b: u32,
    c: u32,
}

#[repr(C)]
#[derive(Debug,Copy,Clone)]
struct MyStruct2 { //bytemuck make it to works well
    b: u32,
    a: f64,
    c: u32,
}

// SAFETY: MyStruct2 is `Pod` because it only contains `Pod` types.
unsafe impl Pod for MyStruct2 {}
unsafe impl Zeroable for MyStruct2 {}
unsafe impl Pod for MyStruct{}
unsafe impl Zeroable for MyStruct {}


#[cfg(test)]
mod tests {
    use crate::buffer_unsafe::{MyStruct, MyStruct2};

    #[test]
    fn test_bytemuck(){
        let mut bytes = vec![0u8; 16]; // 8 bytes for f64 + 4 bytes for u32

        bytes[0..8].copy_from_slice(&1524.65f64.to_le_bytes());
        bytes[8..12].copy_from_slice(&4u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&5415u32.to_le_bytes());

        let my_struct2: &mut MyStruct = bytemuck::from_bytes_mut(&mut bytes);
        assert_eq!(my_struct2.a, 1524.65f64);

        // Fill the byte vector with some data (e.g., for testing)
        bytes[0..4].copy_from_slice(&4u32.to_le_bytes());
        bytes[4..12].copy_from_slice(&1524.65f64.to_le_bytes());
        bytes[12..16].copy_from_slice(&5415u32.to_le_bytes());

        let my_struct2: &mut MyStruct2 = bytemuck::from_bytes_mut(&mut bytes);
        assert_eq!(my_struct2.a, 1524.65f64);

    }

    #[test]
    fn test_bytemuck_Mult(){
        let mut bytes = vec![0u8; 52]; // 8 bytes for f64 + 4 bytes for u32

        // Fill the byte vector with some data (e.g., for testing)
        bytes[0..8].copy_from_slice(&1524.65f64.to_le_bytes());
        bytes[8..12].copy_from_slice(&4u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&5415u32.to_le_bytes());

        bytes[16..24].copy_from_slice(&12.65f64.to_le_bytes());
        bytes[24..28].copy_from_slice(&4u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&5415u32.to_le_bytes());

        bytes[32..36].copy_from_slice(&4u32.to_le_bytes());
        bytes[36..44].copy_from_slice(&17.65f64.to_le_bytes());
        bytes[44..48].copy_from_slice(&4u32.to_le_bytes());
        bytes[48..52].copy_from_slice(&5415u32.to_le_bytes());


        let my_struct1_1: &mut MyStruct = bytemuck::from_bytes_mut(&mut bytes[0..16]);
        assert_eq!(my_struct1_1.a, 1524.65f64);
        let my_struct1_2: &mut MyStruct = bytemuck::from_bytes_mut(&mut bytes[16..32]);
        assert_eq!(my_struct1_2.a, 12.65f64);

        let my_struct1_3: &mut MyStruct = bytemuck::from_bytes_mut(&mut bytes[36..52]); // KO not aligned
        assert_eq!(my_struct1_3.a, 12.65f64);


        // Fill the byte vector with some data (e.g., for testing)
        bytes[0..4].copy_from_slice(&4u32.to_le_bytes());
        bytes[4..12].copy_from_slice(&1524.65f64.to_le_bytes());
        bytes[12..16].copy_from_slice(&5415u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&4u32.to_le_bytes());
        bytes[20..28].copy_from_slice(&12.65f64.to_le_bytes());
        bytes[28..32].copy_from_slice(&5415u32.to_le_bytes());

        let my_struct1: &mut MyStruct2 = bytemuck::from_bytes_mut(&mut bytes[0..16]); // KO offset not correct for 8 bytes (not 8 multiple)
        assert_eq!(my_struct1.a, 1524.65f64);

    }

    #[test]
    fn test_single(){
        let mut bytes = vec![0u8; 16]; // 8 bytes for f64 + 4 bytes for u32

        // Fill the byte vector with some data (e.g., for testing)
        bytes[0..8].copy_from_slice(&8.0f64.to_le_bytes());
        bytes[8..12].copy_from_slice(&42u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&42u32.to_le_bytes());

        let my_struct: &mut MyStruct = unsafe {
            let ptr = bytes[0..16].as_mut_ptr() as *mut MyStruct;
            &mut *ptr
        };
        assert_eq!(my_struct.a, 8.0f64);
        my_struct.a = 3.14;
        assert_eq!(my_struct.a, 3.14);
    }
    #[test]
    fn test_multi(){
        let mut bytes = vec![0u8; 32]; // 8 bytes for f64 + 4 bytes for u32

        // Fill the byte vector with some data (e.g., for testing)
        bytes[0..8].copy_from_slice(&8.5f64.to_le_bytes());
        bytes[8..12].copy_from_slice(&42u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&42u32.to_le_bytes());
        bytes[16..24].copy_from_slice(&9.0f64.to_le_bytes());
        bytes[24..28].copy_from_slice(&43u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&43u32.to_le_bytes());
/*
        let my_struct: &mut MyStruct = unsafe {
            let ptr = bytes[0..12].as_mut_ptr() as *mut MyStruct;
            &mut *ptr
        };
        assert_eq!(my_struct.a, 8.5f64);
*/
        let my_struct2: &mut MyStruct = unsafe {
            let ptr = bytes[16..32].as_mut_ptr() as *mut MyStruct;
            &mut *ptr
        };
        assert_eq!(my_struct2.a, 9.0f64);

        /*
        let vec_struct: Vec<MyStruct> = unsafe {
            Vec::from_raw_parts(bytes.as_mut_ptr() as *mut MyStruct, 2, 2)
        };
        let sum = vec_struct.iter().map(|s| s.a + s.b as f64).sum::<f64>();

        assert_eq!(sum, 101.5);*/
    }

    #[test]
    fn test_second(){
        let mut bytes = vec![0u8; 32]; // 8 bytes for f64 + 4 bytes for u32

        // Fill the byte vector with some data (e.g., for testing)
        bytes[0..8].copy_from_slice(&8.5f64.to_le_bytes());
        bytes[8..12].copy_from_slice(&42u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&42u32.to_le_bytes());
        bytes[16..24].copy_from_slice(&9.0f64.to_le_bytes());
        bytes[24..28].copy_from_slice(&43u32.to_le_bytes());
        bytes[28..32].copy_from_slice(&43u32.to_le_bytes());
        let my_struct2: &mut MyStruct = unsafe {
            let ptr = bytes[16..32].as_mut_ptr() as *mut MyStruct;
            &mut *ptr
        };
        assert_eq!(my_struct2.a, 9.0f64);
    }
}

