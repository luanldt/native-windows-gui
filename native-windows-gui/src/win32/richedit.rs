//! winapi-rs does not implements richedit.h, so here's the low level stuff
//! implemented here instead of in rich_text_box because it's kind of messy
use winapi::um::winuser::WM_USER;
use winapi::um::wingdi::{LF_FACESIZE, RGB};
use winapi::shared::{
    minwindef::{UINT, DWORD, WORD, BYTE},
    ntdef::{LONG, SHORT, LCID},
    windef::{HWND, COLORREF}
};
use crate::win32::window_helper as wh;
use crate::win32::base_helper::{to_utf16, from_utf16};
use crate::controls::{CharFormat, CharEffects, UnderlineType::*};
use std::{mem, ptr};

const EM_GETCHARFORMAT: u32 = WM_USER + 58;
const EM_SETCHARFORMAT: u32 = WM_USER + 68;
const SCF_SELECTION: u32 = 1;

const CFM_EFFECTS: u32 = 0x001 | 0x002 | 0x004 | 0x008 | 0x010 | 0x020 | 0x40000000;
const CFM_SIZE: u32 = 0x80000000;
const CFM_OFFSET: u32 = 0x10000000;
const CFM_COLOR: u32 = 0x40000000;
const CFM_FACE: u32 = 0x20000000;
const CFM_UNDERLINETYPE: u32 = 0x00800000;

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Default)]
struct CHARFORMATW {
    cbSize: UINT,
    dwMask: DWORD,
    dwEffects: DWORD,
    yHeight: LONG,
    yOffset: LONG,
    crTextColor: COLORREF,
    bCharSet: BYTE,
    bPitchAndFamily: BYTE,
    szFaceName: [u16; LF_FACESIZE],
    wWeight: WORD,
    sSpacing: SHORT,
    crBackColor: COLORREF,
    lcid: LCID,
    reserved: DWORD,
    sStyle: SHORT,
    wKerning: WORD,
    bUnderlineType: BYTE,
    bAnimation: BYTE,
    bRevAuthor: BYTE,
    bUnderlineColor: BYTE
}

pub(crate) fn set_char_format(handle: HWND, fmt: &CharFormat) {

    let mut mask = 0;
    if fmt.effects.is_some() { mask |= CFM_EFFECTS; }
    if fmt.height.is_some() { mask |= CFM_SIZE; }
    if fmt.y_offset.is_some() { mask |= CFM_OFFSET; }
    if fmt.text_color.is_some() { mask |= CFM_COLOR; }
    if fmt.font_face_name.is_some() { mask |= CFM_FACE; }
    if fmt.underline_type.is_some() { mask |= CFM_UNDERLINETYPE }
    let mut color = 0;
    if let Some([r, g, b]) = fmt.text_color {
        color = RGB(r, g, b);
    }

    let mut face: [u16; LF_FACESIZE] = [0; LF_FACESIZE];
    if let Some(face_name) = fmt.font_face_name.as_ref() {
        let face_name = to_utf16(&face_name);
        if face_name.len() >= LF_FACESIZE {
            panic!("Font face name cannot be longer than {:?} characters", LF_FACESIZE);
        }

        unsafe {
            ptr::copy_nonoverlapping(face_name.as_ptr(), face.as_mut_ptr(), face_name.len());
        }
    }

    let mut underline_type = 0;
    if let Some(under) = fmt.underline_type {
        underline_type = match under {
            None => 0,
            Solid => 1,
            Dash => 5,
            DashDot => 6,
            DashDotDot => 7,
            Dotted => 4,
            DoubleSolid => 3,
            Wave => 8,
        };
    }

    let mut fmt = CHARFORMATW {
        cbSize: mem::size_of::<CHARFORMATW>() as _,
        dwMask: mask,
        dwEffects: fmt.effects.map(|e| e.bits()).unwrap_or(0),
        yHeight: fmt.height.unwrap_or(0),
        yOffset: fmt.y_offset.unwrap_or(0),
        crTextColor: color,
        bCharSet: 0,
        bPitchAndFamily: 0,
        szFaceName: face,
        wWeight: 0,
        sSpacing: 0,
        crBackColor: 0,
        lcid: 0,
        reserved: 0,
        sStyle: 0,
        wKerning: 0,
        bUnderlineType: underline_type,
        bAnimation: 0,
        bRevAuthor: 0,
        bUnderlineColor: 0
    };

    wh::send_message(handle, EM_SETCHARFORMAT, SCF_SELECTION as _, &mut fmt as *mut CHARFORMATW as _);
}

pub(crate) fn char_format(handle: HWND) -> CharFormat {
    use winapi::um::wingdi::{GetRValue, GetGValue, GetBValue};

    let mut fmt: CHARFORMATW = CHARFORMATW {
        cbSize: mem::size_of::<CHARFORMATW>() as _,
        ..Default::default()
    };

    wh::send_message(handle, EM_GETCHARFORMAT, SCF_SELECTION as _, &mut fmt as *mut CHARFORMATW as _);

    let effects = Some(CharEffects::from_bits_truncate(fmt.dwEffects));

    let mut height = Option::None;
    if fmt.yHeight != 0 {
        height = Some(fmt.yHeight);
    }

    let mut y_offset = Option::None;
    if fmt.yOffset != 0 {
        y_offset = Some(fmt.yOffset);
    }

    let mut text_color = Option::None;
    if fmt.crTextColor != 0 {
        text_color = Some([
            GetRValue(fmt.crTextColor),
            GetGValue(fmt.crTextColor),
            GetBValue(fmt.crTextColor),
        ]);
    }

    let underline_type = match fmt.bUnderlineType {
        1 => Some(Solid),
        5 => Some(Dash),
        6 => Some(DashDot),
        7 => Some(DashDotDot),
        4 => Some(Dotted),
        3 => Some(DoubleSolid),
        8 => Some(Wave),
        _ => Option::None,
    };

    let mut font_face_name = Option::None;
    if fmt.szFaceName[0] != 0 {
        font_face_name = Some(from_utf16(&fmt.szFaceName));
    }

    CharFormat {
        effects,
        height,
        y_offset,
        text_color,
        font_face_name,
        underline_type,
    }
}