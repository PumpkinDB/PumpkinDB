#[macro_export]
macro_rules! write_size_into_slice {
    ($size:expr, $slice: expr) => {
     match $size {
        0...120 => {
            $slice[0] = $size as u8;
            1
        }
        121...255 => {
            $slice[0] = 121u8;
            $slice[1] = $size as u8;
            2
        }
        256...65535 => {
            $slice[0] = 122u8;
            $slice[1] = ($size >> 8) as u8;
            $slice[2] = $size as u8;
            3
        }
        65536...4294967296 => {
            $slice[0] = 123u8;
            $slice[1] = ($size >> 24) as u8;
            $slice[2] = ($size >> 16) as u8;
            $slice[3] = ($size >> 8) as u8;
            $slice[4] = $size as u8;
            5
        }
        _ => unreachable!(),
    }
    };
}

#[macro_export]
macro_rules! write_size {
    ($size: expr, $vec: expr) => {{
        let mut header = vec![0;offset_by_size($size)];
        write_size_into_slice!($size, header.as_mut_slice());
        $vec.append(&mut header);
    }};
}

#[macro_export]
macro_rules! write_size_header {
    ($bytes: expr, $vec: expr) => {{
        write_size!($bytes.len(), $vec);
    }};
}
