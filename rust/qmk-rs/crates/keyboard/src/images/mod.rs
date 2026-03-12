use include_image::include_image;

use crate::utils::{Size, Sizeable};

include_image!("./images/test.png");
include_image!("./images/test2.png");
include_image!("./images/lego.png");
include_image!("./images/green.png", 4);
include_image!("./images/orange.png", 4);
// include_image!("./images/font_small.png");
// include_image!("./images/font_large.png");
// include_image!("./images/font_huge.png");
include_image!("./images/font_huge_a.png");
// include_image!("./images/font_huge_0.png");

impl Sizeable for dyn include_image::Image {
    fn size(&self) -> crate::utils::Size {
        Size {
            width: self.get_width() as u16,
            height: self.get_height() as u16,
        }
    }
}
