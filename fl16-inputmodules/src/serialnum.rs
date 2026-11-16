// Get serial number from last 4K block of the first 1M
const FLASH_OFFSET: usize = 0x10000000;
const LAST_4K_BLOCK: usize = 0xff000;
const SERIALNUM_LEN: usize = 18;

#[repr(packed)]
pub struct SerialnumStructRaw {
    sn_rev: u8,
    serialnum: [u8; SERIALNUM_LEN],
    crc32: [u8; 4],
}

pub struct SerialnumStruct {
    pub serialnum: &'static str,
}

pub fn get_serialnum() -> Option<SerialnumStruct> {
    // Flash is mapped into memory, just read it from there
    let ptr: *const u8 = (FLASH_OFFSET + LAST_4K_BLOCK) as *const u8;
    let sn_raw_ptr = ptr as *const SerialnumStructRaw;
    let sn_raw = unsafe { sn_raw_ptr.as_ref()? };

    // Only rev 1 supported
    if sn_raw.sn_rev != 1 {
        return None;
    }

    let crc: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let mut digest = crc.digest();
    digest.update(&[sn_raw.sn_rev]);
    digest.update(&sn_raw.serialnum);
    let calc_checksum = digest.finalize();

    let actual_checksum = u32::from_le_bytes(sn_raw.crc32);
    // Checksum invalid, serial fall back to default serial number
    if calc_checksum != actual_checksum {
        return None;
    }

    Some(SerialnumStruct {
        serialnum: core::str::from_utf8(&sn_raw.serialnum).ok()?,
    })
}

/// Get the firmware version in a format for USB Device Release
/// The value is in binary coded decimal with a format of 0xJJMN where JJ is the major version number, M is the minor version number and N is the sub minor version number. e.g. USB 2.0 is reported as 0x0200, USB 1.1 as 0x0110 and USB 1.0 as 0x0100.
pub fn device_release() -> u16 {
    (env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap() << 8)
        + (env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap() << 4)
        + env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap()
}

pub fn is_pre_release() -> bool {
    !env!("CARGO_PKG_VERSION_PRE").is_empty()
}

/// White Rabbit build number: the N of the `build-N` release this firmware was built for, set by the
/// release workflow through the `WR_FIRMWARE_BUILD` environment variable. 0 when built by hand.
pub const WR_BUILD: u16 = parse_build(option_env!("WR_FIRMWARE_BUILD"));

const fn parse_build(text: Option<&str>) -> u16 {
    let bytes = match text {
        Some(t) => t.as_bytes(),
        None => return 0,
    };
    let mut n: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b < b'0' || b > b'9' {
            return 0;
        }
        n = n * 10 + (b - b'0') as u32;
        if n > u16::MAX as u32 {
            return 0;
        }
        i += 1;
    }
    n as u16
}

const WR_TAG_PREFIX: &[u8; 10] = b"WRFW-BUILD";

const fn wr_build_tag() -> [u8; 12] {
    let mut tag = [0u8; 12];
    let mut i = 0;
    while i < WR_TAG_PREFIX.len() {
        tag[i] = WR_TAG_PREFIX[i];
        i += 1;
    }
    let n = WR_BUILD.to_le_bytes();
    tag[10] = n[0];
    tag[11] = n[1];
    tag
}

/// "WRFW-BUILD" and the build number (little endian), kept in flash so an app can read the build
/// number out of a .uf2 file before flashing it.
#[used]
pub static WR_BUILD_TAG: [u8; 12] = wr_build_tag();

/// The build number, read through the tag so the tag stays in the firmware image.
pub fn wr_build() -> u16 {
    // Safety: reading a static that is always initialised
    let tag = unsafe { core::ptr::read_volatile(&WR_BUILD_TAG) };
    u16::from_le_bytes([tag[10], tag[11]])
}
