use include_image::include_image;

use crate::utils::{Size, Sizeable};

include_image!("./images/lego.png");
include_image!("./images/green.png", 4);
include_image!("./images/orange.png", 4);

impl Sizeable for dyn include_image::Image {
    fn size(&self) -> crate::utils::Size {
        Size {
            width: self.get_width() as u16,
            height: self.get_height() as u16,
        }
    }
}
