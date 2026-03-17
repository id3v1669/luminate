use crate::kit::sonata::palette::ColorScale;
use crate::kit::sonata::palette::rgb;

pub static SCALE_RED: ColorScale = ColorScale {
    s25: rgb(255, 251, 250),
    s30: rgb(255, 0, 0), // need to be generated
    s40: rgb(255, 0, 0), // need to be generated
    s50: rgb(254, 243, 242),
    s100: rgb(254, 228, 226),
    s200: rgb(253, 195, 190),
    s300: rgb(253, 162, 155),
    s400: rgb(246, 115, 105),
    s500: rgb(240, 68, 56),
    s600: rgb(210, 60, 48),
    s700: rgb(181, 53, 41),
    s800: rgb(151, 46, 33),
    s900: rgb(122, 39, 26),
};
